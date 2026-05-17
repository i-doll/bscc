use anyhow::Result;
use bscc_core::{Exporter, Registry, WalkOptions, walk};
use bscc_export::{CsvExporter, JsonExporter, SarifExporter, TableExporter};
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
    /// Skip hidden (dotfile) files and directories. Default walks them so
    /// `.github/`, `.vscode/`, etc. show up; VCS dirs (`.git`, `.hg`, ...)
    /// are always skipped.
    #[arg(long)]
    pub skip_hidden: bool,
    /// Worker threads. 0 = auto.
    #[arg(short = 'j', long, default_value_t = 0)]
    pub threads: usize,
    /// Suppress the COCOMO cost footer + Cost column.
    #[arg(long)]
    pub no_cost: bool,
    /// Override the COCOMO average annual wage (default 56286).
    #[arg(long)]
    pub avg_wage: Option<u32>,
    /// Override the COCOMO overhead multiplier (default 2.4).
    #[arg(long)]
    pub overhead: Option<f64>,
    /// COCOMO project type: organic | `semi_detached` | embedded.
    #[arg(long)]
    pub project_type: Option<String>,
    /// AI productivity multiplier (default 2.0; >1 = faster than baseline).
    #[arg(long)]
    pub ai_multiplier: Option<f64>,
}

pub fn run(registry: &Registry, args: CountArgs, cfg: &crate::config::Config) -> Result<()> {
    let roots: Vec<PathBuf> = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths
    };

    let opts = WalkOptions {
        threads: args.threads,
        follow_links: false,
        respect_gitignore: !args.no_gitignore,
        skip_hidden: args.skip_hidden,
    };

    let mut report = walk(&roots, registry, &opts);

    let cost_enabled = !args.no_cost && cfg.cost.enable;
    if cost_enabled {
        let project_type_str = args
            .project_type
            .as_deref()
            .unwrap_or(&cfg.cost.project_type);
        let project_type: bscc_cost::ProjectType = project_type_str
            .parse()
            .map_err(|e: String| anyhow::anyhow!(e))?;
        let params = bscc_cost::CostParams {
            avg_wage: args.avg_wage.unwrap_or(cfg.cost.avg_wage),
            overhead: args.overhead.unwrap_or(cfg.cost.overhead),
            project_type,
            ai_multiplier: args.ai_multiplier.unwrap_or(cfg.cost.ai_multiplier),
        };
        report.cost = Some(bscc_cost::estimate(&report, &params));
    }

    let stdout = io::stdout();
    let mut sink = stdout.lock();
    match args.format.as_str() {
        "table" => TableExporter.write(&report, &mut sink)?,
        "json" => JsonExporter { pretty: true }.write(&report, &mut sink)?,
        "csv" => CsvExporter.write(&report, &mut sink)?,
        "sarif" => SarifExporter {
            thresholds: bscc_export::SarifThresholds {
                cyclomatic_max: cfg.thresholds.cyclomatic_max,
                longest_function_lines: cfg.thresholds.longest_function_lines,
            },
        }
        .write(&report, &mut sink)?,
        "html" => bscc_export::HtmlExporter.write(&report, &mut sink)?,
        other => {
            anyhow::bail!("unknown format {other:?}; supported: table, json, csv, sarif, html")
        }
    }
    sink.flush()?;
    Ok(())
}
