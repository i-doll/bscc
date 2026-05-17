//! Smoke test: the production `build_registry` (full set of language crates)
//! must succeed at startup. This catches any `.scm` query that fails to
//! compile against its grammar — those errors would otherwise only surface on
//! a user's first run.

fn build_full_registry() -> bscc_core::Registry {
    let mut r = bscc_core::Registry::new();
    bscc_regex_tier::register(&mut r);
    bscc_lang_rust::register(&mut r);
    bscc_lang_python::register(&mut r);
    bscc_lang_typescript::register(&mut r);
    bscc_lang_go::register(&mut r);
    bscc_lang_c::register(&mut r);
    bscc_lang_cpp::register(&mut r);
    bscc_lang_java::register(&mut r);
    r
}

#[test]
fn full_registry_constructs_without_panic() {
    let r = build_full_registry();
    assert!(r.len() >= 10);
}

#[test]
fn tree_sitter_tier_languages_resolve() {
    let r = build_full_registry();
    for (ext, expected_name) in [
        ("rs", "Rust"),
        ("py", "Python"),
        ("ts", "TypeScript"),
        ("tsx", "TSX"),
        ("go", "Go"),
        ("c", "C"),
        ("cpp", "C++"),
        ("java", "Java"),
    ] {
        let entry = r
            .lookup_by_extension(ext)
            .unwrap_or_else(|| panic!(".{ext} should resolve"));
        assert_eq!(entry.name, expected_name, "name for .{ext}");
        assert_eq!(entry.tier, bscc_core::Tier::TreeSitter, "tier for .{ext}");
    }
}
