//! HCL / Terraform / Packer / `OpenTofu` each get their own bucket in
//! the report, and the JSON-syntax variants stay separated so the HCL2
//! tree-sitter analyzer never receives a `.tf.json` to choke on.

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
    r
}

#[test]
fn hashicorp_languages_split_into_buckets() {
    let registry = build_registry();
    let report = bscc_core::walk(
        &[&fixtures_root()],
        &registry,
        &bscc_core::WalkOptions {
            respect_gitignore: false,
            ..Default::default()
        },
    );
    let totals = report.by_language();

    let terraform = totals.get("Terraform").expect("Terraform bucket present");
    assert_eq!(
        terraform.files, 3,
        "main.tf + variables.tfvars + complexity_terraform.tf"
    );

    let hcl = totals.get("HCL").expect("HCL bucket present");
    assert_eq!(hcl.files, 2, "config.hcl + complexity.hcl");

    let packer = totals.get("Packer").expect("Packer bucket present");
    assert_eq!(packer.files, 2, "builder.pkr.hcl + complexity_packer.pkr.hcl");

    let tofu = totals.get("OpenTofu").expect("OpenTofu bucket present");
    assert_eq!(tofu.files, 1, "complexity.tofu");

    let tfjson = totals
        .get("Terraform JSON")
        .expect("Terraform JSON bucket present");
    assert_eq!(tfjson.files, 1, "legacy.tf.json");
}

#[test]
fn packer_extension_routes_to_packer_not_hcl() {
    let registry = build_registry();
    let entry = registry
        .lookup_by_path(std::path::Path::new("anywhere/foo.pkr.hcl"))
        .expect("pkr.hcl resolves");
    assert_eq!(entry.name, "Packer");
}

#[test]
fn terraform_json_routes_to_terraform_json() {
    let registry = build_registry();
    let entry = registry
        .lookup_by_path(std::path::Path::new("anywhere/infra.tf.json"))
        .expect("tf.json resolves");
    assert_eq!(entry.name, "Terraform JSON");
}

#[test]
fn packer_json_routes_to_packer_json() {
    let registry = build_registry();
    let entry = registry
        .lookup_by_path(std::path::Path::new("anywhere/builder.pkr.json"))
        .expect("pkr.json resolves");
    assert_eq!(entry.name, "Packer JSON");
}
