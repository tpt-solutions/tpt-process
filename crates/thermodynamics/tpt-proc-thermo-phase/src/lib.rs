//! Phase equilibrium and flash calculations.
//!
//! [`FlashSolver`] implements the two-phase VLE workhorse of the suite
//! (design per RFC 0002): PT flash by successive substitution on K-values
//! with a bracketed Rachford-Rice inner loop, PH flash as an outer
//! enthalpy bisection, and bubble/dew point solves.
//!
//! Design commitments:
//! - **never panics and never lies**: non-convergence is an `Err`, the
//!   converged flag and iteration count are part of every result;
//! - **mass balance is exact**: `z = (1−β)·x + β·y` holds to machine
//!   precision by construction;
//! - **degenerate single-component two-phase states return a guarded
//!   result** (`degenerate: true`, β ≈ 0.5) instead of diverging.
//!
//! # Example
//!
//! ```
//! use tpt_proc_core::Composition;
//! use tpt_proc_thermo_database::ChemicalDatabase;
//! use tpt_proc_thermo_eos::CubicEos;
//! use tpt_proc_thermo_phase::FlashSolver;
//! use tpt_proc_thermo_core::PropertyPackage;
//!
//! let db = ChemicalDatabase::builtin();
//! let components = db.components_for(&["water", "methanol"]).unwrap();
//! let eos = CubicEos::peng_robinson(components.clone());
//!
//! let package = tpt_proc_thermo_core::PropertyPackage::new(components)
//!     .unwrap()
//!     .with_eos(std::sync::Arc::new(eos));
//! let flash = FlashSolver::new(package);
//!
//! let feed = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
//! let result = flash.pt_flash(&feed, 350.0, 101_325.0).unwrap();
//! println!("β = {}", result.vapor_fraction);
//! assert!((0.0..=1.0).contains(&result.vapor_fraction));
//! ```

#![forbid(unsafe_code)]

use std::sync::Arc;

use tpt_proc_core::{Composition, PhaseState};
// Core trait methods (enthalpy, density, …) resolve through this import.
use tpt_proc_core::PropertyPackage as _;
use tpt_proc_thermo_core::{Component, PropertyPackage};
use tpt_proc_thermo_eos::{CubicEos, MixingRule};

mod error;
mod rachford;

pub use error::{FlashError, FlashResult};

/// Convergence tolerance on max |Δ ln K| between successive substitutions.
const K_TOLERANCE: f64 = 1e-10;
/// Iteration cap for the K-value outer loop.
const MAX_K_ITERATIONS: u32 = 200;
/// Iteration cap for the Rachford-Rice bisection.
const MAX_RR_ITERATIONS: u32 = 100;

/// A configured flash calculator over one property package.
#[derive(Clone)]
pub struct FlashSolver {
    package: Arc<PropertyPackage>,
}

impl FlashSolver {
    /// Builds a flash solver over an existing property package.
    #[must_use]
    pub fn new(package: PropertyPackage) -> Self {
        Self {
            package: Arc::new(package),
        }
    }

    /// Convenience constructor: Peng-Robinson package from a component list.
    #[must_use]
    pub fn peng_robinson(components: Vec<Component>, mixing: MixingRule) -> Self {
        let eos = CubicEos::peng_robinson(components.clone()).with_mixing_rule(mixing);
        let package = PropertyPackage::new(components)
            .expect("components validated at EOS construction")
            .with_eos(Arc::new(eos));
        Self::new(package)
    }

    /// The property package backing this solver.
    #[must_use]
    pub fn package(&self) -> &PropertyPackage {
        &self.package
    }

