use bscc_ast_tier::TreeSitterAnalyzer;
use bscc_core::{LanguageEntry, Registry, Tier};
use std::sync::Arc;

const QUERY: &str = include_str!("../queries/metrics.scm");

pub fn register(registry: &mut Registry) {
    let language: tree_sitter::Language = tree_sitter_c::LANGUAGE.into();
    let analyzer = TreeSitterAnalyzer::new("C".into(), language, QUERY)
        .expect("bundled C metrics.scm must compile");
    registry.register(LanguageEntry {
        name: "C".into(),
        extensions: vec!["c".into(), "h".into()],
        tier: Tier::TreeSitter,
        analyzer: Arc::new(analyzer),
    });
}
