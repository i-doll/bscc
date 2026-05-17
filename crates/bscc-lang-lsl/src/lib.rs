//! Tree-sitter-backed LSL language plugin for bscc.

use bscc_ast_tier::TreeSitterAnalyzer;
use bscc_core::{LanguageEntry, Registry, Tier};
use std::sync::Arc;

const QUERY: &str = include_str!("../queries/metrics.scm");

pub fn register(registry: &mut Registry) {
    let language: tree_sitter::Language = tree_sitter_lsl::LANGUAGE.into();
    let analyzer = TreeSitterAnalyzer::new("LSL".into(), language, QUERY)
        .expect("bundled LSL metrics.scm must compile");
    registry.register(LanguageEntry {
        name: "LSL".into(),
        extensions: vec!["lsl".into()],
        filenames: vec![],
        tier: Tier::TreeSitter,
        analyzer: Arc::new(analyzer),
    });
}
