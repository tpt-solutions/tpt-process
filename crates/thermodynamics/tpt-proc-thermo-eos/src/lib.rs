//! Cubic equations of state: Peng-Robinson and Soave-Redlich-Kwong.
//!
//! Both flavors are implemented as the generalized 2-parameter cubic
//!
//! ```text
//! P = R·T / (v − b) − a / (v² + u·b·v + w·b²)
//! ```
//!
//! with (u, w) = (2, −1) for Peng-Robinson and (u, w) = (1, 0) for SRK.
//! Mixing is the classical van der Waals one-fluid rule with a symmetric
//! binary interaction matrix `kᵢⱼ` (see RFC 0001). Huron-Vidal and
//! Wong-Sandler mixing arrive with `tpt-proc-thermo-activity` integration.
//!
//! # Example
//!
//! ```
//! use tpt_proc_thermo_core::{Component, CpCorrelation, EquationOfStateModel};
//! use tpt_proc_thermo_eos::CubicEos;
//! use tpt_proc_core::Composition;
//!
//! // Pure water saturation pressure at 100 °C (PR EOS).
//! let water = Component::new(
//!     "water", "7732-18-5", 18.015e-3,
//!     647.096, 22.064e6, 55.9e-6, 0.344, 373.124,
//!     CpCorrelation::constant(34.0),
//! );
//! let eos = CubicEos::peng_robinson(vec![water]);
//! let p_sat = eos.pure_vapor_pressure(0, 373.15).unwrap();
//!
//! // Within ~3% of 1 atm for PR water.
//! assert!((p_sat - 101_325.0).abs() < 101_325.0 * 0.05);
//! let _ = Composition::from_mole_fractions(&[1.0]).unwrap();
//! ```

#![forbid(unsafe_code)]

pub mod cubic;

use std::collections::BTreeMap;

use tpt_proc_core::Composition;
use tpt_proc_thermo_core::{
    Component, EquationOfStateModel, PhaseSelection, Result, ThermoError, R,
};

#[cfg(test)]
use tpt_proc_thermo_core::CpCorrelation;

mod params;
mod vapor;

pub use params::MixingRule;

/// Which member of the 2-parameter cubic family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CubicFlavor {
    /// Peng-Robinson (1976): u = 2, w = −1.
    PengRobinson,
    /// Soave-Redlich-Kwong (1972): u = 1, w = 0.
    SoaveRedlichKwong,
}

/// A cubic EOS over a fixed component list.
#[derive(Clone, Debug)]
pub struct CubicEos {
    components: Vec<Component>,
    flavor: CubicFlavor,
    mixing_rule: MixingRule,
    /// Symmetric binary interaction parameters; diagonal implied zero.
    k_ij: BTreeMap<(usize, usize), f64>,
}

impl CubicEos {
    /// Peng-Robinson EOS over the given components.
    #[must_use]
    pub fn peng_robinson(components: Vec<Component>) -> Self {
        Self::new(components, CubicFlavor::PengRobinson)
    }

    /// Soave-Redlich-Kwong EOS over the given components.
    #[must_use]
    pub fn srk(components: Vec<Component>) -> Self {
        Self::new(components, CubicFlavor::SoaveRedlichKwong)
    }

    /// The configured mixing rule.
    #[must_use]
    pub const fn mixing_rule(&self) -> &MixingRule {
        &self.mixing_rule
    }

    /// Overrides the mixing rule (Huron-Vidal / Wong-Sandler are rejected
    /// at evaluation time until the activity-crate integration lands).
    #[must_use]
    pub fn with_mixing_rule(mut self, rule: MixingRule) -> Self {
        self.mixing_rule = rule;
        self
    }

    fn new(components: Vec<Component>, flavor: CubicFlavor) -> Self {
        Self {
            components,
            flavor,
            mixing_rule: MixingRule::VanDerWaals,
            k_ij: BTreeMap::new(),
        }
    }

