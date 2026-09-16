//! Model contracts implemented by `tpt-proc-thermo-eos` and
//! `tpt-proc-thermo-activity`.
//!
//! Keeping these traits here (rather than in the implementing crates) lets
//! [`crate::PropertyPackage`] compose models without dependency cycles, and
//! lets the flash solver accept any package built from any model pair.

use crate::{PhaseSelection, Result};
use tpt_proc_core::Composition;

/// Errors raised by thermodynamic models.
#[derive(Clone, PartialEq, Debug)]
pub enum ThermoError {
    /// Input constants or state were invalid.
    InvalidComponent(String),
    /// The requested state is outside the model's domain.
    StateOutOfDomain(String),
    /// A numerical procedure failed (no root, no convergence).
    Numeric(String),
    /// The model does not implement the requested property.
    Unsupported(String),
}

impl std::fmt::Display for ThermoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidComponent(m) => write!(f, "invalid component: {m}"),
            Self::StateOutOfDomain(m) => write!(f, "state out of domain: {m}"),
            Self::Numeric(m) => write!(f, "numerical failure: {m}"),
            Self::Unsupported(m) => write!(f, "unsupported: {m}"),
        }
    }
}

impl std::error::Error for ThermoError {}

/// A volumetric/fugacity model (cubic EOS, ideal gas, …).
///
/// Implementations must be pure, thread-safe, and deterministic.
pub trait EquationOfStateModel: Send + Sync {
    /// Model name (diagnostics).
    fn name(&self) -> &str;

    /// Number of components the model is built for.
    fn num_components(&self) -> usize;

    /// Natural-log fugacity coefficients ln φᵢ for the selected phase.
    ///
    /// # Errors
    /// [`ThermoError::StateOutOfDomain`] when T/P are unphysical;
    /// [`ThermoError::Numeric`] when the root problem has no usable root.
    fn ln_fugacity_coefficients(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        selection: PhaseSelection,
    ) -> Result<Vec<f64>>;

    /// Selected compressibility factor Z at the state point.
    ///
    /// # Errors
    /// See [`EquationOfStateModel::ln_fugacity_coefficients`].
    fn compressibility(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        selection: PhaseSelection,
    ) -> Result<f64>;

    /// Molar enthalpy departure (h − h⁰), J/mol, for the selected phase.
    ///
    /// # Errors
    /// See [`EquationOfStateModel::ln_fugacity_coefficients`].
    fn enthalpy_departure(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        selection: PhaseSelection,
    ) -> Result<f64>;

    /// Molar entropy departure (s − s⁰), J/(mol·K), for the selected phase.
    ///
    /// # Errors
    /// See [`EquationOfStateModel::ln_fugacity_coefficients`].
    fn entropy_departure(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        selection: PhaseSelection,
    ) -> Result<f64>;

    /// Pure-component vapor pressure at `temperature`, Pa, via fugacity
    /// equality (valid below Tc).
    ///
    /// # Errors
    /// [`ThermoError::StateOutOfDomain`] at/above Tc or below ~0.4·Tc where
    /// the cubic saturation pressure loses meaning.
    fn pure_vapor_pressure(&self, component_index: usize, temperature: f64) -> Result<f64>;
}

/// A liquid excess-Gibbs (activity coefficient) model.
pub trait ActivityCoefficientModel: Send + Sync {
    /// Model name (diagnostics).
    fn name(&self) -> &str;

    /// Number of components the model is parameterized for.
    fn num_components(&self) -> usize;

    /// Natural-log activity coefficients ln γᵢ at composition `x` and
    /// temperature `t`.
    ///
    /// # Errors
    /// [`ThermoError::InvalidComponent`] when `x` has the wrong length.
    fn ln_gamma(&self, x: &[f64], temperature: f64) -> Result<Vec<f64>>;
}
