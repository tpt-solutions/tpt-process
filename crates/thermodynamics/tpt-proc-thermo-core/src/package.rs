//! The concrete [`PropertyPackage`] tying components, EOS, and activity
//! model together.

use std::sync::Arc;

use tpt_proc_core::{Composition, PhaseState};
// The core trait must be in scope: inherent-looking calls like
// `self.mixture_molecular_weight(..)` resolve through it.
use tpt_proc_core::PropertyPackage as _;

use crate::component::Component;
use crate::methods::{PhaseSelection, PropertyMethods};
use crate::traits::{ActivityCoefficientModel, EquationOfStateModel, ThermoError};
use crate::{Result, R};

/// Reference temperature for ideal-gas enthalpy/entropy datums, K.
pub const REFERENCE_TEMPERATURE: f64 = 298.15;

/// A configured thermodynamic property package.
///
/// Compose it from a component list plus optional models:
///
/// ```
/// use tpt_proc_thermo_core::{PropertyPackage, Component, CpCorrelation};
///
/// let water = Component::new(
///     "water", "7732-18-5", 18.015e-3,
///     647.096, 22.064e6, 55.9e-6, 0.344, 373.124,
///     CpCorrelation::dippr127(3.470, 1.450e-3, 0.0, 0.0, 0.121e5),
/// );
/// let package = PropertyPackage::new(vec![water]);
/// ```
///
/// Property evaluation rules:
/// - **Vapor**: ideal-gas enthalpy/entropy plus the EOS departure when an
///   EOS is attached; ideal-gas density when none is.
/// - **Liquid**: EOS departure and density when an EOS is attached, else
///   the Rackett corresponding-states liquid density.
/// - **K-values**: EOS fugacity ratio when an EOS is attached, else the
///   modified Raoult path `γᵢ·P_sat / P` (activity model or ideal γ = 1),
///   with `P_sat` from the EOS pure-component vapor pressure or the
///   Trouton–Clausius fallback.
#[derive(Clone, Default)]
pub struct PropertyPackage {
    components: Vec<Component>,
    eos: Option<Arc<dyn EquationOfStateModel>>,
    activity: Option<Arc<dyn ActivityCoefficientModel>>,
    reference_temperature: f64,
}

impl std::fmt::Debug for PropertyPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertyPackage")
            .field("components", &self.components.len())
            .field("eos", &self.eos.as_ref().map(|m| m.name()))
            .field("activity", &self.activity.as_ref().map(|m| m.name()))
            .finish()
    }
}

impl PropertyPackage {
    /// Creates a package over a component list (ideal behavior until models
    /// are attached).
    ///
    /// # Errors
    /// [`ThermoError::InvalidComponent`] when the list is empty or a
    /// component fails validation.
    pub fn new(components: Vec<Component>) -> Result<Self> {
        if components.is_empty() {
            return Err(ThermoError::InvalidComponent(
                "a package needs at least one component".into(),
            ));
        }
        for component in &components {
            component.validate()?;
        }
        Ok(Self {
            components,
            eos: None,
            activity: None,
            reference_temperature: REFERENCE_TEMPERATURE,
        })
    }

    /// Attaches an equation of state.
    #[must_use]
    pub fn with_eos(mut self, eos: Arc<dyn EquationOfStateModel>) -> Self {
        self.eos = Some(eos);
        self
    }

    /// Attaches an activity-coefficient model for the liquid phase.
    #[must_use]
    pub fn with_activity(mut self, activity: Arc<dyn ActivityCoefficientModel>) -> Self {
        self.activity = Some(activity);
        self
    }

    /// Overrides the ideal-gas enthalpy/entropy reference temperature, K.
    #[must_use]
    pub fn with_reference_temperature(mut self, t_ref: f64) -> Self {
        self.reference_temperature = t_ref;
        self
    }

    /// The component list.
    #[must_use]
    pub fn components(&self) -> &[Component] {
        &self.components
    }

    /// The attached EOS, if any.
    #[must_use]
    pub fn eos(&self) -> Option<&Arc<dyn EquationOfStateModel>> {
        self.eos.as_ref()
    }

    /// The attached activity model, if any.
    #[must_use]
    pub fn activity(&self) -> Option<&Arc<dyn ActivityCoefficientModel>> {
        self.activity.as_ref()
    }

    /// The property-method classification implied by the attached models.
    #[must_use]
    pub fn methods(&self) -> PropertyMethods {
        if self.eos.is_some() {
            PropertyMethods::CubicEos
        } else if self.activity.is_some() {
            PropertyMethods::ActivityCoefficient
        } else {
            PropertyMethods::IdealGas
        }
    }

    /// Ideal-gas mixture heat capacity, J/(mol·K).
    #[must_use]
    pub fn cp_ig_mixture(&self, composition: &Composition, temperature: f64) -> f64 {
        composition
            .as_slice()
            .iter()
            .zip(&self.components)
            .map(|(x, c)| x * c.ideal_gas_cp.cp_ig(temperature))
            .sum()
    }

