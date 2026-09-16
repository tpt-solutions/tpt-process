//! Pipe flow hydraulics: Reynolds number, friction factors, Darcy-Weisbach.
//!
//! All quantities SI. The friction factor is the **Darcy** factor
//! (f = 64/Re laminar), not the Fanning factor (divide by 4 to convert).
//!
//! # Example
//!
//! ```
//! use tpt_proc_fluid::{FluidProperties, PipeFlow};
//!
//! // Water in a 100 m commercial-steel pipe, 0.05 m diameter.
//! let pipe = PipeFlow::new(0.05, 100.0, 4.6e-5)
//!     .with_fluid(FluidProperties { density: 998.0, viscosity: 1.0e-3 });
//!
//! let v = 1.0; // m/s
//! assert!(pipe.reynolds_number(v) > 4000.0); // turbulent
//! let dp = pipe.pressure_drop(v);
//! assert!(dp > 0.0);
//! ```

#![forbid(unsafe_code)]

/// Fluid transport properties for incompressible pipe flow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FluidProperties {
    /// Density, kg/m³.
    pub density: f64,
    /// Dynamic viscosity, Pa·s.
    pub viscosity: f64,
}

/// A circular pipe with incompressible flow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PipeFlow {
    /// Internal diameter, m.
    pub diameter: f64,
    /// Length, m.
    pub length: f64,
    /// Absolute roughness, m (commercial steel ≈ 4.6e-5).
    pub roughness: f64,
    /// Fluid properties.
    pub fluid: FluidProperties,
}

impl PipeFlow {
    /// Creates a pipe (fluid defaults to water at 20 °C).
    #[must_use]
    pub fn new(diameter: f64, length: f64, roughness: f64) -> Self {
        Self {
            diameter,
            length,
            roughness,
            fluid: FluidProperties {
                density: 998.0,
                viscosity: 1.0e-3,
            },
        }
    }

    /// Sets the fluid.
    #[must_use]
    pub const fn with_fluid(mut self, fluid: FluidProperties) -> Self {
        self.fluid = fluid;
        self
    }

    /// Cross-sectional area, m².
    #[must_use]
    pub fn area(&self) -> f64 {
        std::f64::consts::FRAC_PI_4 * self.diameter * self.diameter
    }

    /// Mean velocity for a volumetric flow Q in m³/s.
    #[must_use]
    pub fn velocity(&self, flow_rate: f64) -> f64 {
        flow_rate / self.area()
    }

    /// Volumetric flow for a mean velocity, m³/s.
    #[must_use]
    pub fn flow_rate(&self, velocity: f64) -> f64 {
        velocity * self.area()
    }

    /// Reynolds number Re = ρvD/μ.
    #[must_use]
    pub fn reynolds_number(&self, velocity: f64) -> f64 {
        self.fluid.density * velocity * self.diameter / self.fluid.viscosity
    }

    /// Darcy friction factor.
    ///
    /// Laminar (Re < 2300): f = 64/Re. Turbulent: Colebrook-White solved
    /// by Newton iteration from the Swamee-Jain start (converges to
    /// machine precision in ~5 iterations).
    #[must_use]
    pub fn friction_factor(&self, velocity: f64) -> f64 {
        let re = self.reynolds_number(velocity);
        friction_factor(re, self.roughness / self.diameter)
    }

    /// Darcy-Weisbach pressure drop ΔP = f·(L/D)·ρv²/2, Pa, at mean
    /// velocity. Zero flow gives zero drop.
    #[must_use]
    pub fn pressure_drop(&self, velocity: f64) -> f64 {
        if velocity == 0.0 {
            return 0.0;
        }
        let f = self.friction_factor(velocity);
        f * (self.length / self.diameter) * self.fluid.density * velocity * velocity / 2.0
    }

    /// Pressure drop for a volumetric flow, Pa.
    #[must_use]
    pub fn pressure_drop_at_flow(&self, flow_rate: f64) -> f64 {
        self.pressure_drop(self.velocity(flow_rate))
    }

    /// Minor-loss pressure drop ΣK·ρv²/2, Pa.
    #[must_use]
    pub fn minor_losses(&self, velocity: f64, k_total: f64) -> f64 {
        k_total * self.fluid.density * velocity * velocity / 2.0
    }

