//! scc-style declarative tokenizer. One crate of data + a shared state machine.
//! Plugin crates would normally own a grammar; here we own a TOML config.

mod config;
mod tokenizer;

use bscc_core::{Analyzer, FileMetrics, LanguageEntry, Registry, Tier};
use std::path::Path;
use std::sync::Arc;

use config::LanguageConfig;

const LANGUAGES_TOML: &str = include_str!("../data/languages.toml");

#[derive(serde::Deserialize)]
struct Languages {
    language: Vec<LanguageConfig>,
}

/// Register every language defined in `data/languages.toml` with the registry.
pub fn register(registry: &mut Registry) {
    let parsed: Languages =
        toml::from_str(LANGUAGES_TOML).expect("bundled languages.toml must parse");
    for cfg in parsed.language {
        let cfg = Arc::new(cfg);
        let analyzer: Arc<dyn Analyzer> = Arc::new(RegexAnalyzer {
            cfg: Arc::clone(&cfg),
        });
        registry.register(LanguageEntry {
            name: cfg.name.clone(),
            extensions: cfg.extensions.clone(),
            filenames: cfg.filenames.clone(),
            tier: Tier::Regex,
            analyzer,
        });
    }
}

struct RegexAnalyzer {
    cfg: Arc<LanguageConfig>,
}

impl Analyzer for RegexAnalyzer {
    fn analyze(&self, path: &Path, source: &[u8]) -> FileMetrics {
        let counts = tokenizer::count(&self.cfg, source);
        let mut m = FileMetrics::new(
            path.to_path_buf(),
            self.cfg.name.clone(),
            source.len() as u64,
        );
        m.lines = counts.lines;
        m.code = counts.code;
        m.comments = counts.comments;
        m.blanks = counts.blanks;
        m
    }
}