    /// Pure-component saturation pressure, Pa, at `temperature`.
    ///
    /// EOS vapor pressure when available; otherwise the Trouton–Clausius
    /// correlation (ΔH_vap ≈ 87·Tb J/mol — Trouton's rule) anchored at the
    /// normal boiling point.
    #[must_use]
    pub fn vapor_pressure(&self, index: usize, temperature: f64) -> f64 {
        if let Some(eos) = &self.eos {
            if let Ok(p) = eos.pure_vapor_pressure(index, temperature) {
                return p;
            }
        }
        self.trouton_vapor_pressure(index, temperature)
    }

    /// Trouton–Clausius vapor pressure, Pa.
    #[must_use]
    pub fn trouton_vapor_pressure(&self, index: usize, temperature: f64) -> f64 {
        let component = &self.components[index];
        let t_b = component.normal_boiling_point;
        let dh_vap = 87.0 * t_b; // Trouton's rule, J/mol
        101_325.0 * (-(dh_vap / R) * (1.0 / temperature - 1.0 / t_b)).exp()
    }

    /// Liquid activity coefficients γᵢ (1 for ideal when no activity model
    /// is attached).
    ///
    /// # Errors
    /// Propagates the activity model's errors.
    pub fn activity_coefficients(&self, x: &Composition, temperature: f64) -> Result<Vec<f64>> {
        match &self.activity {
            Some(model) => Ok(model
                .ln_gamma(x.as_slice(), temperature)?
                .into_iter()
                .map(f64::exp)
                .collect()),
            None => Ok(vec![1.0; self.components.len()]),
        }
    }

    /// Mixture liquid density by the Rackett/Spencer–Danner corresponding
    /// states correlation, kg/m³ (used when no EOS supplies one).
    #[must_use]
    pub fn rackett_liquid_density(&self, composition: &Composition, temperature: f64) -> f64 {
        // Simple linear mixing of critical properties (documented
        // approximation for weakly non-ideal mixtures).
        let mut t_cm = 0.0;
        let mut p_cm = 0.0;
        let mut z_cm = 0.0;
        let mw = self.mixture_molecular_weight(composition);
        for (x, c) in composition.as_slice().iter().zip(&self.components) {
            t_cm += x * c.critical_temperature;
            p_cm += x * c.critical_pressure;
            z_cm += x * c.critical_compressibility().clamp(0.2, 0.3);
        }
        let t_r = temperature / t_cm;
        let exponent = 1.0 + (1.0 - t_r).powf(2.0 / 7.0);
        let v_molar = (R * t_cm / p_cm) * z_cm.powf(exponent); // m³/mol
        mw / v_molar
    }

    fn selection_for(phase: PhaseState) -> PhaseSelection {
        match phase {
            PhaseState::Vapor => PhaseSelection::Vapor,
            PhaseState::Liquid => PhaseSelection::Liquid,
            _ => PhaseSelection::Stable,
        }
    }
}

impl tpt_proc_core::PropertyPackage for PropertyPackage {
    fn name(&self) -> &str {
        if self.eos.is_some() && self.activity.is_some() {
            "eos+activity"
        } else if self.eos.is_some() {
            "eos"
        } else if self.activity.is_some() {
            "activity"
        } else {
            "ideal"
        }
    }

    fn num_components(&self) -> usize {
        self.components.len()
    }

    fn molecular_weights(&self) -> Vec<f64> {
        self.components.iter().map(|c| c.molecular_weight).collect()
    }

    fn fugacity_coefficients(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        phase: PhaseState,
    ) -> Vec<f64> {
        let selection = Self::selection_for(phase);
        match &self.eos {
            Some(eos) => eos
                .ln_fugacity_coefficients(composition, temperature, pressure, selection)
                .map(|ln_phi| ln_phi.iter().map(|l| l.exp()).collect())
                .unwrap_or_else(|_| vec![1.0; self.components.len()]),
            None => vec![1.0; self.components.len()],
        }
    }

    fn enthalpy(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        phase: PhaseState,
    ) -> f64 {
        // Ideal-gas part: mole-fraction mean of component integrals.
        let t_ref = self.reference_temperature;
        let h_ig: f64 = composition
            .as_slice()
            .iter()
            .zip(&self.components)
            .map(|(x, c)| x * c.ideal_gas_cp.enthalpy_ig(temperature, t_ref))
            .sum();

        let selection = Self::selection_for(phase);
        let h_dep = match &self.eos {
            Some(eos) => eos
                .enthalpy_departure(composition, temperature, pressure, selection)
                .unwrap_or(0.0),
            None => 0.0,
        };
        // Excess enthalpy of the activity model is neglected (documented);
        // h^E ≈ 0 for scoping calculations.
        h_ig + h_dep
    }

