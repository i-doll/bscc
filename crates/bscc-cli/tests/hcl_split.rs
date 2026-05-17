//! HCL / Terraform / Packer should each get their own bucket in the
//! report, and Packer's `.pkr.hcl` must not be misrouted to HCL.

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
fn hashicorp_languages_split_into_three_buckets() {
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
    assert_eq!(terraform.files, 2, "main.tf + variables.tfvars");

    let hcl = totals.get("HCL").expect("HCL bucket present");
    assert_eq!(hcl.files, 1, "config.hcl");

    let packer = totals.get("Packer").expect("Packer bucket present");
    assert_eq!(packer.files, 1, "builder.pkr.hcl");
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
fn terraform_json_routes_to_terraform_not_json() {
    let registry = build_registry();
    let entry = registry
        .lookup_by_path(std::path::Path::new("anywhere/infra.tf.json"))
        .expect("tf.json resolves");
    assert_eq!(entry.name, "Terraform");
}
