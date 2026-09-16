//! The property-package contract.
//!
//! [`PropertyPackage`] is the single interface the flash solver, unit
//! operations, and flowsheet engine use to query thermodynamics. Concrete
//! implementations live in the `tpt-proc-thermo-*` crates (ideal solution,
//! cubic EOS, activity models) and implement this trait.

use crate::composition::Composition;
use crate::stream::PhaseState;

/// A thermodynamic property package over an ordered component list.
///
/// Implementations must be pure functions of their arguments (no hidden
/// mutable state) and safe to share across threads. Compositions are
/// index-aligned with [`PropertyPackage::num_components`].
pub trait PropertyPackage: Send + Sync {
    /// Human-readable name of the package (e.g. `"Peng-Robinson (PR)"`).
    fn name(&self) -> &str;

    /// Number of components this package is built for.
    fn num_components(&self) -> usize;

    /// Molecular weights in kg/mol, one per component.
    fn molecular_weights(&self) -> Vec<f64>;

    /// Mixture molecular weight, kg/mol, for the given composition.
    fn mixture_molecular_weight(&self, composition: &Composition) -> f64 {
        let weights = self.molecular_weights();
        composition
            .as_slice()
            .iter()
            .zip(weights)
            .map(|(x, w)| x * w)
            .sum()
    }

    /// Fugacity coefficients φᵢ of every component in the given phase.
    fn fugacity_coefficients(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        phase: PhaseState,
    ) -> Vec<f64>;

    /// Mixture molar enthalpy, J/mol, including any departure function.
    fn enthalpy(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        phase: PhaseState,
    ) -> f64;

    /// Mixture molar entropy, J/(mol·K), including any departure function.
    fn entropy(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        phase: PhaseState,
    ) -> f64;

    /// Density, kg/m³, of the given phase at the state point.
    fn density(
        &self,
        composition: &Composition,
        temperature: f64,
        pressure: f64,
        phase: PhaseState,
    ) -> f64;

    /// Equilibrium ratios Kᵢ = yᵢ/xᵢ at the state point.
    fn k_values(&self, composition: &Composition, temperature: f64, pressure: f64) -> Vec<f64>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Composition;

    /// Minimal ideal package used to test the default mixture MW.
    struct IdealTwoComponent;

    impl PropertyPackage for IdealTwoComponent {
        fn name(&self) -> &str {
            "ideal-test"
        }
        fn num_components(&self) -> usize {
            2
        }
        fn molecular_weights(&self) -> Vec<f64> {
            vec![18.015e-3, 32.042e-3]
        }
        fn fugacity_coefficients(
            &self,
            c: &Composition,
            _t: f64,
            _p: f64,
            _ph: PhaseState,
        ) -> Vec<f64> {
            vec![1.0; c.len()]
        }
        fn enthalpy(&self, _c: &Composition, _t: f64, _p: f64, _ph: PhaseState) -> f64 {
            0.0
        }
        fn entropy(&self, _c: &Composition, _t: f64, _p: f64, _ph: PhaseState) -> f64 {
            0.0
        }
        fn density(&self, _c: &Composition, _t: f64, _p: f64, _ph: PhaseState) -> f64 {
            1.0
        }
        fn k_values(&self, c: &Composition, _t: f64, _p: f64) -> Vec<f64> {
            vec![1.0; c.len()]
        }
    }

    #[test]
    fn default_mixture_molecular_weight() {
        let pkg = IdealTwoComponent;
        let c = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
        let mw = pkg.mixture_molecular_weight(&c);
        assert!((mw - 0.5 * (18.015e-3 + 32.042e-3)).abs() < 1e-12);
    }
}