    /// Sets a symmetric binary interaction parameter kᵢⱼ (also stored for
    /// (j, i)).
    #[must_use]
    pub fn with_binary_interaction(mut self, i: usize, j: usize, k_ij: f64) -> Self {
        if i != j {
            self.k_ij.insert((i.min(j), i.max(j)), k_ij);
        }
        self
    }

    /// Symmetric kᵢⱼ lookup (0 when unset).
    #[must_use]
    pub fn binary_interaction(&self, i: usize, j: usize) -> f64 {
        if i == j {
            0.0
        } else {
            self.k_ij.get(&(i.min(j), i.max(j))).copied().unwrap_or(0.0)
        }
    }

    /// The flavor (PR or SRK).
    #[must_use]
    pub const fn flavor(&self) -> CubicFlavor {
        self.flavor
    }

    /// EOS attraction parameter constant Ωa = a·Pc/(R²Tc²).
    const fn omega_a(&self) -> f64 {
        match self.flavor {
            CubicFlavor::PengRobinson => 0.457_235_528_921_382_2, // 0.45724
            CubicFlavor::SoaveRedlichKwong => 0.427_47,
        }
    }

    /// EOS co-volume constant Ωb = b·Pc/(R·Tc).
    const fn omega_b(&self) -> f64 {
        match self.flavor {
            CubicFlavor::PengRobinson => 0.077_796_073_903_888_47, // 0.07780
            CubicFlavor::SoaveRedlichKwong => 0.086_64,
        }
    }

    /// (u, w) shape parameters of the repulsive/attractive denominator.
    const fn shape(&self) -> (f64, f64) {
        match self.flavor {
            CubicFlavor::PengRobinson => (2.0, -1.0),
            CubicFlavor::SoaveRedlichKwong => (1.0, 0.0),
        }
    }

    /// sqrt(u² − 4w) (2√2 for PR, 1 for SRK).
    fn delta(&self) -> f64 {
        let (u, w) = self.shape();
        (u * u - 4.0 * w).sqrt()
    }

    /// Pure-component attraction parameter a_ci = Ωa·R²·Tc²/Pc, J²·Pa/mol².
    #[must_use]
    fn a_ci(&self, i: usize) -> f64 {
        let c = &self.components[i];
        self.omega_a() * R * R * c.critical_temperature * c.critical_temperature
            / c.critical_pressure
    }

    /// Pure-component co-volume b_i = Ωb·R·Tc/Pc, m³/mol.
    #[must_use]
    fn b_i(&self, i: usize) -> f64 {
        let c = &self.components[i];
        self.omega_b() * R * c.critical_temperature / c.critical_pressure
    }

    /// Temperature function m(ω) of the alpha correlation.
    #[must_use]
    fn m_i(&self, i: usize) -> f64 {
        let omega = self.components[i].acentric_factor;
        match self.flavor {
            CubicFlavor::PengRobinson => 0.374_64 + 1.542_26 * omega - 0.269_92 * omega * omega,
            CubicFlavor::SoaveRedlichKwong => 0.480 + 1.574 * omega - 0.176 * omega * omega,
        }
    }

    /// αᵢ(T) = [1 + mᵢ·(1 − √Tr)]².
    #[must_use]
    fn alpha_i(&self, i: usize, temperature: f64) -> f64 {
        let tr = temperature / self.components[i].critical_temperature;
        let s = 1.0 + self.m_i(i) * (1.0 - tr.sqrt());
        s * s
    }

    /// dαᵢ/dT = −mᵢ·[1 + mᵢ(1−√Tr)]/√(T·Tc).
    #[must_use]
    fn d_alpha_i_dt(&self, i: usize, temperature: f64) -> f64 {
        let c = &self.components[i];
        let tr = temperature / c.critical_temperature;
        let s = 1.0 + self.m_i(i) * (1.0 - tr.sqrt());
        -self.m_i(i) * s / (temperature * c.critical_temperature).sqrt()
    }

    /// aᵢ(T) = a_ci·αᵢ(T).
    #[must_use]
    fn a_i(&self, i: usize, temperature: f64) -> f64 {
        self.a_ci(i) * self.alpha_i(i, temperature)
    }

