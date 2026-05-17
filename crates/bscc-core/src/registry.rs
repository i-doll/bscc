use crate::FileMetrics;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Which tier produced a given `FileMetrics`. Used by exporters to decide
/// which columns are meaningful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Regex,
    TreeSitter,
}

/// A registered language and the analyzer that produces metrics for it.
pub struct LanguageEntry {
    pub name: String,
    pub extensions: Vec<String>,
    /// Exact-match filenames (e.g. "Makefile", "Dockerfile", ".dockerignore").
    /// Filename matches win over extension matches.
    pub filenames: Vec<String>,
    pub tier: Tier,
    pub analyzer: Arc<dyn Analyzer>,
}

/// Analyzers consume the bytes of one file and produce metrics. Both tiers
/// implement this so the engine dispatches uniformly. The default
/// `explain` implementation returns `None`; tree-sitter-tier analyzers
/// override it to provide per-function detail for `bscc explain`.
pub trait Analyzer: Send + Sync {
    fn analyze(&self, path: &Path, source: &[u8]) -> FileMetrics;
    fn explain(&self, _path: &Path, _source: &[u8]) -> Option<Vec<crate::FunctionDetail>> {
        None
    }
}

/// Holds all registered languages. Plugin crates call `register()` once at
/// startup to add themselves. Lookups are by language name or by file path
/// extension.
#[derive(Default)]
pub struct Registry {
    entries: Vec<Arc<LanguageEntry>>,
    by_name: HashMap<String, Arc<LanguageEntry>>,
    by_ext: HashMap<String, Arc<LanguageEntry>>,
    by_filename: HashMap<String, Arc<LanguageEntry>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a language entry. If a language with the same name was
    /// already registered, the new entry replaces it — this is how a
    /// tree-sitter language plugin overrides a regex-tier fallback.
    pub fn register(&mut self, entry: LanguageEntry) {
        let entry = Arc::new(entry);
        if let Some(pos) = self.entries.iter().position(|e| e.name == entry.name) {
            self.entries[pos] = Arc::clone(&entry);
        } else {
            self.entries.push(Arc::clone(&entry));
        }
        self.by_name.insert(entry.name.clone(), Arc::clone(&entry));
        for ext in &entry.extensions {
            self.by_ext
                .insert(ext.to_ascii_lowercase(), Arc::clone(&entry));
        }
        for fname in &entry.filenames {
            self.by_filename
                .insert(fname.to_ascii_lowercase(), Arc::clone(&entry));
        }
    }

    pub fn lookup_by_name(&self, name: &str) -> Option<&LanguageEntry> {
        self.by_name.get(name).map(AsRef::as_ref)
    }

    pub fn lookup_by_extension(&self, ext: &str) -> Option<&LanguageEntry> {
        self.by_ext
            .get(&ext.to_ascii_lowercase())
            .map(AsRef::as_ref)
    }

    pub fn lookup_by_filename(&self, name: &str) -> Option<&LanguageEntry> {
        self.by_filename
            .get(&name.to_ascii_lowercase())
            .map(AsRef::as_ref)
    }

    /// Resolve `path` to a registered language. Priority order:
    ///
    /// 1. **Exact filename** (e.g. `Dockerfile`, `.dockerignore`).
    /// 2. **Compound extensions, longest first** — for `vars.pkr.hcl` we
    ///    try `pkr.hcl` before `hcl`, so Packer wins over generic HCL when
    ///    both are registered. Single extensions are the shortest case and
    ///    so are still tried last for free.
    pub fn lookup_by_path(&self, path: &Path) -> Option<&LanguageEntry> {
        let name = path.file_name().and_then(|n| n.to_str())?;

        if let Some(entry) = self.lookup_by_filename(name) {
            return Some(entry);
        }

        let lower = name.to_ascii_lowercase();
        let mut start = 0;
        while let Some(dot) = lower[start..].find('.') {
            start += dot + 1;
            if let Some(entry) = self.by_ext.get(&lower[start..]) {
                return Some(entry.as_ref());
            }
        }
        None
    }

    pub fn languages(&self) -> impl Iterator<Item = &LanguageEntry> {
        self.entries.iter().map(AsRef::as_ref)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileMetrics;

    struct StubAnalyzer;
    impl Analyzer for StubAnalyzer {
        fn analyze(&self, path: &Path, _: &[u8]) -> FileMetrics {
            FileMetrics::new(path.to_path_buf(), "stub".into(), 0)
        }
    }

    fn entry(name: &str, exts: &[&str]) -> LanguageEntry {
        LanguageEntry {
            name: name.into(),
            extensions: exts.iter().map(|s| (*s).into()).collect(),
            filenames: vec![],
            tier: Tier::Regex,
            analyzer: Arc::new(StubAnalyzer),
        }
    }

    fn registry() -> Registry {
        let mut r = Registry::new();
        r.register(entry("HCL", &["hcl"]));
        r.register(entry("Terraform", &["tf", "tfvars", "tf.json"]));
        r.register(entry("Packer", &["pkr.hcl", "pkrvars.hcl", "pkr.json"]));
        r.register(entry("JSON", &["json"]));
        r
    }

    fn detect(r: &Registry, p: &str) -> Option<String> {
        r.lookup_by_path(Path::new(p)).map(|e| e.name.clone())
    }

    #[test]
    fn compound_extension_packer_wins_over_hcl() {
        let r = registry();
        assert_eq!(detect(&r, "builder.pkr.hcl").as_deref(), Some("Packer"));
        assert_eq!(detect(&r, "vars.pkrvars.hcl").as_deref(), Some("Packer"));
    }

    #[test]
    fn compound_extension_terraform_json_wins_over_json() {
        let r = registry();
        assert_eq!(detect(&r, "infra.tf.json").as_deref(), Some("Terraform"));
    }

    #[test]
    fn plain_extensions_still_resolve() {
        let r = registry();
        assert_eq!(detect(&r, "main.tf").as_deref(), Some("Terraform"));
        assert_eq!(detect(&r, "vars.tfvars").as_deref(), Some("Terraform"));
        assert_eq!(detect(&r, "config.hcl").as_deref(), Some("HCL"));
        assert_eq!(detect(&r, "data.json").as_deref(), Some("JSON"));
    }

    #[test]
    fn case_insensitive_compound_match() {
        let r = registry();
        assert_eq!(detect(&r, "Build.PKR.HCL").as_deref(), Some("Packer"));
    }

    #[test]
    fn unknown_extension_returns_none() {
        let r = registry();
        assert_eq!(detect(&r, "thing.xyzzy"), None);
    }
}