    fn entropy(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        phase: PhaseState,
    ) -> f64 {
        let t_ref = self.reference_temperature;
        let s_ig_pure: f64 = composition
            .as_slice()
            .iter()
            .zip(&self.components)
            .map(|(x, c)| x * c.ideal_gas_cp.entropy_ig(temperature, t_ref))
            .sum();
        // Ideal mixing entropy and (vapor) pressure departure.
        let mut s_mix = 0.0;
        for &x in composition.as_slice() {
            if x > 0.0 {
                s_mix -= x * x.ln();
            }
        }
        let pressure_term = match phase {
            PhaseState::Vapor => -(pressure / 101_325.0).ln(),
            _ => 0.0,
        };
        let selection = Self::selection_for(phase);
        let s_dep = match &self.eos {
            Some(eos) => eos
                .entropy_departure(composition, temperature, pressure, selection)
                .unwrap_or(0.0),
            None => 0.0,
        };
        s_ig_pure + R * s_mix + R * pressure_term + s_dep
    }

    fn density(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        phase: PhaseState,
    ) -> f64 {
        let mw = self.mixture_molecular_weight(composition);
        let selection = Self::selection_for(phase);
        if let Some(eos) = &self.eos {
            if let Ok(z) = eos.compressibility(composition, temperature, pressure, selection) {
                if z > 1e-6 {
                    return pressure * mw / (z * R * temperature);
                }
            }
        }
        match phase {
            PhaseState::Liquid => self.rackett_liquid_density(composition, temperature),
            _ => pressure * mw / (R * temperature), // ideal gas
        }
    }

    fn k_values(&self, composition: &Composition, temperature: f64, pressure: f64) -> Vec<f64> {
        if let Some(eos) = &self.eos {
            if let (Ok(ln_phi_l), Ok(ln_phi_v)) = (
                eos.ln_fugacity_coefficients(
                    composition,
                    temperature,
                    pressure,
                    PhaseSelection::Liquid,
                ),
                eos.ln_fugacity_coefficients(
                    composition,
                    temperature,
                    pressure,
                    PhaseSelection::Vapor,
                ),
            ) {
                return ln_phi_l
                    .iter()
                    .zip(&ln_phi_v)
                    .map(|(l, v)| (l - v).exp())
                    .collect();
            }
            // EOS failed (e.g. above Tc): fall through to Raoult path.
        }

        // Modified Raoult: K_i = gamma_i * Psat_i / P (Poynting neglected).
        let gammas = self
            .activity_coefficients(composition, temperature)
            .unwrap_or_else(|_| vec![1.0; self.components.len()]);
        gammas
            .iter()
            .enumerate()
            .map(|(i, gamma)| gamma * self.vapor_pressure(i, temperature) / pressure)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::CpCorrelation;

    fn water() -> Component {
        Component::new(
            "water",
            "7732-18-5",
            18.015e-3,
            647.096,
            22.064e6,
            55.9e-6,
            0.344,
            373.124,
            CpCorrelation::dippr127(3.470, 1.450e-3, 0.0, 0.0, 0.121e5),
        )
    }

    #[test]
    fn ideal_package_basics() {
        let pkg = PropertyPackage::new(vec![water()]).unwrap();
        assert_eq!(pkg.methods(), PropertyMethods::IdealGas);
        assert_eq!(pkg.num_components(), 1);
        let c = Composition::from_mole_fractions(&[1.0]).unwrap();
        // Ideal vapor density at 373 K, 1 atm ≈ 0.58 kg/m³.
        let rho = pkg.density(&c, 373.15, 101_325.0, PhaseState::Vapor);
        assert!((rho - 0.598).abs() < 0.01, "rho = {rho}");
        // Trouton vapor pressure at Tb returns ~1 atm.
        let psat = pkg.trouton_vapor_pressure(0, 373.124);
        assert!((psat - 101_325.0).abs() < 101_325.0 * 0.02);
    }

    #[test]
    fn rejects_invalid_components() {
        let mut bad = water();
        bad.critical_pressure = -1.0;
        assert!(PropertyPackage::new(vec![bad]).is_err());
    }

    #[test]
    fn cp_mixture_is_linear_in_mole_fractions() {
        let mut ethanol = water();
        ethanol.name = "ethanol".into();
        ethanol.ideal_gas_cp = CpCorrelation::constant(65.0);
        let pkg = PropertyPackage::new(vec![water(), ethanol]).unwrap();
        let c = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
        let cp_mix = pkg.cp_ig_mixture(&c, 400.0);
        // water Cp(400K) ≈ 34.3; mix mean with 65 → ≈ 49.6.
        assert!(cp_mix > 45.0 && cp_mix < 55.0, "cp_mix = {cp_mix}");
    }
}
