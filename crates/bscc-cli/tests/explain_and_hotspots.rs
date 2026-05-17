//! M4 integration: `explain` per-function output + `hotspots` ranking in a
//! tempdir git repo.

use std::path::{Path, PathBuf};
use std::process::Command;

fn build_registry() -> bscc_core::Registry {
    let mut r = bscc_core::Registry::new();
    bscc_regex_tier::register(&mut r);
    bscc_lang_rust::register(&mut r);
    r
}

fn run_git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git is available");
    assert!(status.success(), "git {args:?} failed");
}

#[test]
fn explain_returns_per_function_breakdown_for_tree_sitter_languages() {
    let registry = build_registry();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("rust")
        .join("lib.rs");
    let entry = registry.lookup_by_path(&fixture).expect("rust registered");
    let source = std::fs::read(&fixture).unwrap();
    let details = entry
        .analyzer
        .explain(&fixture, &source)
        .expect("tree-sitter tier supports explain");
    assert_eq!(details.len(), 4, "fixture has 4 functions");
    let max_cc = details.iter().map(|d| d.cyclomatic).max().unwrap();
    assert_eq!(max_cc, 5, "classify() has CC 5");
}

#[test]
fn explain_returns_none_for_regex_tier_languages() {
    let registry = build_registry();
    let lsl = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("sample")
        .join("hello.lsl");
    let entry = registry.lookup_by_path(&lsl).expect("lsl registered");
    let source = std::fs::read(&lsl).unwrap();
    assert!(entry.analyzer.explain(&lsl, &source).is_none());
}

#[test]
fn hotspots_enrichment_captures_per_file_churn_and_authors() {
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    run_git(dir.path(), &["init", "--initial-branch=main"]);
    run_git(dir.path(), &["config", "user.email", "t@e.x"]);
    run_git(dir.path(), &["config", "user.name", "T"]);
    run_git(dir.path(), &["config", "commit.gpgsign", "false"]);

    // Equal-complexity files; only churn varies, so the hotspot score
    // difference is fully attributable to git enrichment.
    let hot = dir.path().join("hot.rs");
    let cold = dir.path().join("cold.rs");
    std::fs::write(&hot, "fn a() { let x = 1; }\n").unwrap();
    std::fs::write(&cold, "fn b() { let y = 1; }\n").unwrap();
    run_git(dir.path(), &["add", "."]);
    run_git(dir.path(), &["commit", "-m", "init"]);

    // Churn `hot` 5 more times.
    for i in 0..5 {
        std::fs::write(&hot, format!("fn a() {{ let x = {i}; }}\n")).unwrap();
        run_git(dir.path(), &["add", "hot.rs"]);
        run_git(dir.path(), &["commit", "-m", &format!("v{i}")]);
    }

    let registry = build_registry();
    let mut report = bscc_core::walk(
        &[dir.path()],
        &registry,
        &bscc_core::WalkOptions {
            respect_gitignore: false,
            ..Default::default()
        },
    );
    bscc_git::enrich(&mut report, dir.path(), &bscc_git::GitOptions::default()).unwrap();

    let hot_git = report
        .files
        .iter()
        .find(|f| f.path.file_name().is_some_and(|n| n == "hot.rs"))
        .and_then(|f| f.git.as_ref())
        .expect("hot.rs got git enrichment");
    let cold_git = report
        .files
        .iter()
        .find(|f| f.path.file_name().is_some_and(|n| n == "cold.rs"))
        .and_then(|f| f.git.as_ref())
        .expect("cold.rs got git enrichment");

    assert_eq!(hot_git.changes_in_window, 6, "init + 5 updates");
    assert_eq!(cold_git.changes_in_window, 1, "init only");
    assert_eq!(hot_git.authors_count, 1);
    assert!(
        hot_git.hotspot_score > cold_git.hotspot_score,
        "with equal complexity, more churn must outrank (hot={}, cold={})",
        hot_git.hotspot_score,
        cold_git.hotspot_score
    );
}

#[test]
fn repo_root_errors_for_non_repo_dir() {
    let dir = tempfile::TempDir::new().unwrap();
    let res = bscc_git::repo_root(dir.path());
    assert!(res.is_err(), "expected NotARepo, got {res:?}");
}
