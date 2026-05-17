//! Tree-sitter-backed Rust language plugin for bscc.

use bscc_ast_tier::TreeSitterAnalyzer;
use bscc_core::{LanguageEntry, Registry, Tier};
use std::sync::Arc;

const QUERY: &str = include_str!("../queries/metrics.scm");

pub fn register(registry: &mut Registry) {
    let language: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
    let analyzer = TreeSitterAnalyzer::new("Rust".into(), language, QUERY)
        .expect("bundled Rust metrics.scm must compile");
    registry.register(LanguageEntry {
        name: "Rust".into(),
        extensions: vec!["rs".into()],
        filenames: vec![],
        tier: Tier::TreeSitter,
        analyzer: Arc::new(analyzer),
    });
}
