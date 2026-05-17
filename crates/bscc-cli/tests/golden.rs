use std::path::PathBuf;

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("sample")
}

#[test]
fn walks_fixture_tree_and_classifies_each_language() {
    let mut registry = bscc_core::Registry::new();
    bscc_regex_tier::register(&mut registry);

    let root = fixtures_root();
    let report = bscc_core::walk(
        &[&root],
        &registry,
        &bscc_core::WalkOptions {
            respect_gitignore: false,
            ..Default::default()
        },
    );

    // Snapshot the per-language totals. File-level paths are environment-
    // dependent so we only snapshot the aggregate.
    let by_lang = report.by_language();
    let snapshot = serde_json::to_string_pretty(&by_lang).unwrap();
    insta::assert_snapshot!(snapshot);
}

#[test]
fn lsl_files_are_detected() {
    let mut registry = bscc_core::Registry::new();
    bscc_regex_tier::register(&mut registry);

    let root = fixtures_root();
    let report = bscc_core::walk(
        &[&root],
        &registry,
        &bscc_core::WalkOptions {
            respect_gitignore: false,
            ..Default::default()
        },
    );

    let lsl = report
        .files
        .iter()
        .find(|f| f.language == "LSL")
        .expect("hello.lsl should be detected as LSL");
    assert!(lsl.lines > 0);
    assert!(lsl.code > 0);
    assert!(lsl.comments > 0);
}
