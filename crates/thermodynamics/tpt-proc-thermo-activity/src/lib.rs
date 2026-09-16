//! Activity coefficient models for the liquid phase.
//!
//! All models implement [`ActivityCoefficientModel`](tpt_proc_thermo_core::ActivityCoefficientModel)
//! and can be attached to a property package to drive the modified-Raoult
//! K-value path `Kᵢ = γᵢ·P_sat,ᵢ/P`:
//!
//! - [`Nrtl`] — electrolyte-free NRTL with temperature-independent τ and
//!   non-randomness α;
//! - [`Wilson`] — Wilson's local-composition model (miscible systems only);
//! - [`Uniquac`] — UNIQUAC combinatorial + residual;
//! - [`Unifac`] — UNIFAC group-contribution estimation (built-in table
//!   covers alkane/aromatic/alcohol/water groups; load fuller tables for
//!   production use).
//!
//! Pair parameters are stored by component *index* in the package's
//! component list.
//!
//! # Example
//!
//! ```
//! use tpt_proc_thermo_activity::{Nrtl, NrtlParams};
//! use tpt_proc_thermo_core::ActivityCoefficientModel;
//!
//! // Ethanol(0)/water(1), illustrative NRTL parameters (J/mol).
//! let nrtl = Nrtl::from_params(2, NrtlParams::binary(0.30, 728.0, 385.0));
//! let gamma = nrtl.gamma(&[0.5, 0.5], 351.45).unwrap();
//!
//! // Non-ideal but finite; both gammas are plausible magnitudes.
//! assert!(gamma[0] > 1.0 && gamma[0] < 3.0);
//! assert!(gamma[1] > 0.8 && gamma[1] < 3.0);
//! ```

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use tpt_proc_thermo_core::ThermoError;

mod nrtl;
mod unifac;
mod uniquac;
mod wilson;

pub use nrtl::{Nrtl, NrtlParams};
pub use unifac::{GroupCount, Unifac, UnifacTable};
pub use uniquac::{Uniquac, UniquacParams};
pub use wilson::{Wilson, WilsonParams};

/// Shared parameter-map type: entries keyed by index pairs.
pub type PairMap = BTreeMap<(usize, usize), f64>;

/// Validates a composition slice: finite, non-negative, sums to ~1.
pub(crate) fn validate_composition(x: &[f64]) -> Result<(), ThermoError> {
    let sum: f64 = x.iter().sum();
    if x.is_empty() || x.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(ThermoError::InvalidComponent(
            "composition must be finite and non-negative".into(),
        ));
    }
    if (sum - 1.0).abs() > 1e-3 {
        return Err(ThermoError::InvalidComponent(format!(
            "composition must sum to 1 (got {sum})"
        )));
    }
    Ok(())
}

/// Molar excess Gibbs energy over RT, Σ xᵢ ln γᵢ, from any model.
///
/// Useful for mixing-rule coupling (RFC 0001) and as a test invariant.
#[must_use]
pub fn excess_gibbs_over_rt(ln_gamma: &[f64], x: &[f64]) -> f64 {
    x.iter().zip(ln_gamma).map(|(xi, g)| xi * g).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excess_gibbs_is_weighted_sum() {
        let g = excess_gibbs_over_rt(&[0.2_f64.ln(), 1.5_f64.ln()], &[0.25, 0.75]);
        let expected = 0.25 * 0.2_f64.ln() + 0.75 * 1.5_f64.ln();
        assert!((g - expected).abs() < 1e-12);
    }

    #[test]
    fn rejects_bad_composition() {
        assert!(validate_composition(&[]).is_err());
        assert!(validate_composition(&[0.5, 0.6]).is_err());
        assert!(validate_composition(&[-0.1, 1.1]).is_err());
        assert!(validate_composition(&[0.25, 0.75]).is_ok());
    }
}
