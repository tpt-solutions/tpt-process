//! Liquid-liquid extraction: single-stage and multistage crosscurrent
//! contact with a constant distribution coefficient (dilute systems).
//!
//! # Example
//!
//! ```
//! use tpt_proc_extraction::{single_stage, Extractor};
//!
//! // 100 kg feed with 10 kg solute, K = 8, equal solvent mass:
//! let (raffinate_solute, extract_solute) =
//!     single_stage(100.0, 10.0, 100.0, 8.0);
//! assert!((raffinate_solute + extract_solute - 10.0).abs() < 1e-9);
//! assert!(raffinate_solute < extract_solute);
//!
//! // Three crosscurrent stages with a third of the solvent each.
//! let (multi_raf, _) = Extractor::new(3).crosscurrent(100.0, 10.0, 300.0, 8.0);
//! assert!(multi_raf < single_stage(100.0, 10.0, 300.0, 8.0).0);
//! ```

#![forbid(unsafe_code)]

/// A crosscurrent extraction battery.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extractor {
    /// Number of equilibrium stages.
    pub num_stages: u32,
}

impl Extractor {
    /// Creates an extraction battery.
    #[must_use]
    pub const fn new(num_stages: u32) -> Self {
        Self { num_stages }
    }

    /// Multistage crosscurrent extraction: fresh solvent `total_solvent`
    /// is split equally among the stages. Dilute basis — the carrier and
    /// solvent flows are treated as constant, with the distribution law
    /// `Y = K·X` on a mass-of-solute/mass-of-solvent basis.
    ///
    /// Returns (raffinate solute, total extract solute).
    #[must_use]
    pub fn crosscurrent(
        &self,
        feed_mass: f64,
        solute_mass: f64,
        total_solvent: f64,
        distribution_coefficient: f64,
    ) -> (f64, f64) {
        let stages = self.num_stages.max(1);
        let solvent_per_stage = total_solvent / f64::from(stages);
        let k = distribution_coefficient.max(1e-9);
        let mut solute = solute_mass.max(0.0);
        let raffinate_mass = feed_mass.max(1e-9);
        let mut extracted_total = 0.0;
        for _ in 0..stages {
            // X = solute/raffinate phase mass; Y = K·X in the solvent.
            // Solute balance: M = X·raffinate + Y·S = X·(raffinate + K·S)
            // → X = M/(raffinate + K·S).
            let x = solute / (raffinate_mass + k * solvent_per_stage);
            let extracted = x * k * solvent_per_stage;
            solute -= extracted;
            extracted_total += extracted;
        }
        (solute, extracted_total)
    }
}

/// Single equilibrium stage. Returns (raffinate solute, extract solute).
#[must_use]
pub fn single_stage(
    feed_mass: f64,
    solute_mass: f64,
    solvent_mass: f64,
    distribution_coefficient: f64,
) -> (f64, f64) {
    let k = distribution_coefficient.max(1e-9);
    let solute = solute_mass.max(0.0);
    let raffinate = feed_mass.max(1e-9);
    let x = solute / (raffinate + k * solvent_mass);
    let in_extract = x * k * solvent_mass;
    (solute - in_extract, in_extract)
}

/// Fraction of solute remaining in the raffinate after crosscurrent
/// extraction with the stage-extraction factor `E = K·S/F` per stage:
/// `(1/(1+E))^n`.
#[must_use]
pub fn fraction_remaining(stages: u32, extraction_factor: f64) -> f64 {
    let e = extraction_factor.max(0.0);
    (1.0 / (1.0 + e)).powi(stages.max(1) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_stage_mass_balance() {
        let (raf, ext) = single_stage(100.0, 10.0, 100.0, 8.0);
        assert!((raf + ext - 10.0).abs() < 1e-9);
        // X = 10/(100 + 800) = 0.01111 → extract = 0.01111·800 = 8.89.
        assert!((ext - 10.0 * 800.0 / 900.0).abs() < 1e-9);
    }

    #[test]
    fn crosscurrent_beats_single_contact() {
        let single = single_stage(100.0, 10.0, 300.0, 8.0).0;
        let multi = Extractor::new(3).crosscurrent(100.0, 10.0, 300.0, 8.0).0;
        assert!(multi < single, "multi {multi} vs single {single}");
    }

    #[test]
    fn more_stages_recover_more() {
        let r1 = Extractor::new(1).crosscurrent(100.0, 10.0, 100.0, 5.0).0;
        let r4 = Extractor::new(4).crosscurrent(100.0, 10.0, 100.0, 5.0).0;
        assert!(r4 < r1);
    }

    #[test]
    fn fraction_remaining_formula() {
        // E = 1 per stage: (1/2)^n.
        assert!((fraction_remaining(2, 1.0) - 0.25).abs() < 1e-12);
        assert!((fraction_remaining(3, 2.0) - (1.0 / 3.0f64).powi(3)).abs() < 1e-12);
    }

    #[test]
    fn consistent_with_stage_formula() {
        // crosscurrent with equal solvent per stage matches the closed
        // form for the per-stage extraction factor.
        let (raf, _) = Extractor::new(2).crosscurrent(100.0, 20.0, 100.0, 4.0);
        let e = 4.0 * 50.0 / 100.0;
        assert!((raf - 20.0 * fraction_remaining(2, e)).abs() < 1e-9);
    }
}
