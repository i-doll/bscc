//! L4 integration test: bscc-lang-lsl populates the tree-sitter-tier fields
//! and the LSL plugin overrides the regex-tier LSL entry.

use std::path::PathBuf;

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("lsl")
}

fn build_registry() -> bscc_core::Registry {
    let mut r = bscc_core::Registry::new();
    bscc_regex_tier::register(&mut r);
    bscc_lang_lsl::register(&mut r);
    r
}

#[test]
fn lsl_file_gets_tree_sitter_metrics() {
    let registry = build_registry();
    let report = bscc_core::walk(
        &[&fixtures_root()],
        &registry,
        &bscc_core::WalkOptions {
            respect_gitignore: false,
            ..Default::default()
        },
    );

    let f = report
        .files
        .iter()
        .find(|f| f.path.file_name().is_some_and(|n| n == "sample.lsl"))
        .expect("sample.lsl walked");

    assert_eq!(f.language, "LSL");
    // 2 global functions + 4 event handlers (3 in default, 1 in idle).
    assert_eq!(f.functions, Some(6), "functions count");
    // classify() has 3 `if` arms -> CC 4. That's the max in the fixture.
    assert_eq!(f.cyclomatic_max, Some(4), "cyclomatic_max");
    // greet=1, classify=4, state_entry(default)=1, touch_start=3, listen=2,
    // state_entry(idle)=1.  Sum = 12.
    assert_eq!(f.cyclomatic_total, Some(12), "cyclomatic_total");
    assert_eq!(f.todo_comments, Some(1), "TODO comment counted");
    assert!(f.longest_function_lines.is_some_and(|n| n > 0));
}

#[test]
fn lsl_registered_at_tree_sitter_tier() {
    let registry = build_registry();
    let entry = registry.lookup_by_extension("lsl").expect("lsl registered");
    assert_eq!(entry.name, "LSL");
    assert_eq!(entry.tier, bscc_core::Tier::TreeSitter);
}

#[test]
fn explain_returns_per_function_breakdown_for_lsl() {
    let registry = build_registry();
    let path = fixtures_root().join("sample.lsl");
    let source = std::fs::read(&path).unwrap();
    let entry = registry.lookup_by_path(&path).expect("lsl registered");
    let details = entry
        .analyzer
        .explain(&path, &source)
        .expect("tree-sitter tier supports explain");
    assert_eq!(details.len(), 6);
    let max_cc = details.iter().map(|d| d.cyclomatic).max().unwrap();
    assert_eq!(max_cc, 4);
}
