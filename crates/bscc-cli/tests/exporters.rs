//! M3 integration: CSV + SARIF exporter shape.

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

fn report() -> bscc_core::Report {
    bscc_core::walk(
        &[&fixtures_root()],
        &build_registry(),
        &bscc_core::WalkOptions {
            respect_gitignore: false,
            ..Default::default()
        },
    )
}

#[test]
fn csv_has_header_and_one_row_per_file() {
    let mut buf = Vec::new();
    bscc_export::CsvExporter.write(&report(), &mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines
            .first()
            .is_some_and(|h| h.starts_with("path,language,"))
    );
    assert_eq!(lines.len(), 2, "header + one data row, got {lines:?}");
    let data = lines[1];
    assert!(data.contains("Rust"), "row references the language");
    assert!(
        data.contains(",11,5,,,"),
        "tree-sitter fields fill, optional fields stay empty"
    );
}

#[test]
fn sarif_has_no_results_when_thresholds_not_exceeded() {
    // Default thresholds: cyclomatic_max > 10 or longest_function_lines > 100.
    // The Rust fixture's worst function is CC=5, length=11.
    let mut buf = Vec::new();
    bscc_export::SarifExporter::default()
        .write(&report(), &mut buf)
        .unwrap();
    let text = String::from_utf8(buf).unwrap();
    assert!(text.contains("\"results\": []"));
    assert!(text.contains("\"version\": \"2.1.0\""));
}

#[test]
fn sarif_emits_results_when_thresholds_exceeded() {
    // Set thresholds below the fixture's actual numbers so we get results.
    let exp = bscc_export::SarifExporter {
        thresholds: bscc_export::SarifThresholds {
            cyclomatic_max: 2,
            longest_function_lines: 5,
        },
    };
    let mut buf = Vec::new();
    exp.write(&report(), &mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();
    assert!(text.contains("complexity/cyclomatic"));
    assert!(text.contains("size/longest-function"));
    assert!(text.contains("lib.rs"));
}