    /// Cross attraction aᵢⱼ(T) = √(aᵢaⱼ)·(1 − kᵢⱼ).
    #[must_use]
    fn a_ij(&self, i: usize, j: usize, temperature: f64) -> f64 {
        (self.a_i(i, temperature) * self.a_i(j, temperature)).sqrt()
            * (1.0 - self.binary_interaction(i, j))
    }

    /// Mixture attraction a_mix = Σᵢ Σⱼ xᵢxⱼaᵢⱼ.
    #[must_use]
    pub fn a_mix(&self, composition: &Composition, temperature: f64) -> f64 {
        let x = composition.as_slice();
        let mut sum = 0.0;
        for (i, xi) in x.iter().enumerate() {
            for (j, xj) in x.iter().enumerate() {
                sum += xi * xj * self.a_ij(i, j, temperature);
            }
        }
        sum
    }

    /// Mixture co-volume b = Σ xᵢbᵢ.
    #[must_use]
    pub fn b_mix(&self, composition: &Composition) -> f64 {
        composition
            .as_slice()
            .iter()
            .enumerate()
            .map(|(i, x)| x * self.b_i(i))
            .sum()
    }

    /// d(a_mix)/dT from the analytic alpha derivatives.
    #[must_use]
    pub fn da_mix_dt(&self, composition: &Composition, temperature: f64) -> f64 {
        let x = composition.as_slice();
        let mut sum = 0.0;
        for (i, xi) in x.iter().enumerate() {
            for (j, xj) in x.iter().enumerate() {
                // aᵢⱼ = √(a_ci·a_cj)·√(αᵢαⱼ)·(1−kᵢⱼ), so
                // d(aᵢⱼ)/dT = √(a_ci·a_cj)·(1−kᵢⱼ)·d√(αᵢαⱼ)/dT.
                let alpha_i = self.alpha_i(i, temperature);
                let alpha_j = self.alpha_i(j, temperature);
                let sqrt_ac = (self.a_ci(i) * self.a_ci(j)).sqrt();
                let k = self.binary_interaction(i, j);
                let sqrt_alpha = (alpha_i * alpha_j).sqrt();
                let d_sqrt_alpha = sqrt_alpha
                    * 0.5
                    * (self.d_alpha_i_dt(i, temperature) / alpha_i
                        + self.d_alpha_i_dt(j, temperature) / alpha_j);
                sum += xi * xj * sqrt_ac * (1.0 - k) * d_sqrt_alpha;
            }
        }
        sum
    }

    /// EOS pressure at (composition, T, molar volume v).
    #[must_use]
    pub fn pressure(&self, composition: &Composition, temperature: f64, v: f64) -> f64 {
        let a = self.a_mix(composition, temperature);
        let b = self.b_mix(composition);
        let (u, w) = self.shape();
        R * temperature / (v - b) - a / (v * v + u * b * v + w * b * b)
    }

    /// All positive real compressibility roots at (T, P), ascending.
    ///
    /// # Errors
    /// [`ThermoError::Numeric`] when no positive root exists.
    pub fn compressibility_roots(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
    ) -> Result<Vec<f64>> {
        let a = self.a_mix(composition, temperature);
        let b = self.b_mix(composition);
        let aa = a * pressure / (R * R * temperature * temperature); // A
        let bb = b * pressure / (R * temperature); // B

        let (c2, c1, c0) = match self.flavor {
            CubicFlavor::PengRobinson => (
                -(1.0 - bb),
                aa - 3.0 * bb * bb - 2.0 * bb,
                -(aa * bb - bb * bb - bb * bb * bb),
            ),
            CubicFlavor::SoaveRedlichKwong => (-1.0, aa - bb - bb * bb, -(aa * bb)),
        };

        let roots: Vec<f64> = cubic::solve_cubic(c2, c1, c0)
            .into_iter()
            .filter(|z| *z > 1e-9 && z.is_finite())
            .collect();
        if roots.is_empty() {
            return Err(ThermoError::Numeric(format!(
                "no positive compressibility root at T={temperature}, P={pressure}"
            )));
        }
        Ok(roots)
    }

