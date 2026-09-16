//! Gas absorption and stripping: the Kremser equation for multistage
//! countercurrent contact with constant absorption factor.
//!
//! # Example
//!
//! ```
//! use tpt_proc_absorption::{Absorber, kremser_fraction_remaining};
//!
//! // 5 stages, absorption factor A = 1.5: fraction of solute remaining.
//! let frac = kremser_fraction_remaining(5, 1.5);
//! assert!(frac < 0.1); // deep absorption
//!
//! let absorber = Absorber::new(5);
//! let result = absorber.absorb(0.05, 1.5);
//! assert!((result.solute_recovery - (1.0 - frac)).abs() < 1e-9);
//! ```

#![forbid(unsafe_code)]

/// A countercurrent absorber/stripper with ideal stages.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Absorber {
    /// Number of theoretical stages.
    pub num_stages: u32,
}

impl Absorber {
    /// Creates an absorber.
    #[must_use]
    pub const fn new(num_stages: u32) -> Self {
        Self { num_stages }
    }

    /// Absorbs solute from a lean entering gas: lean solvent (entering
    /// solute-free) removes `A`-weighted solute per stage.
    ///
    /// `entering_solute` is the solute mole fraction in the feed gas.
    ///
    /// # Panics
    /// Never — non-finite inputs return zero-recovery results.
    #[must_use]
    pub fn absorb(&self, entering_solute: f64, absorption_factor: f64) -> AbsorptionResult {
        let frac = kremser_fraction_remaining(self.num_stages, absorption_factor);
        AbsorptionResult {
            exiting_solute: entering_solute * frac,
            solute_recovery: 1.0 - frac,
            fraction_remaining: frac,
        }
    }

    /// Strips solute from a rich entering liquid with the stripping
    /// factor S = 1/A (symmetric Kremser form).
    #[must_use]
    pub fn strip(&self, entering_solute: f64, stripping_factor: f64) -> AbsorptionResult {
        let frac = kremser_fraction_remaining(self.num_stages, stripping_factor);
        AbsorptionResult {
            exiting_solute: entering_solute * frac,
            solute_recovery: 1.0 - frac,
            fraction_remaining: frac,
        }
    }
}

/// Absorption outcome.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AbsorptionResult {
    /// Solute fraction in the exiting gas, mole fraction basis of the
    /// entering value.
    pub exiting_solute: f64,
    /// Fraction of entering solute removed into the liquid, [0, 1].
    pub solute_recovery: f64,
    /// The raw Kremser fraction remaining.
    pub fraction_remaining: f64,
}

/// Kremser fraction of entering solute remaining in the gas after `n`
/// ideal stages with absorption factor `A = L/(m·G)`:
///
/// ```text
/// frac = (A − 1) / (A^{n+1} − 1)          for A ≠ 1
/// frac = 1 / (n + 1)                      for A = 1
/// ```
///
/// Deep absorption (A > 1) drives the fraction toward zero; A < 1 leaves
/// most solute in the gas.
#[must_use]
pub fn kremser_fraction_remaining(stages: u32, absorption_factor: f64) -> f64 {
    if !absorption_factor.is_finite() || absorption_factor <= 0.0 || stages == 0 {
        return 1.0;
    }
    let a = absorption_factor;
    let n = f64::from(stages);
    if (a - 1.0).abs() < 1e-10 {
        1.0 / (n + 1.0)
    } else {
        (a - 1.0) / (a.powf(n + 1.0) - 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_stage_fraction_is_one_over_a() {
        // n = 1: frac = (A−1)/(A²−1) = 1/(A+1).
        assert!((kremser_fraction_remaining(1, 2.0) - 1.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn unit_absorption_factor_linear_limit() {
        // A = 1: each stage removes proportionally: frac = 1/(n+1).
        assert!((kremser_fraction_remaining(3, 1.0) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn more_stages_recover_more() {
        let r3 = Absorber::new(3).absorb(0.05, 1.4).solute_recovery;
        let r6 = Absorber::new(6).absorb(0.05, 1.4).solute_recovery;
        assert!(r6 > r3);
        assert!(r6 < 1.0);
    }

    #[test]
    fn absorption_factor_above_one_is_required_for_deep_recovery() {
        let weak = Absorber::new(5).absorb(0.05, 0.5).solute_recovery;
        let strong = Absorber::new(5).absorb(0.05, 2.0).solute_recovery;
        assert!(weak < 0.5);
        assert!(strong > 0.9);
    }

    #[test]
    fn stripping_mirrors_absorption() {
        let strip = Absorber::new(4).strip(0.10, 1.6);
        let absorb = Absorber::new(4).absorb(0.10, 1.6);
        assert!((strip.fraction_remaining - absorb.fraction_remaining).abs() < 1e-12);
    }
}
