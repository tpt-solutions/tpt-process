//! Refining stream characterization: API gravity, Watson K factor,
//! boiling-point cut blending, and simplified crude assays.
//!
//! # Example
//!
//! ```
//! use tpt_proc_refining::{api_to_specific_gravity, watson_k_factor};
//!
//! // A 40° API light stream.
//! let sg = api_to_specific_gravity(40.0);
//! assert!((sg - 0.8251).abs() < 1e-3);
//!
//! // Watson characterization factor from MeABP (avg boiling point).
//! let k = watson_k_factor(650.0, sg); // °R basis
//! assert!((10.0..=13.0).contains(&k));
//! ```

#![forbid(unsafe_code)]
#![allow(clippy::similar_names)]

/// Converts API gravity to specific gravity (60 °F relative to water).
#[must_use]
pub fn api_to_specific_gravity(api: f64) -> f64 {
    141.5 / (api + 131.5)
}

/// Converts specific gravity to API gravity.
#[must_use]
pub fn specific_gravity_to_api(sg: f64) -> f64 {
    141.5 / sg - 131.5
}

/// Watson characterization factor K = (Tb_R)^(1/3)/sg, with the mean
/// average boiling point `meabp_rankine` in degrees Rankine. (The familiar
/// 1.216 constant applies to the Kelvin form: K = 1.216·Tb_K^(1/3)/sg.)
#[must_use]
pub fn watson_k_factor(meabp_rankine: f64, specific_gravity: f64) -> f64 {
    if specific_gravity <= 0.0 || meabp_rankine <= 0.0 {
        return f64::NAN;
    }
    meabp_rankine.powf(1.0 / 3.0) / specific_gravity
}

/// A boiling-point cut with mass yield over a crude assay.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cut {
    /// Cut name (e.g. "naphtha").
    pub name: &'static str,
    /// Boiling range: initial point, °F.
    pub initial_boiling_point_f: f64,
    /// Boiling range: final point, °F.
    pub final_boiling_point_f: f64,
    /// Mass fraction of the crude, [0, 1].
    pub mass_fraction: f64,
}

/// A simple crude assay: a list of cuts that should sum to ~1.
#[derive(Clone, Debug, PartialEq)]
pub struct CrudeAssay {
    /// Crude name.
    pub name: String,
    /// Cuts by boiling range.
    pub cuts: Vec<Cut>,
}

impl CrudeAssay {
    /// Creates an assay.
    #[must_use]
    pub fn new(name: &str, cuts: Vec<Cut>) -> Self {
        Self {
            name: name.to_string(),
            cuts,
        }
    }

    /// Total normalized yield (should be ≈ 1).
    #[must_use]
    pub fn total_yield(&self) -> f64 {
        self.cuts.iter().map(|c| c.mass_fraction).sum()
    }

    /// Weighted mean API gravity of the crude given per-cut gravities
    /// (paired by index; missing entries default to 35° API).
    #[must_use]
    pub fn weighted_api(&self, cut_apis: &[f64]) -> f64 {
        let mut weighted = 0.0;
        for (i, cut) in self.cuts.iter().enumerate() {
            let api = cut_apis.get(i).copied().unwrap_or(35.0);
            weighted += cut.mass_fraction * api;
        }
        weighted
    }

    /// Yield of a product blended from cuts whose final boiling point is
    /// at or below `max_final_bp_f` (e.g. the gasoline pool).
    #[must_use]
    pub fn yield_below(&self, max_final_bp_f: f64) -> f64 {
        self.cuts
            .iter()
            .filter(|c| c.final_boiling_point_f <= max_final_bp_f)
            .map(|c| c.mass_fraction)
            .sum()
    }
}

/// A reference light-crude assay shape (illustrative; ~40° API class).
#[must_use]
pub fn light_crude_assay() -> CrudeAssay {
    CrudeAssay::new(
        "light-crude",
        vec![
            Cut {
                name: "butanes-plus",
                initial_boiling_point_f: 90.0,
                final_boiling_point_f: 175.0,
                mass_fraction: 0.06,
            },
            Cut {
                name: "light-naphtha",
                initial_boiling_point_f: 175.0,
                final_boiling_point_f: 250.0,
                mass_fraction: 0.10,
            },
            Cut {
                name: "heavy-naphtha",
                initial_boiling_point_f: 250.0,
                final_boiling_point_f: 380.0,
                mass_fraction: 0.17,
            },
            Cut {
                name: "kerosene",
                initial_boiling_point_f: 380.0,
                final_boiling_point_f: 500.0,
                mass_fraction: 0.13,
            },
            Cut {
                name: "diesel",
                initial_boiling_point_f: 500.0,
                final_boiling_point_f: 650.0,
                mass_fraction: 0.18,
            },
            Cut {
                name: "vgo",
                initial_boiling_point_f: 650.0,
                final_boiling_point_f: 1000.0,
                mass_fraction: 0.24,
            },
            Cut {
                name: "vacuum-resid",
                initial_boiling_point_f: 1000.0,
                final_boiling_point_f: 1400.0,
                mass_fraction: 0.12,
            },
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_gravity_roundtrip() {
        let sg = api_to_specific_gravity(40.0);
        assert!((sg - 0.8251).abs() < 1e-3);
        assert!((specific_gravity_to_api(sg) - 40.0).abs() < 1e-9);
        // Water is 10° API.
        assert!((specific_gravity_to_api(1.0) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn watson_k_classifies_hydrocarbon_type() {
        // Paraffinic: K ≈ 12.5+; naphthenic: 11-12; aromatic: ≈ 10.5.
        let k = watson_k_factor(650.0 + 459.67, 0.8);
        assert!((10.5..=13.0).contains(&k), "K = {k}");
    }

    #[test]
    fn assay_yields_sum_and_pool() {
        let assay = light_crude_assay();
        assert!((assay.total_yield() - 1.0).abs() < 1e-9);
        // Naphtha pool: everything up to 380 °F.
        let naphtha = assay.yield_below(380.0);
        assert!((naphtha - 0.33).abs() < 1e-9, "naphtha = {naphtha}");
    }

    #[test]
    fn weighted_api_is_mass_weighted() {
        let assay = light_crude_assay();
        let apis = vec![95.0, 70.0, 55.0, 45.0, 38.0, 24.0, 12.0];
        let api = assay.weighted_api(&apis);
        assert!(api > 35.0 && api < 50.0, "API = {api}");
        // Missing gravities default to 35.
        assert!((assay.weighted_api(&[]) - 35.0).abs() < 1e-9);
    }
}
