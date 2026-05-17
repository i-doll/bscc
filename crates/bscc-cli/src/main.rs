use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd;
mod config;

#[derive(Parser)]
#[command(name = "bscc", version, about = "Better scc: a code-metrics tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Count lines of code per language.
    Count(cmd::count::CountArgs),
    /// Rank files by hotspot score (complexity × log(churn)).
    Hotspots(cmd::hotspots::HotspotsArgs),
    /// Per-function breakdown for one file (tree-sitter tier only).
    Explain(cmd::explain::ExplainArgs),
    /// List registered languages and which tier they use.
    Languages,
    /// Run the bscc LSP server over stdio (alias for the `bscc-lsp` binary).
    Lsp,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let registry = build_registry();
    let cfg = config::load(std::path::Path::new("."))
        .map(|(_, c)| c)
        .unwrap_or_default();
    match cli.command {
        Command::Count(args) => cmd::count::run(&registry, args, &cfg),
        Command::Hotspots(args) => cmd::hotspots::run(&registry, args, &cfg),
        Command::Explain(args) => cmd::explain::run(&registry, &args),
        Command::Languages => cmd::languages::run(&registry),
        Command::Lsp => cmd::lsp::run(&registry, &cfg),
    }
}

fn build_registry() -> bscc_core::Registry {
    let mut r = bscc_core::Registry::new();
    // Regex tier first; tree-sitter language crates register after so they
    // override the regex-tier entry for their extensions.
    bscc_regex_tier::register(&mut r);
    bscc_lang_rust::register(&mut r);
    bscc_lang_python::register(&mut r);
    bscc_lang_typescript::register(&mut r);
    bscc_lang_go::register(&mut r);
    bscc_lang_c::register(&mut r);
    bscc_lang_cpp::register(&mut r);
    bscc_lang_java::register(&mut r);
    bscc_lang_lsl::register(&mut r);
    bscc_lang_hcl::register(&mut r);
    r
}
