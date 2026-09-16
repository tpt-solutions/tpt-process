//! Pure-component data and ideal-gas heat capacity correlations.

use crate::{Result, ThermoError};

/// Ideal-gas heat capacity correlation.
///
/// # DIPPR-127 form
///
/// `Cp/R = A + B·T + C·T² + D·T³ + E/T²` with T in K, giving J/(mol·K)
/// after multiplying by R. All five coefficients are supplied; set unused
/// ones to 0.
///
/// Coefficients shipped in `tpt-proc-thermo-database` are fitted to DIPPR /
/// NIST tabulations (Smith–Van Ness–Abboss Appendix C style); they are
/// documented as approximate and can be overridden by the user.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CpCorrelation {
    /// Constant term.
    pub a: f64,
    /// Linear coefficient, 1/K.
    pub b: f64,
    /// Quadratic coefficient, 1/K².
    pub c: f64,
    /// Cubic coefficient, 1/K³.
    pub d: f64,
    /// Inverse-square coefficient, K².
    pub e: f64,
}

impl CpCorrelation {
    /// Builds a DIPPR-127 correlation from the five coefficients.
    #[must_use]
    pub const fn dippr127(a: f64, b: f64, c: f64, d: f64, e: f64) -> Self {
        Self { a, b, c, d, e }
    }

    /// Temperature-independent heat capacity, J/(mol·K).
    #[must_use]
    pub const fn constant(cp: f64) -> Self {
        Self {
            a: cp / crate::R,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            e: 0.0,
        }
    }

    /// Ideal-gas heat capacity Cp⁰ at `temperature`, J/(mol·K).
    #[must_use]
    pub fn cp_ig(&self, temperature: f64) -> f64 {
        let t = temperature;
        crate::R * (self.a + self.b * t + self.c * t * t + self.d * t * t * t + self.e / (t * t))
    }

    /// Ideal-gas enthalpy change h⁰(T) − h⁰(T_ref), J/mol (analytic
    /// integral of Cp dT).
    #[must_use]
    pub fn enthalpy_ig(&self, temperature: f64, t_ref: f64) -> f64 {
        let (t, t0) = (temperature, t_ref);
        crate::R
            * (self.a * (t - t0)
                + self.b * (t * t - t0 * t0) / 2.0
                + self.c * (t * t * t - t0 * t0 * t0) / 3.0
                + self.d * (t * t * t * t - t0 * t0 * t0 * t0) / 4.0
                - self.e * (1.0 / t - 1.0 / t0))
    }

    /// Ideal-gas entropy change s⁰(T) − s⁰(T_ref) (without pressure
    /// term), J/(mol·K) (analytic integral of Cp/T dT).
    #[must_use]
    pub fn entropy_ig(&self, temperature: f64, t_ref: f64) -> f64 {
        let (t, t0) = (temperature, t_ref);
        crate::R
            * (self.a * (t / t0).ln()
                + self.b * (t - t0)
                + self.c * (t * t - t0 * t0) / 2.0
                + self.d * (t * t * t - t0 * t0 * t0) / 3.0
                - self.e * (1.0 / (t * t) - 1.0 / (t0 * t0)) / 2.0)
    }
}

/// A pure chemical component with its physical constants.
///
/// Molecular weight is kg/mol; temperatures in K; pressures in Pa; critical
/// volume in m³/mol.
#[derive(Clone, Debug, PartialEq)]
pub struct Component {
    /// Human-readable name (unique within a database).
    pub name: String,
    /// CAS registry number.
    pub cas_number: String,
    /// Molecular weight, kg/mol.
    pub molecular_weight: f64,
    /// Critical temperature, K.
    pub critical_temperature: f64,
    /// Critical pressure, Pa.
    pub critical_pressure: f64,
    /// Critical molar volume, m³/mol.
    pub critical_volume: f64,
    /// Pitzer acentric factor ω.
    pub acentric_factor: f64,
    /// Normal boiling point (1 atm), K.
    pub normal_boiling_point: f64,
    /// Ideal-gas heat capacity correlation.
    pub ideal_gas_cp: CpCorrelation,
}

