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
    pub tier: Tier,
    pub analyzer: Arc<dyn Analyzer>,
}

/// Analyzers consume the bytes of one file and produce metrics. Both tiers
/// implement this so the engine dispatches uniformly.
pub trait Analyzer: Send + Sync {
    fn analyze(&self, path: &Path, source: &[u8]) -> FileMetrics;
}

/// Holds all registered languages. Plugin crates call `register()` once at
/// startup to add themselves. Lookups are by language name or by file path
/// extension.
#[derive(Default)]
pub struct Registry {
    entries: Vec<Arc<LanguageEntry>>,
    by_name: HashMap<String, Arc<LanguageEntry>>,
    by_ext: HashMap<String, Arc<LanguageEntry>>,
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
    }

    pub fn lookup_by_name(&self, name: &str) -> Option<&LanguageEntry> {
        self.by_name.get(name).map(AsRef::as_ref)
    }

    pub fn lookup_by_extension(&self, ext: &str) -> Option<&LanguageEntry> {
        self.by_ext
            .get(&ext.to_ascii_lowercase())
            .map(AsRef::as_ref)
    }

    pub fn lookup_by_path(&self, path: &Path) -> Option<&LanguageEntry> {
        let ext = path.extension().and_then(|e| e.to_str())?;
        self.lookup_by_extension(ext)
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
