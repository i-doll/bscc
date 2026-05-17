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
    #[serde(default)]
    pub cost: CostConfig,
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

#[derive(Debug, Clone, Deserialize)]
pub struct CostConfig {
    #[serde(default = "default_cost_enable")]
    pub enable: bool,
    #[serde(default = "default_avg_wage")]
    pub avg_wage: u32,
    #[serde(default = "default_overhead")]
    pub overhead: f64,
    #[serde(default = "default_project_type")]
    pub project_type: String,
    #[serde(default = "default_ai_multiplier")]
    pub ai_multiplier: f64,
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            enable: default_cost_enable(),
            avg_wage: default_avg_wage(),
            overhead: default_overhead(),
            project_type: default_project_type(),
            ai_multiplier: default_ai_multiplier(),
        }
    }
}

fn default_cost_enable() -> bool {
    true
}
fn default_avg_wage() -> u32 {
    bscc_cost::defaults::AVG_WAGE
}
fn default_overhead() -> f64 {
    bscc_cost::defaults::OVERHEAD
}
fn default_project_type() -> String {
    bscc_cost::defaults::PROJECT_TYPE.into()
}
fn default_ai_multiplier() -> f64 {
    bscc_cost::defaults::AI_MULTIPLIER
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
