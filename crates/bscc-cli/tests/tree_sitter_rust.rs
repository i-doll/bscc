//! M2 integration test: bscc-lang-rust populates the tree-sitter-tier fields.

use std::path::PathBuf;

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("rust")
}

fn build_registry() -> bscc_core::Registry {
    let mut r = bscc_core::Registry::new();
    bscc_regex_tier::register(&mut r);
    bscc_lang_rust::register(&mut r);
    r
}

#[test]
fn rust_file_gets_tree_sitter_metrics() {
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
        .find(|f| f.path.file_name().is_some_and(|n| n == "lib.rs"))
        .expect("lib.rs walked");

    assert_eq!(f.language, "Rust");
    // 4 named functions in the fixture.
    assert_eq!(f.functions, Some(4), "functions count");
    // The `classify` function has 3 `if` arms + 1 `&&` = 4 branches, CC=5.
    assert_eq!(f.cyclomatic_max, Some(5), "cyclomatic_max");
    // sum of: add=1, classify=5, sum_evens=3, build_index=2 = 11.
    assert_eq!(f.cyclomatic_total, Some(11), "cyclomatic_total");
    assert_eq!(f.imports, Some(1), "imports");
    assert_eq!(f.todo_comments, Some(1), "todo_comments");
    assert!(f.longest_function_lines.is_some_and(|n| n > 0));
}

#[test]
fn rust_registered_at_tree_sitter_tier() {
    let registry = build_registry();
    let entry = registry.lookup_by_extension("rs").expect("rust registered");
    assert_eq!(entry.name, "Rust");
    assert_eq!(entry.tier, bscc_core::Tier::TreeSitter);
}

#[test]
fn json_format_includes_ast_fields_for_rust() {
    use bscc_core::Exporter;
    let registry = build_registry();
    let report = bscc_core::walk(
        &[&fixtures_root()],
        &registry,
        &bscc_core::WalkOptions {
            respect_gitignore: false,
            ..Default::default()
        },
    );

    let mut buf = Vec::new();
    bscc_export::JsonExporter { pretty: false }
        .write(&report, &mut buf)
        .unwrap();
    let text = String::from_utf8(buf).unwrap();
    assert!(
        text.contains("\"functions\":4"),
        "functions in JSON: {text}"
    );
    assert!(
        text.contains("\"cyclomatic_max\":5"),
        "cyclomatic_max in JSON: {text}"
    );
    assert!(
        text.contains("\"todo_comments\":1"),
        "todo_comments in JSON: {text}"
    );
}