    /// Total pressure drop (friction + minor losses), Pa.
    #[must_use]
    pub fn total_pressure_drop(&self, velocity: f64, k_total: f64) -> f64 {
        self.pressure_drop(velocity) + self.minor_losses(velocity, k_total)
    }
}

/// Darcy friction factor from Reynolds number and relative roughness.
#[must_use]
pub fn friction_factor(reynolds: f64, relative_roughness: f64) -> f64 {
    if reynolds <= 0.0 {
        return f64::INFINITY;
    }
    if reynolds < 2300.0 {
        return 64.0 / reynolds;
    }
    // Colebrook-White: 1/√f = −2·log10(ε/3.7D + 2.51/(Re·√f)).
    // Newton iteration on x = 1/√f:
    // g(x) = −2·log10(ε/3.7D + 2.51·x/Re) − x = 0.
    let mut x = swamee_jain(reynolds, relative_roughness).recip().sqrt();
    for _ in 0..20 {
        let term = relative_roughness / 3.7 + 2.51 * x / reynolds;
        let g = -2.0 * term.log10() - x;
        // dg/dx = −2/(ln10)·(2.51/Re)/term − 1
        let dg = -2.0 / std::f64::consts::LN_10 * (2.51 / reynolds) / term - 1.0;
        let step = g / dg;
        x -= step;
        if step.abs() < 1e-14 {
            break;
        }
    }
    1.0 / (x * x)
}

/// Swamee-Jain explicit friction-factor approximation.
#[must_use]
pub fn swamee_jain(reynolds: f64, relative_roughness: f64) -> f64 {
    if reynolds <= 0.0 {
        return f64::INFINITY;
    }
    0.25 / (relative_roughness / 3.7 + 5.74 / reynolds.powf(0.9))
        .log10()
        .powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn laminar_matches_hagen_poiseuille() {
        // Analytical: ΔP = 128·μ·L·Q/(π·D⁴).
        let mut pipe = PipeFlow::new(0.02, 10.0, 0.0);
        pipe.fluid = FluidProperties {
            density: 1000.0,
            viscosity: 0.001,
        };
        let velocity = 0.05; // Re = 1000 → laminar
        assert!(pipe.reynolds_number(velocity) < 2300.0);
        let dp = pipe.pressure_drop(velocity);
        let q = pipe.flow_rate(velocity);
        let analytic = 128.0 * 0.001 * 10.0 * q / (std::f64::consts::PI * 0.02f64.powi(4));
        assert!((dp - analytic).abs() < 1e-9 + 1e-9 * analytic);
    }

    #[test]
    fn laminar_friction_factor_is_64_over_re() {
        assert!((friction_factor(1000.0, 0.001) - 64.0 / 1000.0).abs() < 1e-12);
    }

    #[test]
    fn turbulent_colebrook_matches_swamee_jain_start() {
        // Smooth pipe at Re = 1e5: Colebrook f ≈ 0.018 (±10%).
        let f = friction_factor(1e5, 0.0);
        assert!((f - 0.018).abs() < 0.002, "f = {f}");
        // Rough pipe at high Re approaches the fully-rough asymptote
        // (within 0.5% at Re = 1e7).
        let f_rough = friction_factor(1e7, 0.05);
        let fully_rough = 1.0 / (-2.0 * (0.05_f64 / 3.7).log10()).powi(2);
        assert!(
            (f_rough - fully_rough).abs() < 5e-3 * fully_rough,
            "f_rough = {f_rough}, fully_rough = {fully_rough}"
        );
    }

    #[test]
    fn newton_colebrook_satisfies_the_equation() {
        let re = 1e5;
        let eps = 0.001;
        let f = friction_factor(re, eps);
        let lhs = 1.0 / f.sqrt();
        let rhs = -2.0 * (eps / 3.7 + 2.51 / (re * f.sqrt())).log10();
        assert!((lhs - rhs).abs() < 1e-8, "lhs={lhs} rhs={rhs}");
    }

    #[test]
    fn zero_flow_gives_zero_drop() {
        let pipe = PipeFlow::new(0.1, 10.0, 0.0);
        assert_eq!(pipe.pressure_drop(0.0), 0.0);
        assert_eq!(pipe.total_pressure_drop(0.0, 5.0), 0.0);
    }
}