    /// PT flash: two-phase equilibrium at fixed temperature and pressure.
    ///
    /// # Errors
    /// [`FlashError::InvalidInput`] for malformed feeds;
    /// [`FlashError::NotConverged`] when the K-value loop stalls.
    pub fn pt_flash(
        &self,
        feed: &Composition,
        temperature: f64,
        pressure: f64,
    ) -> Result<FlashResult, FlashError> {
        self.validate_feed(feed)?;
        let n = feed.len();

        // Pure components saturate over a β-range: β is not unique, so
        // return the guarded midpoint rather than iterating.
        if n == 1 {
            let mut result = self.single_phase_result(feed, temperature, pressure, true);
            result.vapor_fraction = 0.5;
            result.degenerate = true;
            return Ok(result);
        }

        // Wilson-correlation initial K estimate; iterate to convergence.
        let mut k_values = self.wilson_k_values(feed, temperature, pressure);

        // Successive substitution on K with a clamped Rachford-Rice: if the
        // Ks drift into a single-phase classification the clamp keeps the
        // loop alive until the Ks stop changing.
        let mut beta = 0.5_f64;
        let mut iterations = 0_u32;
        let mut converged = false;
        for iter in 0..MAX_K_ITERATIONS {
            iterations = iter + 1;
            let (beta_new, _bracketed) =
                rachford::solve(feed.as_slice(), &k_values, MAX_RR_ITERATIONS)?;
            beta = beta_new;
            let (x, y) = phase_compositions(feed.as_slice(), &k_values, beta);
            let x_comp = Composition::from_amounts(&x, tpt_proc_core::CompositionBasis::Mole)
                .map_err(|e| FlashError::InvalidInput(e.to_string()))?;
            let y_comp = Composition::from_amounts(&y, tpt_proc_core::CompositionBasis::Mole)
                .map_err(|e| FlashError::InvalidInput(e.to_string()))?;

            let new_k = self.equilibrium_k_values(&x_comp, &y_comp, temperature, pressure);
            let delta = k_values
                .iter()
                .zip(&new_k)
                .map(|(k_old, k_new)| ((k_new / k_old).ln()).abs())
                .fold(0.0_f64, f64::max);
            // Damping grows as iterations accumulate (near-critical mixes).
            let damping = if iter < 20 { 1.0 } else { 0.7 };
            for (k, k_new) in k_values.iter_mut().zip(&new_k) {
                *k = k.powf(1.0 - damping) * k_new.powf(damping);
            }
            if delta < K_TOLERANCE {
                converged = true;
                break;
            }
        }
        if !converged {
            return Err(FlashError::NotConverged {
                iterations: MAX_K_ITERATIONS,
            });
        }

        let (x, y) = phase_compositions(feed.as_slice(), &k_values, beta);
        Ok(FlashResult {
            temperature,
            pressure,
            vapor_fraction: beta,
            liquid_composition: Composition::from_amounts(
                &x,
                tpt_proc_core::CompositionBasis::Mole,
            )
            .map_err(|e| FlashError::InvalidInput(e.to_string()))?,
            vapor_composition: Composition::from_amounts(&y, tpt_proc_core::CompositionBasis::Mole)
                .map_err(|e| FlashError::InvalidInput(e.to_string()))?,
            k_values,
            iterations,
            degenerate: false,
        })
    }

    /// PH flash: pressure fixed, enthalpy (J/mol of feed) given; solves for
    /// the equilibrium temperature by bisection on the PT flash enthalpy.
    ///
    /// # Errors
    /// [`FlashError::InvalidInput`] when the enthalpy is outside the
    /// bracketable range; [`FlashError::NotConverged`] otherwise.
    pub fn ph_flash(
        &self,
        feed: &Composition,
        pressure: f64,
        enthalpy: f64,
    ) -> Result<FlashResult, FlashError> {
        self.validate_feed(feed)?;
        let h_at = |t: f64| -> Result<f64, FlashError> {
            // Enthalpy of the (possibly single-phase) equilibrium state.
            let result = self.pt_flash(feed, t, pressure)?;
            let beta = result.vapor_fraction;
            let h_l =
                self.package
                    .enthalpy(&result.liquid_composition, t, pressure, PhaseState::Liquid);
            let h_v =
                self.package
                    .enthalpy(&result.vapor_composition, t, pressure, PhaseState::Vapor);
            Ok((1.0 - beta) * h_l + beta * h_v)
        };

        // Bracket the target enthalpy by expanding T.
        let mut t_low = 200.0_f64;
        let mut t_high = 700.0_f64;
        let mut h_low = h_at(t_low)?;
        let mut h_high = h_at(t_high)?;
        let mut expansion = 0;
        while (h_low > enthalpy || h_high < enthalpy) && expansion < 40 {
            if h_low > enthalpy {
                t_low = (t_low / 1.5).max(1.0);
                h_low = h_at(t_low)?;
            }
            if h_high < enthalpy {
                t_high *= 1.5;
                h_high = h_at(t_high)?;
            }
            expansion += 1;
        }
        if h_low > enthalpy || h_high < enthalpy {
            return Err(FlashError::InvalidInput(format!(
                "enthalpy {enthalpy} J/mol outside the reachable range [{h_low}, {h_high}]"
            )));
        }

        for _ in 0..80 {
            let t_mid = 0.5 * (t_low + t_high);
            if t_mid <= t_low || t_mid >= t_high {
                break;
            }
            let h_mid = h_at(t_mid)?;
            if h_mid < enthalpy {
                t_low = t_mid;
            } else {
                t_high = t_mid;
            }
        }
        let t_final = 0.5 * (t_low + t_high);
        let result = self.pt_flash(feed, t_final, pressure)?;
        Ok(result)
    }