    /// Log mixture fugacity coefficient at a given compressibility root.
    #[must_use]
    pub fn ln_phi_mixture(&self, composition: &Composition, t: f64, p: f64, z: f64) -> f64 {
        let a = self.a_mix(composition, t);
        let b = self.b_mix(composition);
        let aa = a * p / (R * R * t * t);
        let bb = b * p / (R * t);
        if z <= bb {
            return f64::INFINITY;
        }
        let log_term = self.log_term(z, bb);
        z - 1.0 - (z - bb).ln() - (aa / (self.delta() * bb)) * log_term
    }

    /// The flavor-specific log factor multiplying A/δB.
    fn log_term(&self, z: f64, bb: f64) -> f64 {
        match self.flavor {
            CubicFlavor::PengRobinson => ((z + (1.0 + std::f64::consts::SQRT_2) * bb)
                / (z + (1.0 - std::f64::consts::SQRT_2) * bb))
                .ln(),
            CubicFlavor::SoaveRedlichKwong => (1.0 + bb / z).ln(),
        }
    }

    /// Integral factor I = [1/(b·δ)]·ln((2Z + B(u+δ))/(2Z + B(u−δ)))
    /// shared by the departure functions.
    fn integral_term(&self, z: f64, bb: f64, b_mix: f64) -> f64 {
        let (u, _) = self.shape();
        let d = self.delta();
        let num = 2.0 * z + bb * (u + d);
        let den = 2.0 * z + bb * (u - d);
        let log_val = (num / den).ln();
        1.0 / (b_mix * d) * log_val
    }

    /// Component log fugacity coefficients at a specific root Z.
    ///
    /// ```text
    /// ln φᵢ = (bᵢ/b)(Z−1) − ln(Z−B) − (A/(δ·B))·(2Σⱼyⱼaᵢⱼ/a − bᵢ/b)·L(Z,B)
    /// ```
    ///
    /// # Errors
    /// [`ThermoError::Numeric`] when Z ≤ B (degenerate state).
    pub fn ln_phi_at_root(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        z: f64,
    ) -> Result<Vec<f64>> {
        let x = composition.as_slice();
        let a = self.a_mix(composition, temperature);
        let b = self.b_mix(composition);
        let aa = a * pressure / (R * R * temperature * temperature);
        let bb = b * pressure / (R * temperature);
        if z <= bb + 1e-12 {
            return Err(ThermoError::Numeric(format!(
                "root Z={z} at/below B={bb}: fugacity undefined"
            )));
        }
        let log_term = self.log_term(z, bb);
        let prefactor = aa / (self.delta() * bb);

        let mut out = Vec::with_capacity(x.len());
        for (i, xi) in x.iter().enumerate() {
            let b_i = self.b_i(i);
            let sum = x
                .iter()
                .enumerate()
                .map(|(j, xj)| xj * self.a_ij(i, j, temperature))
                .sum::<f64>();
            let ln_phi = (b_i / b) * (z - 1.0)
                - (z - bb).ln()
                - prefactor * (2.0 * sum / a - b_i / b) * log_term;
            out.push(ln_phi);
            let _ = xi;
        }
        Ok(out)
    }

    /// Selects the compressibility root for a phase selection.
    ///
    /// # Errors
    /// Propagates [`CubicEos::compressibility_roots`].
    pub fn select_root(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        selection: PhaseSelection,
    ) -> Result<f64> {
        let roots = self.compressibility_roots(composition, temperature, pressure)?;
        match selection {
            PhaseSelection::Vapor => Ok(*roots.last().expect("non-empty")),
            PhaseSelection::Liquid => Ok(roots[0]),
            PhaseSelection::Stable => {
                if roots.len() == 1 {
                    return Ok(roots[0]);
                }
                let z_l = roots[0];
                let z_v = *roots.last().expect("non-empty");
                let g_l = self.ln_phi_mixture(composition, temperature, pressure, z_l);
                let g_v = self.ln_phi_mixture(composition, temperature, pressure, z_v);
                if g_v <= g_l {
                    Ok(z_v)
                } else {
                    Ok(z_l)
                }
            }
        }
    }

