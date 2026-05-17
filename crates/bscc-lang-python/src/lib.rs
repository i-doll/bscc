use bscc_ast_tier::TreeSitterAnalyzer;
use bscc_core::{LanguageEntry, Registry, Tier};
use std::sync::Arc;

const QUERY: &str = include_str!("../queries/metrics.scm");

pub fn register(registry: &mut Registry) {
    let language: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();
    let analyzer = TreeSitterAnalyzer::new("Python".into(), language, QUERY)
        .expect("bundled Python metrics.scm must compile");
    registry.register(LanguageEntry {
        name: "Python".into(),
        extensions: vec!["py".into(), "pyi".into()],
        tier: Tier::TreeSitter,
        analyzer: Arc::new(analyzer),
    });
}
