//! COCOMO + AI-assisted cost estimation for bscc reports.
//!
//! Implements Boehm's basic COCOMO (1981) with three project classes
//! (organic / semi-detached / embedded), plus an "AI-assisted" variant
//! that divides effort by a configurable productivity multiplier and
//! recomputes the dependent quantities. Pure arithmetic — no I/O, no
//! network, no LLM calls.
//!
//! Output types (`Estimate`, `CostReport`, ...) live in `bscc-core` so
//! exporters can consume them without depending on this crate.

mod cocomo;

use bscc_core::{CostParamsSnapshot, CostReport, Estimate, ProjectEstimate, Report};
use std::str::FromStr;

pub use cocomo::ProjectType;

/// User-tunable inputs. Constructed from CLI flags / TOML config.
#[derive(Debug, Clone, Copy)]
pub struct CostParams {
    /// Annual developer salary in the user's currency (treated as USD
    /// in the table footer's `$` symbol; numerically agnostic).
    pub avg_wage: u32,
    /// Multiplier applied to salary to account for benefits, equipment,
    /// office, etc. Standard COCOMO loading factor is `2.4`.
    pub overhead: f64,
    /// COCOMO project class. Most general-purpose codebases are
    /// `Organic`; large in-house systems are `SemiDetached`; firmware
    /// and tightly-coupled embedded code is `Embedded`.
    pub project_type: ProjectType,
    /// Productivity gain attributed to AI tooling. `2.0` means AI users
    /// finish the same work in half the time; `1.0` is a no-op (the
    /// AI-assisted estimate equals the baseline).
    pub ai_multiplier: f64,
}

impl Default for CostParams {
    fn default() -> Self {
        Self {
            avg_wage: defaults::AVG_WAGE,
            overhead: defaults::OVERHEAD,
            project_type: ProjectType::Organic,
            ai_multiplier: defaults::AI_MULTIPLIER,
        }
    }
}

pub mod defaults {
    //! Default values, also re-used by the CLI's serde defaults.
    pub const AVG_WAGE: u32 = 56_286;
    pub const OVERHEAD: f64 = 2.4;
    pub const PROJECT_TYPE: &str = "organic";
    pub const AI_MULTIPLIER: f64 = 2.0;
}

/// Compute the project-wide and per-language cost estimates for a
/// walked report. Per-language estimates apply COCOMO independently to
/// each language's code-LOC; the project estimate uses the grand total.
/// Because COCOMO is non-linear in KLOC, the per-language sum exceeds
/// the project total — both numbers are meaningful but answer different
/// questions.
#[must_use]
pub fn estimate(report: &Report, params: &CostParams) -> CostReport {
    let totals = report.by_language();
    let grand = report.grand_total();

    let project_baseline = cocomo::baseline(grand.code, params);
    let project_ai = cocomo::ai_assisted(grand.code, params);

    let per_language = totals
        .into_iter()
        .map(|(name, t)| (name, cocomo::baseline(t.code, params)))
        .collect();

    CostReport {
        project: ProjectEstimate {
            baseline: project_baseline,
            ai_assisted: project_ai,
        },
        per_language,
        params: snapshot(params),
    }
}

/// Convenience: estimate just one KLOC value (e.g. for unit tests or
/// for callers that already have an aggregated total).
#[must_use]
pub fn estimate_loc(code_lines: u32, params: &CostParams) -> Estimate {
    cocomo::baseline(code_lines, params)
}

fn snapshot(p: &CostParams) -> CostParamsSnapshot {
    CostParamsSnapshot {
        avg_wage: p.avg_wage,
        overhead: p.overhead,
        project_type: p.project_type.as_str().into(),
        ai_multiplier: p.ai_multiplier,
    }
}

