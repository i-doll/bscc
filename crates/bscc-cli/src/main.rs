use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd;

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
    /// List registered languages and which tier they use.
    Languages,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let registry = build_registry();
    match cli.command {
        Command::Count(args) => cmd::count::run(&registry, args),
        Command::Languages => cmd::languages::run(&registry),
    }
}

fn build_registry() -> bscc_core::Registry {
    let mut r = bscc_core::Registry::new();
    // Regex tier first; tree-sitter language crates register after so they
    // override the regex-tier entry for their extensions.
    bscc_regex_tier::register(&mut r);
    bscc_lang_rust::register(&mut r);
    r
}
