use anyhow::Result;
use bscc_core::{Exporter, Registry, WalkOptions, walk};
use bscc_export::TableExporter;
use clap::Args;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct CountArgs {
    /// One or more paths to walk. Defaults to the current directory.
    pub paths: Vec<PathBuf>,
    /// Output format. For M1: `table` only. JSON/CSV/SARIF/HTML come in later milestones.
    #[arg(long, default_value = "table")]
    pub format: String,
    /// Disable .gitignore handling.
    #[arg(long)]
    pub no_gitignore: bool,
    /// Include hidden files.
    #[arg(long)]
    pub hidden: bool,
    /// Worker threads. 0 = auto.
    #[arg(short = 'j', long, default_value_t = 0)]
    pub threads: usize,
}

pub fn run(registry: &Registry, args: CountArgs) -> Result<()> {
    let roots: Vec<PathBuf> = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths
    };

    let opts = WalkOptions {
        threads: args.threads,
        follow_links: false,
        respect_gitignore: !args.no_gitignore,
        include_hidden: args.hidden,
    };

    let report = walk(&roots, registry, &opts);

    let stdout = io::stdout();
    let mut sink = stdout.lock();
    match args.format.as_str() {
        "table" => TableExporter.write(&report, &mut sink)?,
        other => {
            anyhow::bail!("unknown format {other:?}; only 'table' is supported in M1")
        }
    }
    sink.flush()?;
    Ok(())
}
