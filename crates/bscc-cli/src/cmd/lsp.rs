use anyhow::Result;
use bscc_core::Registry;

pub fn run(_registry: &Registry, cfg: &crate::config::Config) -> Result<()> {
    // bscc_lsp::run takes ownership of the Registry, so we build a fresh one
    // here rather than mutating the shared one. The CLI's main registry stays
    // unused for this subcommand.
    let mut r = Registry::new();
    bscc_regex_tier::register(&mut r);
    bscc_lang_rust::register(&mut r);
    bscc_lang_python::register(&mut r);
    bscc_lang_typescript::register(&mut r);
    bscc_lang_go::register(&mut r);
    bscc_lang_c::register(&mut r);
    bscc_lang_cpp::register(&mut r);
    bscc_lang_java::register(&mut r);
    bscc_lsp::run(
        r,
        bscc_lsp::Thresholds {
            cyclomatic_max: cfg.thresholds.cyclomatic_max,
            longest_function_lines: cfg.thresholds.longest_function_lines,
        },
    )
}
