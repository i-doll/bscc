//! M5 integration: HTML exporter shape + bscc.toml loader.

use bscc_core::Exporter;
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
fn html_exporter_emits_self_contained_document() {
    let report = bscc_core::walk(
        &[&fixtures_root()],
        &build_registry(),
        &bscc_core::WalkOptions {
            respect_gitignore: false,
            ..Default::default()
        },
    );

    let mut buf = Vec::new();
    bscc_export::HtmlExporter.write(&report, &mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();

    // Self-contained: no external CSS/JS links.
    assert!(!text.contains("<link"), "no external CSS");
    assert!(!text.contains("<script"), "no external JS");
    // Has the expected structural elements.
    assert!(text.starts_with("<!doctype html>"));
    assert!(text.contains("<style>"));
    assert!(text.contains("bscc report"));
    // Per-language and per-file tables present.
    assert!(text.contains("<h2>Languages</h2>"));
    assert!(text.contains("<h2>Files</h2>"));
    // The Rust fixture's numbers leak through.
    assert!(text.contains(">Rust<"));
    assert!(text.contains("lib.rs"));
}
