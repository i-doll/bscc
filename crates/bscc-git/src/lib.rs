//! Git enrichment for `bscc` reports. v1 shells out to `git log`; a pure-Rust
//! `gix`-based path is a worthwhile follow-up but the surface area required
//! (commit walking + tree diffs) outweighs the local-tool benefit for v0.1.

use bscc_core::{GitMetrics, Report};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("not a git repository: {0}")]
    NotARepo(PathBuf),
    #[error("git command failed: {0}")]
    Command(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct GitOptions {
    pub window_days: u32,
}

impl Default for GitOptions {
    fn default() -> Self {
        Self { window_days: 90 }
    }
}

/// `true` if `path` is inside a git repository.
pub fn is_repo(path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output()
        .ok()
        .is_some_and(|o| o.status.success())
}

/// Locate the repo root containing `path`, or `Err(NotARepo)` if there isn't
/// one. Returned path is absolute.
pub fn repo_root(path: &Path) -> Result<PathBuf, GitError> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()?;
    if !out.status.success() {
        return Err(GitError::NotARepo(path.to_path_buf()));
    }
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(PathBuf::from(line))
}

#[derive(Default)]
struct Accum {
    changes: u32,
    authors: HashSet<String>,
    last_modified: i64,
}

/// Walk `git log` in `repo_root` and produce per-file metrics for the window.
pub fn analyze(
    repo_root: &Path,
    opts: &GitOptions,
) -> Result<HashMap<PathBuf, Accumulated>, GitError> {
    let out = Command::new("git")
        .args([
            "log",
            &format!("--since={} days ago", opts.window_days),
            "--name-only",
            "--no-renames",
            // Header line: <hash>\0<email>\0<timestamp>
            "--pretty=format:%H%x00%ae%x00%at",
        ])
        .current_dir(repo_root)
        .output()?;
    if !out.status.success() {
        return Err(GitError::Command(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }

    let text = String::from_utf8_lossy(&out.stdout);
    let mut accum: HashMap<PathBuf, Accum> = HashMap::new();
    let mut author = String::new();
    let mut time: i64 = 0;

    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        if line.contains('\0') {
            // Header
            let parts: Vec<&str> = line.split('\0').collect();
            if parts.len() == 3 {
                author = parts[1].to_string();
                time = parts[2].parse().unwrap_or(0);
            }
            continue;
        }
        // Path
        let entry = accum.entry(PathBuf::from(line)).or_default();
        entry.changes += 1;
        entry.authors.insert(author.clone());
        if time > entry.last_modified {
            entry.last_modified = time;
        }
    }

    Ok(accum
        .into_iter()
        .map(|(p, a)| {
            (
                p,
                Accumulated {
                    changes_in_window: a.changes,
                    authors_count: u32::try_from(a.authors.len()).unwrap_or(u32::MAX),
                    last_modified: if a.last_modified > 0 {
                        Some(a.last_modified)
                    } else {
                        None
                    },
                },
            )
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct Accumulated {
    pub changes_in_window: u32,
    pub authors_count: u32,
    pub last_modified: Option<i64>,
}

/// Attach `GitMetrics` to every file in the report that has a matching path
/// in the per-file analysis. `hotspot_score` = `complexity × ln(1 + changes)`,
/// where complexity is `cyclomatic_total` if present else `code` LOC.
pub fn enrich(report: &mut Report, repo_root: &Path, opts: &GitOptions) -> Result<(), GitError> {
    let info = analyze(repo_root, opts)?;
    let canonical_root = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());
    for f in &mut report.files {
        let abs = f.path.canonicalize().unwrap_or_else(|_| f.path.clone());
        let rel: PathBuf = abs
            .strip_prefix(&canonical_root)
            .unwrap_or(&abs)
            .to_path_buf();
        if let Some(a) = info.get(&rel) {
            let complexity = f
                .cyclomatic_total
                .map_or_else(|| f64::from(f.code), f64::from);
            let hotspot_score = complexity * (1.0 + f64::from(a.changes_in_window)).ln();
            f.git = Some(GitMetrics {
                changes_in_window: a.changes_in_window,
                authors_count: a.authors_count,
                last_modified: a.last_modified,
                hotspot_score,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn run(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .expect("git available");
        assert!(status.success(), "git {args:?} failed in {dir:?}");
    }

    fn init_repo(dir: &Path) {
        run(dir, &["init", "--initial-branch=main"]);
        run(dir, &["config", "user.email", "test@example.com"]);
        run(dir, &["config", "user.name", "Test"]);
        run(dir, &["config", "commit.gpgsign", "false"]);
    }

    #[test]
    fn enriches_files_with_change_count() {
        let dir = TempDir::new().unwrap();
        init_repo(dir.path());

        let hot = dir.path().join("hot.txt");
        let cold = dir.path().join("cold.txt");
        fs::write(&hot, "v1\n").unwrap();
        fs::write(&cold, "stable\n").unwrap();
        run(dir.path(), &["add", "hot.txt", "cold.txt"]);
        run(dir.path(), &["commit", "-m", "init"]);

        for v in 2..=4 {
            fs::write(&hot, format!("v{v}\n")).unwrap();
            run(dir.path(), &["add", "hot.txt"]);
            run(dir.path(), &["commit", "-m", &format!("v{v}")]);
        }

        let info = analyze(dir.path(), &GitOptions::default()).unwrap();
        let hot_info = info.get(Path::new("hot.txt")).expect("hot.txt tracked");
        let cold_info = info.get(Path::new("cold.txt")).expect("cold.txt tracked");
        assert_eq!(hot_info.changes_in_window, 4, "4 commits touched hot.txt");
        assert_eq!(cold_info.changes_in_window, 1, "1 commit touched cold.txt");
    }
}