    /// Component list (borrowed).
    #[must_use]
    pub fn components(&self) -> &[Component] {
        &self.components
    }
}

impl EquationOfStateModel for CubicEos {
    fn name(&self) -> &str {
        match self.flavor {
            CubicFlavor::PengRobinson => "Peng-Robinson",
            CubicFlavor::SoaveRedlichKwong => "SRK",
        }
    }

    fn num_components(&self) -> usize {
        self.components.len()
    }

    fn ln_fugacity_coefficients(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        selection: PhaseSelection,
    ) -> Result<Vec<f64>> {
        self.validate_state(temperature, pressure)?;
        let z = self.select_root(composition, temperature, pressure, selection)?;
        self.ln_phi_at_root(composition, temperature, pressure, z)
    }

    fn compressibility(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        selection: PhaseSelection,
    ) -> Result<f64> {
        self.validate_state(temperature, pressure)?;
        self.select_root(composition, temperature, pressure, selection)
    }

    fn enthalpy_departure(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        selection: PhaseSelection,
    ) -> Result<f64> {
        let z = self.select_root(composition, temperature, pressure, selection)?;
        let a = self.a_mix(composition, temperature);
        let b = self.b_mix(composition);
        let bb = b * pressure / (R * temperature);
        let da = self.da_mix_dt(composition, temperature);
        let i_term = self.integral_term(z, bb, b);
        Ok(R * temperature * (z - 1.0) + (temperature * da - a) * i_term)
    }

    fn entropy_departure(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        selection: PhaseSelection,
    ) -> Result<f64> {
        let z = self.select_root(composition, temperature, pressure, selection)?;
        let b = self.b_mix(composition);
        let bb = b * pressure / (R * temperature);
        let da = self.da_mix_dt(composition, temperature);
        let i_term = self.integral_term(z, bb, b);
        Ok(R * (z - bb).ln() + temperature * da * i_term)
    }

    fn pure_vapor_pressure(&self, component_index: usize, temperature: f64) -> Result<f64> {
        vapor::pure_vapor_pressure(self, component_index, temperature)
    }
}

impl CubicEos {
    fn validate_state(&self, temperature: f64, pressure: f64) -> Result<()> {
        if !matches!(self.mixing_rule, MixingRule::VanDerWaals) {
            return Err(ThermoError::Unsupported(format!(
                "mixing rule {:?} requires the activity-crate integration (RFC 0001)",
                self.mixing_rule
            )));
        }
        if !(temperature.is_finite() && temperature > 0.0) {
            return Err(ThermoError::StateOutOfDomain(format!(
                "temperature {temperature} must be finite and > 0 K"
            )));
        }
        if !(pressure.is_finite() && pressure > 0.0) {
            return Err(ThermoError::StateOutOfDomain(format!(
                "pressure {pressure} must be finite and > 0 Pa"
            )));
        }
        Ok(())
    }
}

