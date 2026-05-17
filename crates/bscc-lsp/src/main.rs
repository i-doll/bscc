fn main() -> anyhow::Result<()> {
    let mut registry = bscc_core::Registry::new();
    bscc_regex_tier::register(&mut registry);
    bscc_lang_rust::register(&mut registry);
    bscc_lang_python::register(&mut registry);
    bscc_lang_typescript::register(&mut registry);
    bscc_lang_go::register(&mut registry);
    bscc_lang_c::register(&mut registry);
    bscc_lang_cpp::register(&mut registry);
    bscc_lang_java::register(&mut registry);
    bscc_lsp::run(registry, bscc_lsp::Thresholds::default())
}
