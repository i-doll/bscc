//! `bscc.toml` loader. Searches from the start path upward for a `bscc.toml`
//! file; returns `None` if none found (defaults apply).

use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub thresholds: Thresholds,
    #[serde(default)]
    pub git: GitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Thresholds {
    #[serde(default = "default_cyclo")]
    pub cyclomatic_max: u32,
    #[serde(default = "default_len")]
    pub longest_function_lines: u32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            cyclomatic_max: default_cyclo(),
            longest_function_lines: default_len(),
        }
    }
}

fn default_cyclo() -> u32 {
    10
}
fn default_len() -> u32 {
    100
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_window")]
    pub window_days: u32,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            window_days: default_window(),
        }
    }
}

fn default_window() -> u32 {
    90
}

/// Walk upward from `start` looking for `bscc.toml`. Returns `(path, parsed)`
/// or `None` if none exists.
pub fn load(start: &Path) -> Option<(PathBuf, Config)> {
    let mut dir = start.canonicalize().ok()?;
    loop {
        let candidate = dir.join("bscc.toml");
        if candidate.is_file() {
            let text = std::fs::read_to_string(&candidate).ok()?;
            let cfg = toml::from_str(&text).ok()?;
            return Some((candidate, cfg));
        }
        if !dir.pop() {
            return None;
        }
    }
}
