use anyhow::{Result, bail};
use bscc_core::Registry;
use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ExplainArgs {
    /// File to analyze. Must be a language with a tree-sitter plugin.
    pub file: PathBuf,
}

pub fn run(registry: &Registry, args: &ExplainArgs) -> Result<()> {
    let entry = registry
        .lookup_by_path(&args.file)
        .ok_or_else(|| anyhow::anyhow!("no registered language for {}", args.file.display()))?;
    let source = std::fs::read(&args.file)?;
    let Some(details) = entry.analyzer.explain(&args.file, &source) else {
        bail!(
            "explain requires a tree-sitter-tier language; {} is registered at {:?} tier",
            entry.name,
            entry.tier
        );
    };

    println!("{}  ({})", args.file.display(), entry.name);
    println!("{}", "-".repeat(50));
    println!("{:>6}  {:>6}  {:>4}  lines", "start", "end", "cc");
    for d in &details {
        println!(
            "{:>6}  {:>6}  {:>4}  {}",
            d.start_line, d.end_line, d.cyclomatic, d.lines
        );
    }
    if details.is_empty() {
        println!("(no functions detected)");
    } else {
        let total: u32 = details.iter().map(|d| d.cyclomatic).sum();
        let max = details.iter().map(|d| d.cyclomatic).max().unwrap_or(0);
        println!("{}", "-".repeat(50));
        println!(
            "functions={}  cyclomatic_total={}  cyclomatic_max={}",
            details.len(),
            total,
            max
        );
    }
    Ok(())
}
