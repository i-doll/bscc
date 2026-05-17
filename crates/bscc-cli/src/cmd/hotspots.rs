use anyhow::Result;
use bscc_core::{Registry, WalkOptions, walk};
use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct HotspotsArgs {
    pub paths: Vec<PathBuf>,
    /// Window for `git log` (in days).
    #[arg(long, default_value_t = 90)]
    pub window_days: u32,
    /// Maximum rows to print.
    #[arg(long, default_value_t = 20)]
    pub top: usize,
    #[arg(long)]
    pub no_gitignore: bool,
}

pub fn run(registry: &Registry, args: HotspotsArgs, cfg: &crate::config::Config) -> Result<()> {
    let roots: Vec<PathBuf> = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths
    };

    let mut report = walk(
        &roots,
        registry,
        &WalkOptions {
            respect_gitignore: !args.no_gitignore,
            ..Default::default()
        },
    );

    // Resolve the repo root from the first walked path. If it's not in a
    // repo, fall back to printing files ranked by complexity (or LOC).
    let probe = roots[0].canonicalize().unwrap_or(roots[0].clone());
    // CLI flag wins over config file.
    let window_days = if args.window_days == 90 {
        cfg.git.window_days
    } else {
        args.window_days
    };
    let opts = bscc_git::GitOptions { window_days };
    if let Ok(root) = bscc_git::repo_root(&probe) {
        bscc_git::enrich(&mut report, &root, &opts)?;
    } else {
        eprintln!("note: not inside a git repository — ranking by complexity/LOC only");
    }

    let mut ranked: Vec<_> = report.files.iter().collect();
    ranked.sort_by(|a, b| {
        let sa = a.git.as_ref().map_or_else(
            || f64::from(a.cyclomatic_total.unwrap_or(a.code)),
            |g| g.hotspot_score,
        );
        let sb = b.git.as_ref().map_or_else(
            || f64::from(b.cyclomatic_total.unwrap_or(b.code)),
            |g| g.hotspot_score,
        );
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    println!(
        "{:<60} {:>10} {:>8} {:>8}",
        "File", "Score", "CC(max)", "Changes"
    );
    println!("{}", "-".repeat(60 + 1 + 10 + 1 + 8 + 1 + 8));
    for f in ranked.iter().take(args.top) {
        let score = f.git.as_ref().map_or_else(
            || f64::from(f.cyclomatic_total.unwrap_or(f.code)),
            |g| g.hotspot_score,
        );
        let changes = f.git.as_ref().map_or(0, |g| g.changes_in_window);
        println!(
            "{:<60} {:>10.1} {:>8} {:>8}",
            f.path.display(),
            score,
            f.cyclomatic_max.unwrap_or(0),
            changes
        );
    }
    Ok(())
}