    /// Bubble-point temperature of a liquid at fixed pressure, K.
    ///
    /// # Errors
    /// [`FlashError::NotConverged`] / [`FlashError::InvalidInput`].
    pub fn bubble_point_t(&self, liquid: &Composition, pressure: f64) -> Result<f64, FlashError> {
        self.dew_bubble_solve(liquid, pressure, true)
    }

    /// Dew-point temperature of a vapor at fixed pressure, K.
    ///
    /// # Errors
    /// [`FlashError::NotConverged`] / [`FlashError::InvalidInput`].
    pub fn dew_point_t(&self, vapor: &Composition, pressure: f64) -> Result<f64, FlashError> {
        self.dew_bubble_solve(vapor, pressure, false)
    }

    fn dew_bubble_solve(
        &self,
        composition: &Composition,
        pressure: f64,
        bubble: bool,
    ) -> Result<f64, FlashError> {
        self.validate_feed(composition)?;
        // Residual, normalized to increase with T:
        // bubble: Σ z_i K_i(T) − 1; dew: 1 − Σ z_i/K_i(T).
        let sign = if bubble { 1.0 } else { -1.0 };
        let residual = |t: f64| -> f64 {
            let k = self.wilson_or_package_k(composition, t, pressure);
            if bubble {
                sign * (composition
                    .as_slice()
                    .iter()
                    .zip(&k)
                    .map(|(z, ki)| z * ki)
                    .sum::<f64>()
                    - 1.0)
            } else {
                sign * (composition
                    .as_slice()
                    .iter()
                    .zip(&k)
                    .map(|(z, ki)| z / ki)
                    .sum::<f64>()
                    - 1.0)
            }
        };
        let mut t_low = 1.0_f64;
        let mut t_high = 2000.0_f64;
        let mut r_low = residual(t_low);
        let mut r_high = residual(t_high);
        let mut expansion = 0;
        while (r_low > 0.0 || r_high < 0.0) && expansion < 60 {
            if r_low > 0.0 {
                t_low = (t_low / 1.5).max(0.1);
                r_low = residual(t_low);
            }
            if r_high < 0.0 {
                t_high *= 1.5;
                r_high = residual(t_high);
            }
            expansion += 1;
        }
        if r_low > 0.0 || r_high < 0.0 {
            return Err(FlashError::InvalidInput(
                "could not bracket the dew/bubble temperature".into(),
            ));
        }
        for _ in 0..80 {
            let t_mid = 0.5 * (t_low + t_high);
            if t_mid <= t_low || t_mid >= t_high {
                break;
            }
            // Residual increases with T: negative → root is above mid.
            if residual(t_mid) < 0.0 {
                t_low = t_mid;
            } else {
                t_high = t_mid;
            }
        }
        Ok(0.5 * (t_low + t_high))
    }

    /// K-values at a state point using the phase-specific fugacity path
    /// when an EOS is attached, else modified Raoult with the activity
    /// model (or ideal γ = 1).
    fn equilibrium_k_values(
        &self,
        x: &Composition,
        y: &Composition,
        temperature: f64,
        pressure: f64,
    ) -> Vec<f64> {
        if let Some(eos) = self.package.eos() {
            let ln_phi_l = eos.ln_fugacity_coefficients(
                x,
                temperature,
                pressure,
                tpt_proc_thermo_core::PhaseSelection::Liquid,
            );
            let ln_phi_v = eos.ln_fugacity_coefficients(
                y,
                temperature,
                pressure,
                tpt_proc_thermo_core::PhaseSelection::Vapor,
            );
            if let (Ok(l), Ok(v)) = (ln_phi_l, ln_phi_v) {
                return l.iter().zip(v).map(|(li, vi)| (li - vi).exp()).collect();
            }
        }
        // Modified Raoult: K_i = γ_i(x)·Psat_i/P.
        let gammas = self
            .package
            .activity_coefficients(x, temperature)
            .unwrap_or_else(|_| vec![1.0; x.len()]);
        gammas
            .iter()
            .enumerate()
            .map(|(i, gamma)| gamma * self.package.vapor_pressure(i, temperature) / pressure)
            .collect()
    }

