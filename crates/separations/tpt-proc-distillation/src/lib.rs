//! Distillation column design: Fenske-Underwood-Gilliland shortcut,
//! McCabe-Thiele binary stepping, and a constant-molal-overflow BP-method
//! rigorous column.
//!
//! # Example
//!
//! ```
//! use tpt_proc_distillation::{FugSpec, fug_shortcut};
//!
//! // Benzene/toluene at ~1.1 atm with α ≈ 2.4, recover 95% LK, 95% HK.
//! let spec = FugSpec {
//!     relative_volatility: 2.4,
//!     feed_mole_fraction: 0.5,
//!     light_key_recovery: 0.95,
//!     heavy_key_recovery: 0.05,
//!     actual_reflux_ratio: 1.5,
//! };
//! let result = fug_shortcut(&spec).unwrap();
//! assert!(result.min_stages > 3.0);
//! assert!(result.min_reflux < spec.actual_reflux_ratio);
//! assert!(result.actual_stages > result.min_stages);
//! ```

#![forbid(unsafe_code)]

use std::sync::Arc;

use tpt_proc_core::Composition;
use tpt_proc_core::PropertyPackage as _;
use tpt_proc_thermo_core::{Component, PropertyPackage};

mod mccabe;
mod mesh;

pub use mccabe::{mccabe_thiele_stages, MccabeResult};
pub use mesh::{simulate, ColumnSpec, MeshResult};

/// Inputs for the FUG shortcut calculation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FugSpec {
    /// Relative volatility of the light key to the heavy key (constant).
    pub relative_volatility: f64,
    /// Light-key mole fraction in the feed (binary feed basis).
    pub feed_mole_fraction: f64,
    /// Fraction of the LK in the feed recovered in the distillate, (0, 1).
    pub light_key_recovery: f64,
    /// Fraction of the HK in the feed remaining in the distillate, [0, 1).
    pub heavy_key_recovery: f64,
    /// Actual reflux ratio (> R_min for a finite stage count).
    pub actual_reflux_ratio: f64,
}

/// FUG shortcut results.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FugResult {
    /// Fenske minimum number of stages at total reflux (includes the
    /// reboiler).
    pub min_stages: f64,
    /// Underwood minimum reflux ratio.
    pub min_reflux: f64,
    /// Gilliland actual stage count at the specified reflux.
    pub actual_stages: f64,
}

/// Fenske minimum stages:
/// `N_min = ln[(x_LK/x_HK)_D · (x_HK/x_LK)_B] / ln(α_LK,HK)`.
#[must_use]
pub fn fenske_min_stages(spec: &FugSpec) -> f64 {
    let f_lk = spec.feed_mole_fraction;
    let f_hk = 1.0 - f_lk;
    let d_lk = f_lk * spec.light_key_recovery;
    let d_hk = f_hk * spec.heavy_key_recovery;
    let b_lk = f_lk - d_lk;
    let b_hk = f_hk - d_hk;
    let ratio = (d_lk / d_hk) * (b_hk / b_lk);
    ratio.ln() / spec.relative_volatility.ln()
}

/// Underwood minimum reflux for a binary at the feed condition (q = 1,
/// saturated liquid). First equation `Σ αᵢ·zᵢ/(αᵢ−θ) = 1−q = 0` is solved
/// by bisection in (1, α); the second equation
/// `R_min + 1 = Σ αᵢ·x_D,ᵢ/(αᵢ−θ)` then yields R_min.
#[must_use]
pub fn underwood_min_reflux(spec: &FugSpec) -> f64 {
    let alpha = spec.relative_volatility;
    let z_lk = spec.feed_mole_fraction;
    // First Underwood equation (q = 1): z_LK·α/(α−θ) + z_HK·1/(1−θ) = 0.
    let z_hk = 1.0 - z_lk;
    let f = |theta: f64| z_lk * alpha / (alpha - theta) + z_hk / (1.0 - theta);
    let (mut lo, mut hi) = (1.0 + 1e-9, alpha - 1e-9);
    if f(lo) * f(hi) > 0.0 {
        return f64::NAN;
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break;
        }
        if f(lo) * f(mid) <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let theta = 0.5 * (lo + hi);
    // Second Underwood equation with the distillate composition:
    // x_D,LK = d_LK/(d_LK + d_HK) on the distillate basis.
    let d_lk = z_lk * spec.light_key_recovery;
    let d_hk = z_hk * spec.heavy_key_recovery;
    let x_d_lk = d_lk / (d_lk + d_hk + 1e-12);
    let x_d_hk = 1.0 - x_d_lk;
    x_d_lk * alpha / (alpha - theta) + x_d_hk / (1.0 - theta) - 1.0
}

