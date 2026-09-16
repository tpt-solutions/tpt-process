//! Thermodynamic property framework.
//!
//! `tpt-proc-thermo-core` defines the pieces every thermodynamic model
//! shares:
//!
//! - [`Component`] — pure-component constants (critical properties,
//!   acentric factor, ideal-gas heat capacity correlation);
//! - [`CpCorrelation`] — ideal-gas heat capacity with analytic enthalpy /
//!   entropy integrals;
//! - the [`EquationOfStateModel`] and [`ActivityCoefficientModel`] traits —
//!   the contracts `tpt-proc-thermo-eos` and `tpt-proc-thermo-activity`
//!   implement;
//! - [`PropertyPackage`] — the concrete package tying components + EOS +
//!   activity model together and implementing
//!   [`tpt_proc_core::PropertyPackage`], so the flash solver and flowsheet
//!   engine can consume it.
//!
//! All quantities are SI: T in K, P in Pa, molar amounts in mol, energies
//! in J.

#![forbid(unsafe_code)]

pub mod component;
pub mod methods;
pub mod package;
pub mod traits;

pub use component::{Component, CpCorrelation};
pub use methods::{Phase, PhaseSelection, PropertyMethods};
pub use package::PropertyPackage;
pub use traits::{ActivityCoefficientModel, EquationOfStateModel, ThermoError};

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, ThermoError>;

/// Universal gas constant, J/(mol·K) (CODATA 2018).
pub const R: f64 = 8.314_462_618;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gas_constant_value() {
        assert!((R - 8.314462618).abs() < 1e-9);
    }
}