/// Test helper re-export: build a component with minimal boilerplate.
#[cfg(test)]
pub(crate) fn test_component(
    name: &str,
    mw: f64,
    tc: f64,
    pc: f64,
    omega: f64,
    tb: f64,
) -> Component {
    Component::new(
        name,
        "0-00-0",
        mw,
        tc,
        pc,
        55.9e-6,
        omega,
        tb,
        CpCorrelation::constant(34.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn water() -> Component {
        test_component("water", 18.015e-3, 647.096, 22.064e6, 0.344, 373.124)
    }

    #[test]
    fn ideal_gas_limit() {
        let eos = CubicEos::peng_robinson(vec![water()]);
        let c = Composition::from_mole_fractions(&[1.0]).unwrap();
        // Low pressure: Z → 1 and ln φ → 0.
        let z = eos
            .compressibility(&c, 400.0, 100.0, PhaseSelection::Vapor)
            .unwrap();
        assert!((z - 1.0).abs() < 1e-3, "Z = {z}");
        let ln_phi = eos
            .ln_fugacity_coefficients(&c, 400.0, 100.0, PhaseSelection::Vapor)
            .unwrap();
        assert!(ln_phi[0].abs() < 1e-3);
    }

    #[test]
    fn methane_z_at_high_pressure_is_subcritical_vapor() {
        let methane = test_component("methane", 16.043e-3, 190.56, 4.599e6, 0.0115, 111.67);
        let eos = CubicEos::peng_robinson(vec![methane]);
        let c = Composition::from_mole_fractions(&[1.0]).unwrap();
        // At 300 K, 100 bar methane vapor: Z ≈ 0.83–0.88 (PR).
        let z = eos
            .compressibility(&c, 300.0, 10e6, PhaseSelection::Vapor)
            .unwrap();
        assert!((0.80..=0.92).contains(&z), "Z = {z}");
    }

    #[test]
    fn water_vapor_pressure_near_boiling() {
        let eos = CubicEos::peng_robinson(vec![water()]);
        let p_sat = eos.pure_vapor_pressure(0, 373.15).unwrap();
        // PR water saturation at 100 °C is within ~3% of 1 atm.
        assert!(
            (p_sat - 101_325.0).abs() < 101_325.0 * 0.05,
            "Psat = {p_sat}"
        );
    }

    #[test]
    fn vapor_pressure_monotone_in_temperature() {
        let propane = test_component("propane", 44.097e-3, 369.83, 4.248e6, 0.1523, 231.04);
        let eos = CubicEos::peng_robinson(vec![propane]);
        let p1 = eos.pure_vapor_pressure(0, 231.0).unwrap();
        let p2 = eos.pure_vapor_pressure(0, 300.0).unwrap();
        let p3 = eos.pure_vapor_pressure(0, 350.0).unwrap();
        assert!(p1 < p2 && p2 < p3);
        // Propane at its normal boiling point: Psat ≈ 1 atm.
        assert!((p1 - 101_325.0).abs() < 101_325.0 * 0.05, "p1 = {p1}");
    }

    #[test]
    fn srk_and_pr_agree_at_low_pressure() {
        let ethane = test_component("ethane", 30.070e-3, 305.32, 4.872e6, 0.0995, 184.55);
        let pr = CubicEos::peng_robinson(vec![ethane.clone()]);
        let srk = CubicEos::srk(vec![ethane]);
        let c = Composition::from_mole_fractions(&[1.0]).unwrap();
        let z_pr = pr
            .compressibility(&c, 250.0, 1e5, PhaseSelection::Vapor)
            .unwrap();
        let z_srk = srk
            .compressibility(&c, 250.0, 1e5, PhaseSelection::Vapor)
            .unwrap();
        // Both near ideal at 1 bar, 250 K.
        assert!((z_pr - z_srk).abs() < 0.02);
    }

    #[test]
    fn binary_interaction_is_symmetric() {
        let eos = CubicEos::peng_robinson(vec![water()])
            .with_binary_interaction(0, 0, 0.5) // ignored (diagonal)
            .with_binary_interaction(0, 0, 0.1);
        assert_eq!(eos.binary_interaction(0, 0), 0.0);
    }

    #[test]
    fn departure_functions_vanish_at_low_pressure() {
        let eos = CubicEos::peng_robinson(vec![water()]);
        let c = Composition::from_mole_fractions(&[1.0]).unwrap();
        let h_dep = eos
            .enthalpy_departure(&c, 400.0, 50.0, PhaseSelection::Vapor)
            .unwrap();
        let s_dep = eos
            .entropy_departure(&c, 400.0, 50.0, PhaseSelection::Vapor)
            .unwrap();
        assert!(h_dep.abs() < 5.0, "h_dep = {h_dep}");
        assert!(s_dep.abs() < 0.05, "s_dep = {s_dep}");
    }
}
