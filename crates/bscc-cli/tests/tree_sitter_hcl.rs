//! L6 integration test: bscc-lang-hcl promotes all four HCL2 dialects
//! to the tree-sitter tier and shares one analyzer across them. JSON
//! variants stay at the regex tier under their own dialect names.

use std::path::PathBuf;

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("hcl")
}

fn build_registry() -> bscc_core::Registry {
    let mut r = bscc_core::Registry::new();
    bscc_regex_tier::register(&mut r);
    bscc_lang_hcl::register(&mut r);
    r
}

fn metrics_for(filename: &str) -> bscc_core::FileMetrics {
    let registry = build_registry();
    let report = bscc_core::walk(
        &[&fixtures_root()],
        &registry,
        &bscc_core::WalkOptions {
            respect_gitignore: false,
            ..Default::default()
        },
    );
    report
        .files
        .iter()
        .find(|f| f.path.file_name().is_some_and(|n| n == filename))
        .unwrap_or_else(|| panic!("{filename} walked"))
        .clone()
}

#[test]
fn terraform_file_gets_tree_sitter_metrics() {
    let f = metrics_for("complexity_terraform.tf");

    assert_eq!(f.language, "Terraform");
    // 5 top-level blocks: variable, locals, two resources, output.
    assert_eq!(f.functions, Some(5), "top-level blocks");
    // Largest scope is resource "web": 1 for_each + 1 dynamic + 1 inner
    // for_each + 1 conditional + 1 && = 5 branches -> CC 6.
    assert_eq!(f.cyclomatic_max, Some(6), "cyclomatic_max");
    // variable=1, locals=3 (for_expr + conditional), data=2 (count),
    // web=6, output=3 (conditional + ||). Sum = 15.
    assert_eq!(f.cyclomatic_total, Some(15), "cyclomatic_total");
    assert_eq!(f.todo_comments, Some(1), "TODO counted");
    assert!(f.longest_function_lines.is_some_and(|n| n > 0));
}

#[test]
fn packer_file_gets_tree_sitter_metrics() {
    let f = metrics_for("complexity_packer.pkr.hcl");

    assert_eq!(f.language, "Packer");
    // 2 top-level blocks: source + build.
    assert_eq!(f.functions, Some(2));
    // source: conditional + for_expr in tags = 2 branches -> CC 3.
    assert_eq!(f.cyclomatic_max, Some(3));
    // source=3, build=2 (one inner conditional). Sum = 5.
    assert_eq!(f.cyclomatic_total, Some(5));
    assert_eq!(f.todo_comments, Some(1), "FIXME counted");
}

#[test]
fn hcl_file_gets_tree_sitter_metrics() {
    let f = metrics_for("complexity.hcl");

    assert_eq!(f.language, "HCL");
    // 2 top-level blocks: listener + storage.
    assert_eq!(f.functions, Some(2));
    // listener=3 (conditional + &&), storage=3 (conditional + ||).
    assert_eq!(f.cyclomatic_max, Some(3));
    assert_eq!(f.cyclomatic_total, Some(6));
}

#[test]
fn opentofu_file_gets_tree_sitter_metrics() {
    let f = metrics_for("complexity.tofu");

    assert_eq!(f.language, "OpenTofu");
    // 2 top-level blocks: terraform + variable. No branches.
    assert_eq!(f.functions, Some(2));
    assert_eq!(f.cyclomatic_max, Some(1));
    assert_eq!(f.cyclomatic_total, Some(2));
}

#[test]
fn hcl_family_extensions_registered_at_tree_sitter_tier() {
    let registry = build_registry();
    for (ext, name) in [
        ("hcl", "HCL"),
        ("tf", "Terraform"),
        ("tfvars", "Terraform"),
        ("tofu", "OpenTofu"),
        ("pkr.hcl", "Packer"),
        ("pkrvars.hcl", "Packer"),
    ] {
        let entry = registry
            .lookup_by_extension(ext)
            .unwrap_or_else(|| panic!("{ext} registered"));
        assert_eq!(entry.name, name, "{ext} -> {name}");
        assert_eq!(entry.tier, bscc_core::Tier::TreeSitter, "{ext} tier");
    }
}

#[test]
fn json_variants_stay_regex_tier() {
    let registry = build_registry();
    for (ext, name) in [("tf.json", "Terraform JSON"), ("pkr.json", "Packer JSON")] {
        let entry = registry
            .lookup_by_extension(ext)
            .unwrap_or_else(|| panic!("{ext} registered"));
        assert_eq!(entry.name, name);
        assert_eq!(entry.tier, bscc_core::Tier::Regex);
    }
}

#[test]
fn explain_returns_per_block_breakdown_for_terraform() {
    let registry = build_registry();
    let path = fixtures_root().join("complexity_terraform.tf");
    let source = std::fs::read(&path).unwrap();
    let entry = registry.lookup_by_path(&path).expect("tf registered");
    let details = entry
        .analyzer
        .explain(&path, &source)
        .expect("tree-sitter tier supports explain");
    assert_eq!(details.len(), 5, "5 top-level blocks");
    let max_cc = details.iter().map(|d| d.cyclomatic).max().unwrap();
    assert_eq!(max_cc, 6);
}
