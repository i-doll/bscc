use bscc_ast_tier::TreeSitterAnalyzer;
use bscc_core::{LanguageEntry, Registry, Tier};
use std::sync::Arc;

const QUERY: &str = include_str!("../queries/metrics.scm");

pub fn register(registry: &mut Registry) {
    {
        let language: tree_sitter::Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        let analyzer = TreeSitterAnalyzer::new("TypeScript".into(), language, QUERY)
            .expect("bundled TypeScript metrics.scm must compile");
        registry.register(LanguageEntry {
            name: "TypeScript".into(),
            extensions: vec!["ts".into(), "mts".into(), "cts".into()],
            filenames: vec![],
            tier: Tier::TreeSitter,
            analyzer: Arc::new(analyzer),
        });
    }
    {
        let language: tree_sitter::Language = tree_sitter_typescript::LANGUAGE_TSX.into();
        let analyzer = TreeSitterAnalyzer::new("TSX".into(), language, QUERY)
            .expect("bundled TypeScript metrics.scm must compile against TSX grammar");
        registry.register(LanguageEntry {
            name: "TSX".into(),
            extensions: vec!["tsx".into()],
            filenames: vec![],
            tier: Tier::TreeSitter,
            analyzer: Arc::new(analyzer),
        });
    }
}
