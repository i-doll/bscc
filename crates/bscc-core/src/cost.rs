//! Output types for COCOMO + AI-assisted cost estimation. The
//! computation lives in `bscc-cost`; these are the data shapes
//! exporters consume and the `Report` carries.
//!
//! Mirrors how `GitMetrics` is owned here and produced by `bscc-git`.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq)]
pub struct Estimate {
    pub effort_months: f64,
    pub schedule_months: f64,
    pub people: f64,
    pub cost_usd: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProjectEstimate {
    pub baseline: Estimate,
    pub ai_assisted: Estimate,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CostReport {
    pub project: ProjectEstimate,
    pub per_language: BTreeMap<String, Estimate>,
    pub params: CostParamsSnapshot,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CostParamsSnapshot {
    pub avg_wage: u32,
    pub overhead: f64,
    pub project_type: String,
    pub ai_multiplier: f64,
}
