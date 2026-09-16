//! Crystallization: cooling-crystallization yield from solubility curves
//! and a simple MSMPR population-balance kernel.
//!
//! # Example
//!
//! ```
//! use tpt_proc_crystallization::{cooling_yield, Crystallizer};
//!
//! // KNO3 in water: solubility ~38.3 wt% at 80 °C vs ~13.3 wt% at 20 °C.
//! let yield_frac = cooling_yield(1000.0, 0.51, 0.15);
//! assert!((0.2..0.3).contains(&yield_frac));
//!
//! let c = Crystallizer::new(2.0); // m³ suspension
//! assert!(c.msmpr_nucleation(1.0, 1.0e8, 2.0) > 0.0);
//! ```

#![forbid(unsafe_code)]

/// A well-mixed crystallizer for population-balance estimates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crystallizer {
    /// Working suspension volume, m³.
    pub volume_m3: f64,
}

impl Crystallizer {
    /// Creates a crystallizer.
    #[must_use]
    pub const fn new(volume_m3: f64) -> Self {
        Self { volume_m3 }
    }

    /// Nucleation rate (crystals/(m³·s)) by the empirical power law
    /// `B⁰ = k_B·G^i·M_T^j` with growth rate `growth_m_s` in m/s and magma
    /// density `magma_density_kg_m3` in kg/m³.
    #[must_use]
    pub fn msmpr_nucleation(&self, growth_m_s: f64, kb: f64, magma_density_kg_m3: f64) -> f64 {
        // Common exponent set i = 2, j = 1 for scale estimates.
        kb * growth_m_s.powi(2) * magma_density_kg_m3 / self.volume_m3.max(1e-9).sqrt()
    }

    /// Dominant crystal size (LarsonRandolph moments): L_D = 3.67·G·τ.
    #[must_use]
    pub fn msmpr_dominant_size(&self, growth_m_s: f64, residence_time_s: f64) -> f64 {
        3.67 * growth_m_s * residence_time_s
    }
}

/// Fraction of dissolved solute recovered as crystals on cooling a feed
/// with initial solution concentration `c_initial` (kg solute per kg free
/// solvent) to `c_final`, per kg of initial solution basis.
///
/// `yield_fraction = (c_initial − c_final)·(1 + c_final·... ` — with
/// concentrations on a solvent basis, the yield is
/// `Y = (c_i − c_f)/(1 + c_f)` per kg of initial solution when the
/// initial solution mass is 1 kg (water evaporated = 0).
#[must_use]
pub fn cooling_yield(initial_solution_kg: f64, c_initial: f64, c_final: f64) -> f64 {
    if c_final < 0.0 || c_initial <= c_final {
        return 0.0;
    }
    // Initial solvent = M/(1+c_i); crystals = solvent·(c_i − c_f).
    let solvent = initial_solution_kg / (1.0 + c_initial);
    let crystals = solvent * (c_initial - c_final);
    if initial_solution_kg <= 0.0 {
        return 0.0;
    }
    (crystals / initial_solution_kg).clamp(0.0, 1.0)
}

/// Crystal mass from cooling a solution: kg crystals = solvent_kg·(c_i −
/// c_f).
#[must_use]
pub fn crystal_mass_kg(solvent_kg: f64, c_initial: f64, c_final: f64) -> f64 {
    if c_initial <= c_final || solvent_kg <= 0.0 {
        return 0.0;
    }
    solvent_kg * (c_initial - c_final)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooling_yield_between_zero_and_one() {
        // 1000 kg solution at 0.51 kg/kg solvent cooled to 0.15.
        let y = cooling_yield(1000.0, 0.51, 0.15);
        assert!(y > 0.2 && y < 1.0, "yield = {y}");
        assert!(
            cooling_yield(1000.0, 0.15, 0.51) == 0.0,
            "heating yields nothing"
        );
    }

    #[test]
    fn crystal_mass_matches_solvent_balance() {
        // 800 kg water with KNO3: 0.51 → 0.15 → crystals = 800·0.36 = 288.
        let m = crystal_mass_kg(800.0, 0.51, 0.15);
        assert!((m - 288.0).abs() < 1e-9);
    }

    #[test]
    fn msmpr_dominant_size_scales_with_residence() {
        let c = Crystallizer::new(2.0);
        let l1 = c.msmpr_dominant_size(1e-8, 3600.0);
        let l2 = c.msmpr_dominant_size(1e-8, 7200.0);
        assert!((l2 / l1 - 2.0).abs() < 1e-12);
        // L = 3.67·G·τ: 3.67·1e-8·3600 ≈ 1.32e-4 m = 132 µm.
        assert!((l1 - 3.67e-8 * 3600.0).abs() < 1e-12);
    }

    #[test]
    fn nucleation_grows_with_supersaturation_proxy() {
        let c = Crystallizer::new(2.0);
        let b_slow = c.msmpr_nucleation(1e-9, 1e8, 200.0);
        let b_fast = c.msmpr_nucleation(1e-8, 1e8, 200.0);
        // B ∝ G²: doubling G quadruples B.
        assert!((b_fast / b_slow - 100.0).abs() < 1e-6);
    }
}
