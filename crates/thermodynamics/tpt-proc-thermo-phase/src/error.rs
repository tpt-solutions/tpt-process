//! Flash result and error types.

use tpt_proc_core::Composition;
use tpt_proc_thermo_core::ThermoError;

/// Result of a converged flash calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct FlashResult {
    /// Equilibrium temperature, K.
    pub temperature: f64,
    /// Equilibrium pressure, Pa.
    pub pressure: f64,
    /// Molar vapor fraction β ∈ [0, 1].
    pub vapor_fraction: f64,
    /// Liquid-phase mole fractions.
    pub liquid_composition: Composition,
    /// Vapor-phase mole fractions.
    pub vapor_composition: Composition,
    /// Equilibrium ratios Kᵢ = yᵢ/xᵢ at convergence.
    pub k_values: Vec<f64>,
    /// K-value iterations performed (0 for single-phase short-circuits).
    pub iterations: u32,
    /// True when the state is degenerate (pure component at saturation):
    /// the vapor fraction is not unique and the reported value is a guarded
    /// midpoint.
    pub degenerate: bool,
}

/// Flash calculation errors.
#[derive(Clone, PartialEq, Debug)]
pub enum FlashError {
    /// The K-value loop hit its iteration cap without converging.
    NotConverged {
        /// Iteration cap that was reached.
        iterations: u32,
    },
    /// Feed composition or specifications were invalid.
    InvalidInput(String),
    /// The underlying property package failed.
    Thermo(ThermoError),
}

impl std::fmt::Display for FlashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConverged { iterations } => {
                write!(f, "flash did not converge in {iterations} iterations")
            }
            Self::InvalidInput(m) => write!(f, "invalid flash input: {m}"),
            Self::Thermo(e) => write!(f, "property package failure: {e}"),
        }
    }
}

impl std::error::Error for FlashError {}

impl From<ThermoError> for FlashError {
    fn from(e: ThermoError) -> Self {
        Self::Thermo(e)
    }
}
