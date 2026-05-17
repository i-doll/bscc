use crate::{Registry, Report, detect};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub struct WalkOptions {
    pub threads: usize,
    pub follow_links: bool,
    pub respect_gitignore: bool,
    pub include_hidden: bool,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            threads: 0,
            follow_links: false,
            respect_gitignore: true,
            include_hidden: false,
        }
    }
}

/// Walk `roots` and produce a `Report`. Files whose language is not in the
/// registry are silently skipped.
pub fn walk<P: AsRef<Path>>(roots: &[P], registry: &Registry, opts: &WalkOptions) -> Report {
    let mut files: Vec<PathBuf> = Vec::new();
    if roots.is_empty() {
        return Report::default();
    }

    let mut builder = WalkBuilder::new(&roots[0]);
    for r in &roots[1..] {
        builder.add(r);
    }
    builder
        .follow_links(opts.follow_links)
        .git_ignore(opts.respect_gitignore)
        .git_global(opts.respect_gitignore)
        .git_exclude(opts.respect_gitignore)
        .hidden(!opts.include_hidden);
    if opts.threads > 0 {
        builder.threads(opts.threads);
    }

    for entry in builder.build().flatten() {
        if entry.file_type().is_some_and(|t| t.is_file()) {
            files.push(entry.into_path());
        }
    }

    let metrics: Vec<_> = files
        .par_iter()
        .filter_map(|path| analyze_file(registry, path))
        .collect();

    let mut report = Report::default();
    for m in metrics {
        report.push(m);
    }
    report
}

fn analyze_file(registry: &Registry, path: &Path) -> Option<crate::FileMetrics> {
    let entry = detect::detect(registry, path)?;
    let bytes = std::fs::read(path).ok()?;
    Some(entry.analyzer.analyze(path, &bytes))
}
