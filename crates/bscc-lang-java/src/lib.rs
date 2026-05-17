use bscc_ast_tier::TreeSitterAnalyzer;
use bscc_core::{LanguageEntry, Registry, Tier};
use std::sync::Arc;

const QUERY: &str = include_str!("../queries/metrics.scm");

pub fn register(registry: &mut Registry) {
    let language: tree_sitter::Language = tree_sitter_java::LANGUAGE.into();
    let analyzer = TreeSitterAnalyzer::new("Java".into(), language, QUERY)
        .expect("bundled Java metrics.scm must compile");
    registry.register(LanguageEntry {
        name: "Java".into(),
        extensions: vec!["java".into()],
        tier: Tier::TreeSitter,
        analyzer: Arc::new(analyzer),
    });
}