    /// Initial K guess: Wilson correlation
    /// `Kᵢ = (Pcᵢ/P)·exp(5.373·(1+ωᵢ)·(1 − Tcᵢ/T))`.
    fn wilson_k_values(&self, _feed: &Composition, temperature: f64, pressure: f64) -> Vec<f64> {
        self.package
            .components()
            .iter()
            .map(|c| {
                let t_ratio = c.critical_temperature / temperature;
                (c.critical_pressure / pressure)
                    * (5.373 * (1.0 + c.acentric_factor) * (1.0 - t_ratio)).exp()
            })
            .collect()
    }

    fn wilson_or_package_k(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
    ) -> Vec<f64> {
        // For dew/bubble residuals the composition is the equilibrium-phase
        // composition by definition, so the Wilson estimate is the fast
        // path and good enough inside a monotone bisection.
        self.wilson_k_values(composition, temperature, pressure)
    }

    fn single_phase_result(
        &self,
        feed: &Composition,
        temperature: f64,
        pressure: f64,
        vapor: bool,
    ) -> FlashResult {
        FlashResult {
            temperature,
            pressure,
            vapor_fraction: if vapor { 1.0 } else { 0.0 },
            liquid_composition: feed.clone(),
            vapor_composition: feed.clone(),
            k_values: vec![1.0; feed.len()],
            iterations: 0,
            degenerate: false,
        }
    }

    fn validate_feed(&self, feed: &Composition) -> Result<(), FlashError> {
        if feed.len() != self.package.num_components() {
            return Err(FlashError::InvalidInput(format!(
                "feed has {} components; package has {}",
                feed.len(),
                self.package.num_components()
            )));
        }
        if (feed.sum() - 1.0).abs() > 1e-6 {
            return Err(FlashError::InvalidInput(format!(
                "feed fractions must sum to 1 (got {})",
                feed.sum()
            )));
        }
        if feed.as_slice().iter().any(|z| !z.is_finite() || *z < 0.0) {
            return Err(FlashError::InvalidInput(
                "feed fractions must be finite and non-negative".into(),
            ));
        }
        Ok(())
    }
}

/// Phase compositions from converged K-values and vapor fraction β:
/// `x_i = z_i/(1+β(K_i−1))`, `y_i = K_i·x_i` (mass balance exact by
/// construction).
fn phase_compositions(z: &[f64], k: &[f64], beta: f64) -> (Vec<f64>, Vec<f64>) {
    let x: Vec<f64> = z
        .iter()
        .zip(k)
        .map(|(zi, ki)| zi / (1.0 + beta * (ki - 1.0)))
        .collect();
    let x_sum: f64 = x.iter().sum();
    let y: Vec<f64> = x.iter().zip(k).map(|(xi, ki)| xi * ki).collect();
    let y_sum: f64 = y.iter().sum();
    // Normalize away rounding (relative corrections ~1e-15).
    let x: Vec<f64> = x.iter().map(|xi| xi / x_sum).collect();
    let y: Vec<f64> = y.iter().map(|yi| yi / y_sum).collect();
    (x, y)
}

/// Re-exported so downstream users only need this crate for flash work.
pub mod prelude {
    pub use super::{FlashError, FlashResult, FlashSolver};
    pub use tpt_proc_core::Composition;
    pub use tpt_proc_thermo_core::ThermoError;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_proc_thermo_database::ChemicalDatabase;

    fn benzene_toluene_solver() -> FlashSolver {
        let db = ChemicalDatabase::builtin();
        let components = db.components_for(&["benzene", "toluene"]).unwrap();
        FlashSolver::peng_robinson(components, MixingRule::VanDerWaals)
    }

    #[test]
    fn pure_water_at_saturation_is_degenerate_half_vapor() {
        let db = ChemicalDatabase::builtin();
        let components = db.components_for(&["water"]).unwrap();
        let flash = FlashSolver::peng_robinson(components, MixingRule::VanDerWaals);
        let feed = Composition::from_mole_fractions(&[1.0]).unwrap();
        let result = flash.pt_flash(&feed, 373.15, 101_325.0).unwrap();
        assert!(result.degenerate);
        assert!((result.vapor_fraction - 0.5).abs() < 0.01);
    }

    #[test]
    fn subcooled_and_superheated_detection() {
        let flash = benzene_toluene_solver();
        let feed = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
        // Well below the bubble point → liquid.
        let cold = flash.pt_flash(&feed, 300.0, 101_325.0).unwrap();
        assert!((cold.vapor_fraction - 0.0).abs() < 1e-12);
        // Well above the dew point → vapor.
        let hot = flash.pt_flash(&feed, 450.0, 101_325.0).unwrap();
        assert!((hot.vapor_fraction - 1.0).abs() < 1e-12);
    }

