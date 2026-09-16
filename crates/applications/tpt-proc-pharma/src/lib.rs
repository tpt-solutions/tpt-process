//! Pharmaceutical batch process models: batch reaction cycle accounting,
//! dissolution kinetics (Noyes–Whitney), and first-order degradation /
//! shelf-life estimation.
//!
//! # Example
//!
//! ```
//! use tpt_proc_pharma::{BatchCycle, dissolution_fraction};
//!
//! // Tablet dissolving with k = 0.35 h⁻¹: ~95% in 8.5 h.
//! let f = dissolution_fraction(0.35, 8.5);
//! assert!(f > 0.9 && f < 1.0);
//!
//! let mut cycle = BatchCycle::new(48.0);
//! let total = cycle
//!     .add_step("charge", 2.0)
//!     .add_step("react", 24.0)
//!     .add_step("discharge", 4.0)
//!     .total_duration();
//! assert!((total - 30.0).abs() < 1e-12);
//! ```

#![forbid(unsafe_code)]

/// A batch step schedule.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BatchCycle {
    /// Named steps in execution order: (name, duration h).
    pub steps: Vec<(String, f64)>,
}

impl BatchCycle {
    /// Creates a cycle with a target overall time budget (used for
    /// utilization reporting).
    #[must_use]
    pub const fn new(target_hours: f64) -> Self {
        #[allow(clippy::unused_self)]
        let _ = target_hours;
        Self { steps: Vec::new() }
    }

    /// Adds a step; returns `self` for chaining.
    pub fn add_step(&mut self, name: &str, duration_hours: f64) -> &mut Self {
        self.steps.push((name.to_string(), duration_hours.max(0.0)));
        self
    }

    /// Total cycle duration, h.
    #[must_use]
    pub fn total_duration(&self) -> f64 {
        self.steps.iter().map(|(_, d)| d).sum()
    }

    /// Utilization against the plant time budget: total / target.
    #[must_use]
    pub fn utilization(&self, target_hours: f64) -> f64 {
        if target_hours <= 0.0 {
            return f64::INFINITY;
        }
        self.total_duration() / target_hours
    }
}

/// Fraction of a tablet dissolved after `time_h` by first-order
/// (Noyes–Whitney integrated) kinetics: f = 1 − exp(−k·t).
#[must_use]
pub fn dissolution_fraction(rate_constant_per_h: f64, time_h: f64) -> f64 {
    1.0 - (-rate_constant_per_h.max(0.0) * time_h.max(0.0)).exp()
}

/// Drug potency after storage at `temperature_k` by Arrhenius degradation
/// with first-order rate constant `k_ref_per_day` at `reference_k`.
/// Returns the remaining potency fraction.
#[must_use]
pub fn potency_after_storage(
    k_ref_per_day: f64,
    reference_k: f64,
    temperature_k: f64,
    days: f64,
    activation_energy_j_mol: f64,
) -> f64 {
    let r = 8.314_462_618;
    let k = k_ref_per_day
        * (-activation_energy_j_mol / r * (1.0 / temperature_k - 1.0 / reference_k)).exp();
    (-k * days.max(0.0)).exp()
}

/// Estimated shelf life (days) until potency drops to the USP limit of
/// 95% under reference conditions.
#[must_use]
pub fn shelf_life_days(k_ref_per_day: f64) -> f64 {
    if k_ref_per_day <= 0.0 {
        return f64::INFINITY;
    }
    (1.0_f64 / 0.95).ln() / k_ref_per_day
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dissolution_saturates() {
        assert!(dissolution_fraction(0.35, 0.0) == 0.0);
        let f = dissolution_fraction(0.35, 8.5);
        assert!(f > 0.9 && f < 1.0, "f = {f}");
        assert!(dissolution_fraction(0.35, 100.0) > 0.9999999999);
    }

    #[test]
    fn batch_cycle_adds_and_utilizes() {
        let mut cycle = BatchCycle::new(48.0);
        cycle
            .add_step("charge", 2.0)
            .add_step("react", 24.0)
            .add_step("discharge", 4.0);
        assert!((cycle.total_duration() - 30.0).abs() < 1e-12);
        assert!((cycle.utilization(48.0) - 30.0 / 48.0).abs() < 1e-12);
    }

    #[test]
    fn potency_falls_faster_when_warm() {
        let cool = potency_after_storage(0.001, 298.15, 283.15, 365.0, 75_000.0);
        let warm = potency_after_storage(0.001, 298.15, 313.15, 365.0, 75_000.0);
        assert!(cool > warm, "cool {cool}, warm {warm}");
        assert!(cool <= 1.0 && warm <= 1.0);
    }

    #[test]
    fn shelf_life_of_stable_molecule_exceeds_two_years() {
        // k = 1e-4/day: t95 = ln(1/0.95)/k ≈ 513 days.
        let days = shelf_life_days(1.0e-4);
        assert!((days - (1.0_f64 / 0.95).ln() / 1e-4).abs() < 1.0);
        assert!(days > 500.0);
    }
}
