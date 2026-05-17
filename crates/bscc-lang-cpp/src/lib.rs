use bscc_ast_tier::TreeSitterAnalyzer;
use bscc_core::{LanguageEntry, Registry, Tier};
use std::sync::Arc;

const QUERY: &str = include_str!("../queries/metrics.scm");

pub fn register(registry: &mut Registry) {
    let language: tree_sitter::Language = tree_sitter_cpp::LANGUAGE.into();
    let analyzer = TreeSitterAnalyzer::new("C++".into(), language, QUERY)
        .expect("bundled C++ metrics.scm must compile");
    registry.register(LanguageEntry {
        name: "C++".into(),
        extensions: vec![
            "cpp".into(),
            "cc".into(),
            "cxx".into(),
            "hpp".into(),
            "hxx".into(),
            "hh".into(),
        ],
        filenames: vec![],
        tier: Tier::TreeSitter,
        analyzer: Arc::new(analyzer),
    });
}
