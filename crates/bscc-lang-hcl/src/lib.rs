//! Tree-sitter-backed HCL2 language plugin for bscc.
//!
//! Covers the HCL family: HCL itself plus the Terraform, `OpenTofu`, and
//! Packer dialects. All four share one grammar and one metrics query —
//! the only difference is which extensions and dialect name each
//! `LanguageEntry` claims, preserving the per-dialect split that the
//! report aggregator groups by. The query is small enough that compiling
//! it four times (once per dialect) is cheaper than the alternative
//! plumbing to share a single `Query` across analyzers.
//!
//! JSON-syntax variants (`.tf.json`, `.pkr.json`) are NOT covered here:
//! the HCL2 grammar cannot parse them. They remain at the regex tier
//! under dedicated `Terraform JSON` / `Packer JSON` language entries.

use bscc_ast_tier::TreeSitterAnalyzer;
use bscc_core::{LanguageEntry, Registry, Tier};
use std::sync::Arc;

const QUERY: &str = include_str!("../queries/metrics.scm");

pub fn register(registry: &mut Registry) {
    for (name, extensions) in [
        ("HCL", &["hcl"][..]),
        ("Terraform", &["tf", "tfvars"][..]),
        ("OpenTofu", &["tofu"][..]),
        ("Packer", &["pkr.hcl", "pkrvars.hcl"][..]),
    ] {
        let language: tree_sitter::Language = tree_sitter_hcl::LANGUAGE.into();
        let analyzer = TreeSitterAnalyzer::new(name.into(), language, QUERY)
            .expect("bundled HCL metrics.scm must compile");
        registry.register(LanguageEntry {
            name: name.into(),
            extensions: extensions.iter().map(|s| (*s).into()).collect(),
            filenames: vec![],
            tier: Tier::TreeSitter,
            analyzer: Arc::new(analyzer),
        });
    }
}