impl Component {
    /// Creates a component; convenience for the database and tests.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        cas_number: &str,
        molecular_weight: f64,
        critical_temperature: f64,
        critical_pressure: f64,
        critical_volume: f64,
        acentric_factor: f64,
        normal_boiling_point: f64,
        ideal_gas_cp: CpCorrelation,
    ) -> Self {
        Self {
            name: name.to_string(),
            cas_number: cas_number.to_string(),
            molecular_weight,
            critical_temperature,
            critical_pressure,
            critical_volume,
            acentric_factor,
            normal_boiling_point,
            ideal_gas_cp,
        }
    }

    /// Reduced temperature Tr = T/Tc.
    #[must_use]
    pub fn reduced_temperature(&self, temperature: f64) -> f64 {
        temperature / self.critical_temperature
    }

    /// Reduced pressure Pr = P/Pc.
    #[must_use]
    pub fn reduced_pressure(&self, pressure: f64) -> f64 {
        pressure / self.critical_pressure
    }

    /// Compressibility factor at the critical point, Zc = Pc·Vc/(R·Tc).
    #[must_use]
    pub fn critical_compressibility(&self) -> f64 {
        self.critical_pressure * self.critical_volume / (crate::R * self.critical_temperature)
    }

    /// Validates the constants are finite and physically ordered.
    ///
    /// # Errors
    /// [`ThermoError::InvalidComponent`] when a constant is non-finite or
    /// non-physical.
    pub fn validate(&self) -> Result<()> {
        let checks = [
            (self.molecular_weight, "molecular weight"),
            (self.critical_temperature, "critical temperature"),
            (self.critical_pressure, "critical pressure"),
            (self.critical_volume, "critical volume"),
            (self.normal_boiling_point, "normal boiling point"),
        ];
        for (value, label) in checks {
            if !value.is_finite() || value <= 0.0 {
                return Err(ThermoError::InvalidComponent(format!(
                    "{}: {} must be finite and > 0",
                    self.name, label
                )));
            }
        }
        if self.critical_temperature <= self.normal_boiling_point {
            return Err(ThermoError::InvalidComponent(format!(
                "{}: Tc must exceed Tb",
                self.name
            )));
        }
        if !(-1.0..2.5).contains(&self.acentric_factor) {
            return Err(ThermoError::InvalidComponent(format!(
                "{}: acentric factor {} outside plausible range",
                self.name, self.acentric_factor
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cp(a: f64, b: f64, c: f64, d: f64, e: f64) -> CpCorrelation {
        CpCorrelation::dippr127(a, b, c, d, e)
    }

    #[test]
    fn water_cp_ig_matches_reference_points() {
        // SVNA-style water-vapor coefficients.
        let water = cp(3.470, 1.450e-3, 0.0, 0.0, 0.121e5);
        // At 300 K: Cp ≈ 33.6 J/(mol·K).
        assert!((water.cp_ig(300.0) - 33.58).abs() < 0.1);
        // At 800 K: Cp ≈ 38.7 J/(mol·K).
        assert!((water.cp_ig(800.0) - 38.66).abs() < 0.2);
    }

    #[test]
    fn enthalpy_integral_is_consistent_with_numeric() {
        // Compare the analytic enthalpy integral to a fine Simpson rule.
        let c = cp(4.0, 1.0e-3, -0.2e-6, 0.0, 0.5e5);
        let analytic = c.enthalpy_ig(600.0, 300.0);
        let n = 100_000;
        let h = (600.0 - 300.0) / f64::from(n);
        let f = |t: f64| c.cp_ig(t);
        let mut numeric = f(300.0) + f(600.0);
        for i in 1..n {
            let weight = if i % 2 == 0 { 2.0 } else { 4.0 };
            numeric += weight * f(300.0 + f64::from(i) * h);
        }
        numeric *= h / 3.0;
        assert!((analytic - numeric).abs() < 1e-6 * analytic.abs());
    }

    #[test]
    fn entropy_integral_is_consistent_with_numeric() {
        let c = cp(3.5, 0.8e-3, 0.0, 0.0, 0.2e5);
        let analytic = c.entropy_ig(500.0, 300.0);
        let n = 100_000;
        let h = (500.0 - 300.0) / f64::from(n);
        let f = |t: f64| c.cp_ig(t) / t;
        let mut numeric = f(300.0) + f(500.0);
        for i in 1..n {
            let weight = if i % 2 == 0 { 2.0 } else { 4.0 };
            numeric += weight * f(300.0 + f64::from(i) * h);
        }
        numeric *= h / 3.0;
        assert!((analytic - numeric).abs() < 1e-6 * analytic.abs().max(1.0));
    }

    #[test]
    fn constant_cp_correlation() {
        let c = CpCorrelation::constant(75.3);
        assert!((c.cp_ig(400.0) - 75.3).abs() < 1e-12);
        assert!((c.enthalpy_ig(400.0, 300.0) - 75.3 * 100.0).abs() < 1e-9);
    }

    #[test]
    fn component_validation() {
        let water = Component::new(
            "water",
            "7732-18-5",
            18.015e-3,
            647.096,
            22.064e6,
            55.9e-6,
            0.344,
            373.124,
            cp(3.470, 1.450e-3, 0.0, 0.0, 0.121e5),
        );
        assert!(water.validate().is_ok());
        assert!(water.critical_compressibility() > 0.0);

        let mut broken = water.clone();
        broken.critical_temperature = 100.0; // below Tb
        assert!(broken.validate().is_err());
    }
}