impl FromStr for ProjectType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "organic" => Ok(Self::Organic),
            "semi_detached" | "semi-detached" | "semidetached" => Ok(Self::SemiDetached),
            "embedded" => Ok(Self::Embedded),
            other => Err(format!(
                "unknown project type {other:?}; expected organic, semi_detached, or embedded"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults_params() -> CostParams {
        CostParams::default()
    }

    #[test]
    fn organic_50_kloc_matches_known_cocomo_output() {
        // Hand-computed reference: 50 KLOC organic.
        // effort = 2.4 * 50^1.05 = 145.93 PM
        // schedule = 2.5 * 145.93^0.38 = 17.13 mo (using 0.38)
        // people = 145.93 / 17.13 = 8.52
        // cost = 145.93 * (56286/12) * 2.4 = 1_642_603 USD
        let p = defaults_params();
        let est = estimate_loc(50_000, &p);
        assert!(
            (est.effort_months - 145.93).abs() < 1.0,
            "effort = {}",
            est.effort_months
        );
        assert!(
            (est.schedule_months - 17.13).abs() < 1.0,
            "schedule = {}",
            est.schedule_months
        );
        assert!(
            (est.people - 8.52).abs() < 0.5,
            "people = {}",
            est.people
        );
        // Cost within 1% of hand-computed value.
        let expected_cost = 1_642_603u64;
        let delta = (i64::try_from(est.cost_usd).unwrap() - i64::try_from(expected_cost).unwrap())
            .unsigned_abs();
        assert!(
            delta < expected_cost / 100,
            "cost = {} (expected ~{expected_cost})",
            est.cost_usd
        );
    }

    #[test]
    fn ai_assisted_halves_effort_at_multiplier_2() {
        let p = defaults_params();
        let baseline = estimate_loc(50_000, &p);
        let ai = cocomo::ai_assisted(50_000, &p);
        assert!(
            (ai.effort_months - baseline.effort_months / 2.0).abs() < 0.01,
            "ai={} baseline={}",
            ai.effort_months,
            baseline.effort_months
        );
        // Cost scales linearly with effort; should be ~half.
        let half_baseline = baseline.cost_usd / 2;
        let diff = ai.cost_usd.max(half_baseline) - ai.cost_usd.min(half_baseline);
        assert!(
            diff <= 1,
            "ai_cost={} baseline_cost={}",
            ai.cost_usd,
            baseline.cost_usd
        );
    }

    #[test]
    fn ai_multiplier_one_equals_baseline() {
        let p = CostParams {
            ai_multiplier: 1.0,
            ..CostParams::default()
        };
        let baseline = estimate_loc(50_000, &p);
        let ai = cocomo::ai_assisted(50_000, &p);
        assert_eq!(ai, baseline);
    }

    #[test]
    fn zero_kloc_yields_all_zeros() {
        let p = defaults_params();
        let est = estimate_loc(0, &p);
        assert_eq!(est, Estimate::default());
    }

    #[test]
    fn project_types_produce_different_effort() {
        let organic = estimate_loc(
            50_000,
            &CostParams {
                project_type: ProjectType::Organic,
                ..Default::default()
            },
        );
        let semi = estimate_loc(
            50_000,
            &CostParams {
                project_type: ProjectType::SemiDetached,
                ..Default::default()
            },
        );
        let embedded = estimate_loc(
            50_000,
            &CostParams {
                project_type: ProjectType::Embedded,
                ..Default::default()
            },
        );
        assert!(organic.effort_months < semi.effort_months);
        assert!(semi.effort_months < embedded.effort_months);
    }

    #[test]
    fn project_type_from_str_accepts_variants() {
        assert_eq!("organic".parse::<ProjectType>().unwrap(), ProjectType::Organic);
        assert_eq!(
            "semi_detached".parse::<ProjectType>().unwrap(),
            ProjectType::SemiDetached
        );
        assert_eq!(
            "semi-detached".parse::<ProjectType>().unwrap(),
            ProjectType::SemiDetached
        );
        assert_eq!(
            "EMBEDDED".parse::<ProjectType>().unwrap(),
            ProjectType::Embedded
        );
        assert!("nonsense".parse::<ProjectType>().is_err());
    }

    #[test]
    fn negative_or_zero_ai_multiplier_defaults_to_baseline() {
        let p = CostParams {
            ai_multiplier: 0.0,
            ..CostParams::default()
        };
        let baseline = estimate_loc(50_000, &p);
        let ai = cocomo::ai_assisted(50_000, &p);
        assert_eq!(ai, baseline, "ai_multiplier=0 should fall back to no-op");
    }
}