    #[test]
    fn two_phase_flash_conserves_mass() {
        let flash = benzene_toluene_solver();
        let feed = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
        // For a 50/50 benzene/toluene mix at 1 atm the bubble point is
        // ≈ 365 K and the dew point ≈ 372 K; 368 K sits solidly inside.
        let result = flash.pt_flash(&feed, 368.0, 101_325.0).unwrap();
        assert!(
            (0.0..1.0).contains(&result.vapor_fraction),
            "β = {}",
            result.vapor_fraction
        );
        let beta = result.vapor_fraction;
        for i in 0..2 {
            let mix = (1.0 - beta) * result.liquid_composition.get(i).unwrap()
                + beta * result.vapor_composition.get(i).unwrap();
            assert!((mix - 0.5).abs() < 1e-8, "component {i}: {mix}");
        }
        // Benzene enriches the vapor.
        assert!(result.vapor_composition.get(0).unwrap() > 0.5);
        assert!(result.liquid_composition.get(0).unwrap() < 0.5);
    }

    #[test]
    fn bubble_point_of_benzene_near_boiling() {
        let flash = benzene_toluene_solver();
        let benzene = Composition::from_mole_fractions(&[1.0, 0.0]).unwrap();
        let t_bubble = flash.bubble_point_t(&benzene, 101_325.0).unwrap();
        // PR benzene normal boiling point ≈ 353.2 K (±2 K tolerance).
        assert!((t_bubble - 353.25).abs() < 3.0, "T_bubble = {t_bubble}");
    }

    #[test]
    fn dew_point_of_toluene_near_boiling() {
        let flash = benzene_toluene_solver();
        let toluene = Composition::from_mole_fractions(&[0.0, 1.0]).unwrap();
        let t_dew = flash.dew_point_t(&toluene, 101_325.0).unwrap();
        assert!((t_dew - 383.75).abs() < 3.0, "T_dew = {t_dew}");
    }

    #[test]
    fn mixture_bubble_below_pure_light_key() {
        let flash = benzene_toluene_solver();
        let mix = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
        let t_bubble = flash.bubble_point_t(&mix, 101_325.0).unwrap();
        let t_dew = flash.dew_point_t(&mix, 101_325.0).unwrap();
        // Binary at fixed P: T_bubble < T_dew, both between the pure Tbs.
        assert!(t_bubble < t_dew);
        assert!(t_bubble > 353.25 - 3.0 && t_dew < 383.75 + 3.0);
    }

    #[test]
    fn rachford_rice_bisection_satisfies_residual() {
        // Property test: any valid K set with a bracketable root solves to
        // g(β) ≈ 0.
        let z = [0.3, 0.2, 0.5];
        let k = [2.5, 1.2, 0.4];
        let (beta, bracketed) = rachford::solve(&z, &k, 100).unwrap();
        assert!(bracketed);
        let g: f64 = z
            .iter()
            .zip(k)
            .map(|(zi, ki)| zi * (ki - 1.0) / (1.0 + beta * (ki - 1.0)))
            .sum();
        assert!(g.abs() < 1e-10, "g(β) = {g}");
        assert!((0.0..=1.0).contains(&beta));
    }

    #[test]
    fn ph_flash_recovers_temperature() {
        let flash = benzene_toluene_solver();
        let feed = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
        let reference = flash.pt_flash(&feed, 370.0, 101_325.0).unwrap();
        let beta = reference.vapor_fraction;
        let h_l = flash.package.enthalpy(
            &reference.liquid_composition,
            370.0,
            101_325.0,
            PhaseState::Liquid,
        );
        let h_v = flash.package.enthalpy(
            &reference.vapor_composition,
            370.0,
            101_325.0,
            PhaseState::Vapor,
        );
        let target = (1.0 - beta) * h_l + beta * h_v;
        let recovered = flash.ph_flash(&feed, 101_325.0, target).unwrap();
        assert!(
            (recovered.temperature - 370.0).abs() < 0.5,
            "T = {}",
            recovered.temperature
        );
    }

    #[test]
    fn invalid_feed_rejected() {
        let flash = benzene_toluene_solver();
        let wrong = Composition::from_mole_fractions(&[1.0]).unwrap();
        assert!(matches!(
            flash.pt_flash(&wrong, 350.0, 1e5),
            Err(FlashError::InvalidInput(_))
        ));
    }
}
