//! Membrane separations: solution-diffusion transport for gas and liquid
//! separations.
//!
//! # Example
//!
//! ```
//! use tpt_proc_membranes::Membrane;
//!
//! // CO2-selective polymer membrane.
//! let membrane = Membrane::new(2.0e-6, 30.0); // 2 µm, 30 Barrer CO2
//! let flux = membrane.gas_flux(1.5e6, 0.2e6, 30.0e-3);
//! assert!(flux > 0.0);
//! ```

#![forbid(unsafe_code)]

/// A solution-diffusion membrane element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Membrane {
    /// Selective-layer thickness, m.
    pub thickness: f64,
    /// Permeability coefficient of the fast gas, Barrer
    /// (1 Barrer = 3.348e-16 mol·m/(m²·s·Pa)).
    pub permeability_barrer: f64,
}

/// Barrer → SI permeability (mol·m/(m²·s·Pa)).
const BARRER_TO_SI: f64 = 3.348e-16;

impl Membrane {
    /// Creates a membrane.
    #[must_use]
    pub const fn new(thickness: f64, permeability_barrer: f64) -> Self {
        Self {
            thickness,
            permeability_barrer,
        }
    }

    /// Ideal selectivity α = P_fast/P_slow from two permeabilities.
    #[must_use]
    pub fn selectivity(permeability_fast: f64, permeability_slow: f64) -> f64 {
        if permeability_slow <= 0.0 {
            return f64::INFINITY;
        }
        permeability_fast / permeability_slow
    }

    /// Steady permeate flux of the fast component, mol/(m²·s), from the
    /// solution-diffusion law `J = P·Δp/ℓ` with feed-side partial pressure
    /// `feed_pressure_pa`, permeate-side partial pressure
    /// `permeate_pressure_pa` (both of the permeating component).
    #[must_use]
    pub fn gas_flux(
        &self,
        feed_partial_pressure_pa: f64,
        permeate_partial_pressure_pa: f64,
        permeability_barrer: f64,
    ) -> f64 {
        if self.thickness <= 0.0 {
            return 0.0;
        }
        let driving = (feed_partial_pressure_pa - permeate_partial_pressure_pa).max(0.0);
        permeability_barrer * BARRER_TO_SI * driving / self.thickness
    }

    /// Permeance P/ℓ, GPU (1 GPU = 3.348e-10 mol/(m²·s·Pa)).
    #[must_use]
    pub fn permeance_gpu(&self, permeability_barrer: f64) -> f64 {
        if self.thickness <= 0.0 {
            return 0.0;
        }
        permeability_barrer * BARRER_TO_SI / self.thickness / 3.348e-10
    }

    /// Liquid (reverse-osmosis) solvent flux, m/s:
    /// `J_w = A·(ΔP − Δπ)` with water permeability A in m/(s·Pa),
    /// applied pressure difference ΔP and osmotic pressure difference Δπ
    /// (both Pa; net driving force clamped at zero).
    #[must_use]
    pub fn liquid_flux(water_permeability: f64, delta_p: f64, delta_pi: f64) -> f64 {
        let net = (delta_p - delta_pi).max(0.0);
        water_permeability * net
    }

    /// Osmotic pressure of a solution by the van 't Hoff relation
    /// π = i·c·R·T (c in mol/m³).
    #[must_use]
    pub fn osmotic_pressure(
        solutes_mol_m3: f64,
        van_t_hoff_factor: f64,
        temperature_k: f64,
    ) -> f64 {
        solutes_mol_m3 * van_t_hoff_factor * 8.314462618 * temperature_k
    }

    /// Salt rejection of an RO membrane, [0, 1]:
    /// R = 1 − c_p/c_f from the feed and permeate concentrations.
    #[must_use]
    pub fn rejection(concentration_feed: f64, concentration_permeate: f64) -> f64 {
        if concentration_feed <= 0.0 {
            return 0.0;
        }
        (1.0 - concentration_permeate / concentration_feed).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flux_scales_with_pressure_drop() {
        let m = Membrane::new(2.0e-6, 30.0);
        let f1 = m.gas_flux(1.5e6, 0.2e6, 30.0);
        let f2 = m.gas_flux(3.0e6, 0.2e6, 30.0);
        assert!((f2 / f1 - (2.8 / 1.3)).abs() < 1e-9);
        // Zero driving force → zero flux.
        assert_eq!(m.gas_flux(1e5, 1e5, 30.0), 0.0);
        // Reverse drop clamps to zero.
        assert_eq!(m.gas_flux(1e5, 1e6, 30.0), 0.0);
    }

    #[test]
    fn permeance_conversion_consistency() {
        // Flux = permeance[SI]·Δp; check GPU definition round-trip.
        let m = Membrane::new(1.0e-6, 100.0);
        let gpu = m.permeance_gpu(100.0);
        let dp = 1.0e5;
        let flux = m.gas_flux(dp, 0.0, 100.0);
        assert!((flux - gpu * 3.348e-10 * dp).abs() < 1e-15);
    }

    #[test]
    fn selectivity() {
        assert!((Membrane::selectivity(30.0, 5.0) - 6.0).abs() < 1e-12);
    }

    #[test]
    fn osmotic_pressure_of_seawater() {
        // ~35 g/L NaCl ≈ 600 mol/m³, i ≈ 2: π ≈ 600·2·8.314·298 ≈ 2.97 MPa.
        let pi = Membrane::osmotic_pressure(600.0, 2.0, 298.15);
        assert!((pi - 2.97e6).abs() < 0.1e6, "π = {pi}");
    }

    #[test]
    fn liquid_flux_requires_net_driving_pressure() {
        // Below the osmotic pressure: no flux.
        assert_eq!(Membrane::liquid_flux(1e-11, 2.0e6, 2.8e6), 0.0);
        let flux = Membrane::liquid_flux(1e-11, 5.0e6, 2.8e6);
        assert!((flux - 1e-11 * 2.2e6).abs() < 1e-18);
    }

    #[test]
    fn rejection_bounds() {
        assert!((Membrane::rejection(35.0, 0.35) - 0.99).abs() < 1e-12);
        assert_eq!(Membrane::rejection(35.0, 0.0), 1.0);
        assert_eq!(Membrane::rejection(0.0, 1.0), 0.0);
    }
}
