//! Heat transfer fundamentals: conduction, convection correlations,
//! overall coefficients.
//!
//! All correlations use SI; Nusselt/Reynolds/Prandtl are dimensionless.
//!
//! # Example
//!
//! ```
//! use tpt_proc_heat_transfer::{convection, FlowRegime};
//!
//! // Dittus-Boelter for cooling water inside a tube (Pr > 0.7).
//! let nu = convection::dittus_boelter(10_000.0, 6.0, FlowRegime::Heating);
//! assert!(nu > 0.0);
//! assert!((convection::laminar_internal_constant_t() - 3.66).abs() < 1e-12);
//! ```

#![forbid(unsafe_code)]

/// Flow regime classification for internal/external flow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowRegime {
    /// Re below the laminar limit.
    Laminar,
    /// Transition region — correlations are unreliable.
    Transition,
    /// Turbulent.
    Turbulent,
    /// Fluid is being heated (Pr exponent 0.4 in Dittus-Boelter).
    Heating,
    /// Fluid is being cooled (Pr exponent 0.3 in Dittus-Boelter).
    Cooling,
}

/// Convection correlations.
pub mod convection {
    use super::FlowRegime;

    /// Dittus-Boelter: Nu = 0.023·Re⁰·⁸·Prⁿ (n = 0.4 heating, 0.3
    /// cooling). Valid 10⁴ < Re < 1.2·10⁵, 0.7 < Pr < 120.
    #[must_use]
    pub fn dittus_boelter(reynolds: f64, prandtl: f64, regime: FlowRegime) -> f64 {
        let n = match regime {
            FlowRegime::Heating => 0.4,
            _ => 0.3,
        };
        0.023 * reynolds.powf(0.8) * prandtl.powf(n)
    }

    /// Sieder-Tate: Nu = 0.027·Re⁰·⁸·Pr^(1/3)·(μ/μ_w)⁰·¹⁴ — better for
    /// viscous liquids and large temperature differences.
    #[must_use]
    pub fn sieder_tate(reynolds: f64, prandtl: f64, viscosity_ratio_14: f64) -> f64 {
        0.027 * reynolds.powf(0.8) * prandtl.powf(1.0 / 3.0) * viscosity_ratio_14.powf(0.14)
    }

    /// Fully developed laminar internal flow with constant wall
    /// temperature: Nu = 3.66.
    #[must_use]
    pub fn laminar_internal_constant_t() -> f64 {
        3.66
    }

    /// Fully developed laminar internal flow with constant heat flux:
    /// Nu = 4.36.
    #[must_use]
    pub fn laminar_internal_constant_q() -> f64 {
        4.36
    }

    /// Turbulent internal flow selector: laminar/constant-T below Re 2300,
    /// Dittus-Boelter above 4000, Gnielinski-style interpolation flagged
    /// via the transition fallback (returns the Dittus-Boelter value with
    /// no guarantee).
    #[must_use]
    pub fn internal_flow(reynolds: f64, prandtl: f64, regime: FlowRegime) -> f64 {
        match reynolds {
            r if r < 2300.0 => laminar_internal_constant_t(),
            r if r > 4000.0 => dittus_boelter(r, prandtl, regime),
            _ => dittus_boelter(reynolds, prandtl, regime),
        }
    }

    /// Turbulent flat plate: Nu = 0.037·Re⁰·⁸·Pr^(1/3).
    #[must_use]
    pub fn flat_plate_turbulent(reynolds: f64, prandtl: f64) -> f64 {
        0.037 * reynolds.powf(0.8) * prandtl.powf(1.0 / 3.0)
    }

    /// Laminar flat plate: Nu = 0.664·Re^½·Pr^(1/3).
    #[must_use]
    pub fn flat_plate_laminar(reynolds: f64, prandtl: f64) -> f64 {
        0.664 * reynolds.powf(0.5) * prandtl.powf(1.0 / 3.0)
    }

    /// Convective coefficient from a Nusselt number: h = Nu·k/L,
    /// W/(m²·K).
    #[must_use]
    pub fn heat_transfer_coefficient(nusselt: f64, conductivity: f64, length: f64) -> f64 {
        nusselt * conductivity / length
    }

