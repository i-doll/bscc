//! Pure COCOMO formula module.
//!
//! Basic COCOMO (Boehm '81):
//!   Effort   = a × KLOC^b           (person-months)
//!   Schedule = c × Effort^d         (calendar months)
//!   People   = Effort / Schedule    (developers)
//!   Cost     = Effort × `monthly_salary` × overhead

use crate::CostParams;
use bscc_core::Estimate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Organic,
    SemiDetached,
    Embedded,
}

impl ProjectType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Organic => "organic",
            Self::SemiDetached => "semi_detached",
            Self::Embedded => "embedded",
        }
    }

    /// (a, b) — effort coefficients.
    fn effort_coeffs(self) -> (f64, f64) {
        match self {
            Self::Organic => (2.4, 1.05),
            Self::SemiDetached => (3.0, 1.12),
            Self::Embedded => (3.6, 1.20),
        }
    }

    /// (c, d) — schedule coefficients.
    fn schedule_coeffs(self) -> (f64, f64) {
        match self {
            Self::Organic => (2.5, 0.38),
            Self::SemiDetached => (2.5, 0.35),
            Self::Embedded => (2.5, 0.32),
        }
    }
}

pub fn baseline(code_lines: u32, params: &CostParams) -> Estimate {
    if code_lines == 0 {
        return Estimate::default();
    }
    let kloc = f64::from(code_lines) / 1000.0;
    let (a, b) = params.project_type.effort_coeffs();
    let effort = a * kloc.powf(b);
    estimate_from_effort(effort, params)
}

pub fn ai_assisted(code_lines: u32, params: &CostParams) -> Estimate {
    let base = baseline(code_lines, params);
    // Guard against zero/negative multipliers (config typos, intentional
    // disable). 1.0 is a no-op; anything ≤0 is invalid and we treat as 1.
    if params.ai_multiplier <= 0.0 || (params.ai_multiplier - 1.0).abs() < f64::EPSILON {
        return base;
    }
    let effort = base.effort_months / params.ai_multiplier;
    estimate_from_effort(effort, params)
}

fn estimate_from_effort(effort: f64, params: &CostParams) -> Estimate {
    if effort <= 0.0 {
        return Estimate::default();
    }
    let (c, d) = params.project_type.schedule_coeffs();
    let schedule = c * effort.powf(d);
    let people = if schedule > 0.0 { effort / schedule } else { 0.0 };
    let monthly = f64::from(params.avg_wage) / 12.0;
    let cost = effort * monthly * params.overhead;
    Estimate {
        effort_months: effort,
        schedule_months: schedule,
        people,
        cost_usd: saturating_cost_to_u64(cost),
    }
}

/// Saturating f64 -> u64 conversion. NaN -> 0; negative -> 0; values above
/// `u64::MAX` clamp to `u64::MAX`. Real-world costs never approach this
/// bound, but the cast lints if we don't make the saturation explicit.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn saturating_cost_to_u64(cost: f64) -> u64 {
    if !cost.is_finite() || cost <= 0.0 {
        0
    } else if cost >= 1.844_674_407_370_955_2e19_f64 {
        // 2^64 expressed as the f64 we can compare against without losing
        // precision in the comparison itself.
        u64::MAX
    } else {
        cost as u64
    }
}