/// Gilliland correlation (Seader-Henley exponential fit) for actual
/// stages: Y = 1 − exp[(1+54.4X)/(11+117.2X) · (X−1)/X^0.5] with
/// X = (R − R_min)/(R + 1), valid 0.01 ≤ X ≤ 1 (Y → 0 as X → 1).
///
/// # Errors
/// Returns `Err` when R ≤ R_min (no finite stage count) or X is out of
/// the correlation's range.
pub fn gilliland_stages(n_min: f64, r_min: f64, r_actual: f64) -> Result<f64, String> {
    if r_actual <= r_min {
        return Err(format!("reflux ratio {r_actual} must exceed R_min {r_min}"));
    }
    let x = (r_actual - r_min) / (r_actual + 1.0);
    if !(0.01..=1.0).contains(&x) {
        return Err(format!("Gilliland parameter X = {x} outside [0.01, 1]"));
    }
    let y = 1.0 - (((1.0 + 54.4 * x) / (11.0 + 117.2 * x)) * (x - 1.0) / x.sqrt()).exp();
    // (N − N_min)/(N + 1) = y → N = (N_min + y)/(1 − y).
    Ok((n_min + y) / (1.0 - y))
}

/// Full FUG shortcut.
///
/// # Errors
/// Propagates [`gilliland_stages`] errors.
pub fn fug_shortcut(spec: &FugSpec) -> Result<FugResult, String> {
    let n_min = fenske_min_stages(spec);
    let r_min = underwood_min_reflux(spec);
    let actual = gilliland_stages(n_min, r_min, spec.actual_reflux_ratio)?;
    Ok(FugResult {
        min_stages: n_min,
        min_reflux: r_min,
        actual_stages: actual,
    })
}

/// Builds a benzene/toluene-like property package for column work.
#[must_use]
pub fn benzene_toluene_package() -> Arc<PropertyPackage> {
    let benzene = Component::new(
        "benzene",
        "71-43-2",
        78.114e-3,
        562.05,
        4.895e6,
        259.0e-6,
        0.210,
        353.25,
        tpt_proc_thermo_core::CpCorrelation::constant(82.4),
    );
    let toluene = Component::new(
        "toluene",
        "108-88-3",
        92.141e-3,
        591.75,
        4.106e6,
        316.0e-6,
        0.264,
        383.75,
        tpt_proc_thermo_core::CpCorrelation::constant(103.8),
    );
    Arc::new(PropertyPackage::new(vec![benzene, toluene]).expect("valid components"))
}

/// Equilibrium ratio of the light key to the heavy key from the package's
/// pure vapor pressures at `temperature` (Raoult K-ratio).
#[must_use]
pub fn relative_volatility_from_package(package: &PropertyPackage, temperature: f64) -> f64 {
    package.vapor_pressure(0, temperature) / package.vapor_pressure(1, temperature)
}

/// Convenience: K-values from a package for the given composition.
#[must_use]
pub fn package_k_values(
    package: &PropertyPackage,
    composition: &Composition,
    temperature: f64,
    pressure: f64,
) -> Vec<f64> {
    package.k_values(composition, temperature, pressure)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> FugSpec {
        FugSpec {
            relative_volatility: 2.4,
            feed_mole_fraction: 0.5,
            light_key_recovery: 0.95,
            heavy_key_recovery: 0.05,
            actual_reflux_ratio: 1.5,
        }
    }

    #[test]
    fn fenske_matches_hand_calculation() {
        // Distillate: 0.475 LK / 0.025 HK; bottoms: 0.025 LK / 0.475 HK.
        // ratio = (0.475/0.025)·(0.475/0.025) = 361 → N = ln(361)/ln(2.4).
        let expected = 361.0_f64.ln() / 2.4_f64.ln();
        let n = fenske_min_stages(&spec());
        assert!((n - expected).abs() < 1e-9);
        assert!(n > 5.0 && n < 10.0, "N_min = {n}");
    }

    #[test]
    fn underwood_is_below_actual_reflux() {
        let r_min = underwood_min_reflux(&spec());
        assert!(r_min.is_finite());
        assert!(r_min > 0.0 && r_min < 1.5, "R_min = {r_min}");
    }

    #[test]
    fn gilliland_rejects_reflux_below_minimum() {
        assert!(gilliland_stages(6.0, 1.2, 1.0).is_err());
        let n = gilliland_stages(6.0, 1.0, 1.5).unwrap();
        assert!(n > 6.0, "actual stages must exceed N_min");
    }

    #[test]
    fn reflux_increase_reduces_stages() {
        let low = fug_shortcut(&FugSpec {
            actual_reflux_ratio: 1.3,
            ..spec()
        })
        .unwrap();
        let high = fug_shortcut(&FugSpec {
            actual_reflux_ratio: 2.5,
            ..spec()
        })
        .unwrap();
        assert!(high.actual_stages < low.actual_stages);
    }

    #[test]
    fn benzene_toluene_relative_volatility_plausible() {
        let package = benzene_toluene_package();
        // At 365 K the Raoult α ≈ Psat_benzene/Psat_toluene ≈ 2.2–2.6.
        let alpha = relative_volatility_from_package(&package, 365.0);
        assert!((2.0..=3.0).contains(&alpha), "α = {alpha}");
    }
}