    /// Prandtl number from transport properties: cp·μ/k (all SI).
    #[must_use]
    pub fn prandtl(cp: f64, viscosity: f64, conductivity: f64) -> f64 {
        cp * viscosity / conductivity
    }

    /// Reynolds number for internal flow: ρ·v·D/μ.
    #[must_use]
    pub fn reynolds_internal(density: f64, velocity: f64, diameter: f64, viscosity: f64) -> f64 {
        density * velocity * diameter / viscosity
    }
}

/// Plane-wall / series thermal resistances, m²·K/W.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct WallResistance {
    /// Total resistance accumulated so far.
    pub total: f64,
}

impl WallResistance {
    /// Empty resistance stack.
    #[must_use]
    pub const fn new() -> Self {
        Self { total: 0.0 }
    }

    /// Adds a conductive layer: L/k.
    #[must_use]
    pub fn conductive(mut self, thickness: f64, conductivity: f64) -> Self {
        self.total += thickness / conductivity;
        self
    }

    /// Adds a convective film: 1/h.
    #[must_use]
    pub fn convective(mut self, h: f64) -> Self {
        self.total += 1.0 / h;
        self
    }

    /// Adds a fouling resistance directly.
    #[must_use]
    pub fn fouling(mut self, fouling_factor: f64) -> Self {
        self.total += fouling_factor;
        self
    }

    /// Overall heat-transfer coefficient over area A: U = 1/(R·A),
    /// W/(m²·K).
    #[must_use]
    pub fn overall_u(&self, area: f64) -> f64 {
        1.0 / (self.total * area)
    }
}

/// 1-D steady conduction through a wall: Q = ΔT/R_total per unit area
/// basis handled by [`WallResistance`]. Returns heat flux W/m².
#[must_use]
pub fn conduction_flux(delta_t: f64, resistance_per_area: f64) -> f64 {
    delta_t / resistance_per_area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dittus_boelter_reference_value() {
        // Nu = 0.023·10000^0.8·7^0.4 ≈ 79.6.
        let nu = convection::dittus_boelter(1e4, 7.0, FlowRegime::Heating);
        assert!((nu - 0.023 * 1e4f64.powf(0.8) * 7f64.powf(0.4)).abs() < 1e-9);
        // Cooling uses n = 0.3 → smaller than heating for Pr > 1.
        let cool = convection::dittus_boelter(1e4, 7.0, FlowRegime::Cooling);
        assert!(cool < nu);
    }

    #[test]
    fn laminar_constants() {
        assert!((convection::laminar_internal_constant_t() - 3.66).abs() < 1e-12);
        assert!((convection::laminar_internal_constant_q() - 4.36).abs() < 1e-12);
        // The internal-flow selector falls back to laminar below Re 2300.
        assert_eq!(
            convection::internal_flow(1000.0, 7.0, FlowRegime::Cooling),
            3.66
        );
    }

    #[test]
    fn h_from_nusselt() {
        // h = 79.6·0.6/0.02 ≈ 2388 W/(m²·K).
        let h = convection::heat_transfer_coefficient(79.6, 0.6, 0.02);
        assert!((h - 2388.0).abs() < 1.0);
    }

    #[test]
    fn prandtl_of_water() {
        // cp = 4180 J/(kg·K), μ = 1e-3, k = 0.6 → Pr ≈ 6.97.
        let pr = convection::prandtl(4180.0, 1.0e-3, 0.6);
        assert!((pr - 6.97).abs() < 0.05);
    }

    #[test]
    fn wall_resistance_series() {
        // Classic composite wall: two convective films + one conductive
        // layer; U = 1/(1/h1 + L/k + 1/h2).
        let wall = WallResistance::new()
            .convective(10.0)
            .conductive(0.1, 0.5)
            .convective(20.0);
        let expected_total = 1.0 / 10.0 + 0.1 / 0.5 + 1.0 / 20.0;
        assert!((wall.total - expected_total).abs() < 1e-12);
        // Per unit area, U = 1/R.
        assert!((wall.overall_u(1.0) - 1.0 / expected_total).abs() < 1e-9);
        // Flux through 40 K: q = ΔT/R.
        let flux = conduction_flux(40.0, expected_total);
        assert!((flux - 40.0 / expected_total).abs() < 1e-9);
    }
}
