//! Verifies that the walker honors .gitignore by default and that
//! `--no-gitignore` reaches every file.

use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn init_repo(dir: &Path) {
    // `ignore` crate respects .gitignore in non-repo trees too, but a real
    // `git init` exercises the same path users will hit.
    let s = Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(dir)
        .status()
        .expect("git available");
    assert!(s.success());
}

fn build_registry() -> bscc_core::Registry {
    let mut r = bscc_core::Registry::new();
    bscc_regex_tier::register(&mut r);
    bscc_lang_rust::register(&mut r);
    r
}

fn walk(dir: &Path, respect: bool) -> Vec<String> {
    let report = bscc_core::walk(
        &[dir],
        &build_registry(),
        &bscc_core::WalkOptions {
            respect_gitignore: respect,
            ..Default::default()
        },
    );
    report
        .files
        .iter()
        .map(|f| f.path.file_name().unwrap().to_string_lossy().to_string())
        .collect()
}

#[test]
fn default_walk_skips_gitignored_files_and_directories() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());
    std::fs::write(dir.path().join(".gitignore"), "ignored.rs\nbuild/\n").unwrap();
    std::fs::write(dir.path().join("kept.rs"), "fn a() {}\n").unwrap();
    std::fs::write(dir.path().join("ignored.rs"), "fn b() {}\n").unwrap();
    std::fs::create_dir(dir.path().join("build")).unwrap();
    std::fs::write(dir.path().join("build/inside.rs"), "fn c() {}\n").unwrap();

    let names = walk(dir.path(), true);
    assert!(
        names.contains(&"kept.rs".to_string()),
        "kept.rs walked: {names:?}"
    );
    assert!(
        !names.contains(&"ignored.rs".to_string()),
        "ignored.rs excluded: {names:?}"
    );
    assert!(
        !names.contains(&"inside.rs".to_string()),
        "files inside ignored build/ excluded: {names:?}"
    );
}

#[test]
fn no_gitignore_option_walks_everything() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());
    std::fs::write(dir.path().join(".gitignore"), "secret.rs\n").unwrap();
    std::fs::write(dir.path().join("public.rs"), "fn a() {}\n").unwrap();
    std::fs::write(dir.path().join("secret.rs"), "fn b() {}\n").unwrap();

    let with_ignore = walk(dir.path(), true);
    let without_ignore = walk(dir.path(), false);

    assert!(
        !with_ignore.contains(&"secret.rs".to_string()),
        "respect_gitignore=true should hide secret.rs"
    );
    assert!(
        without_ignore.contains(&"secret.rs".to_string()),
        "respect_gitignore=false should reveal secret.rs"
    );
    assert!(with_ignore.contains(&"public.rs".to_string()));
}
