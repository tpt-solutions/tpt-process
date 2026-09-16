//! Wilson activity coefficients (1964).

use std::result::Result;

use tpt_proc_thermo_core::{ActivityCoefficientModel, ThermoError};

use crate::{validate_composition, PairMap};
use tpt_proc_thermo_core::R;

/// Wilson parameters.
///
/// `lambda` holds the binary interaction energies λ_ij − λ_jj in J/mol per
/// ordered pair (zero diagonal). `molar_volumes` are pure-liquid molar
/// volumes in m³/mol; they only set the scale of Λ_ij.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct WilsonParams {
    /// λ_ij − λ_jj, J/mol, per ordered pair (i, j).
    pub lambda: PairMap,
    /// Pure-liquid molar volumes, m³/mol, one per component.
    pub molar_volumes: Vec<f64>,
}

impl WilsonParams {
    /// Single-binary convenience: λ_01 and λ_10 in J/mol, identical molar
    /// volumes.
    #[must_use]
    pub fn binary(lambda_01: f64, lambda_10: f64) -> Self {
        let mut params = Self {
            lambda: PairMap::new(),
            molar_volumes: vec![1e-4, 1e-4],
        };
        params.lambda.insert((0, 1), lambda_01);
        params.lambda.insert((1, 0), lambda_10);
        params
    }
}

/// The Wilson model (miscible systems only — cannot predict LLE).
#[derive(Clone, Debug, PartialEq)]
pub struct Wilson {
    num_components: usize,
    params: WilsonParams,
}

impl Wilson {
    /// Builds a Wilson model over `num_components` components.
    #[must_use]
    pub fn new(num_components: usize) -> Self {
        Self {
            num_components,
            params: WilsonParams::default(),
        }
    }

    /// Convenience constructor from parameters.
    #[must_use]
    pub fn from_params(num_components: usize, params: WilsonParams) -> Self {
        Self {
            num_components,
            params,
        }
    }

    /// Activity coefficients γᵢ (not logs).
    ///
    /// # Errors
    /// [`ThermoError::InvalidComponent`] on malformed input.
    pub fn gamma(&self, x: &[f64], temperature: f64) -> Result<Vec<f64>, ThermoError> {
        self.ln_gamma_impl(x, temperature)
            .map(|ln| ln.iter().map(|g| g.exp()).collect())
    }

    fn ln_gamma_impl(&self, x: &[f64], temperature: f64) -> Result<Vec<f64>, ThermoError> {
        validate_composition(x)?;
        if x.len() != self.num_components {
            return Err(ThermoError::InvalidComponent(format!(
                "composition has {} components; model parameterized for {}",
                x.len(),
                self.num_components
            )));
        }
        if self.params.molar_volumes.len() != x.len() {
            return Err(ThermoError::InvalidComponent(
                "molar volume vector length must match composition".into(),
            ));
        }
        let n = x.len();
        let rt = R * temperature;

        // Λ_ij = (V_j/V_i)·exp(−λ_ij/(RT)).
        let lambda = |i: usize, j: usize| -> f64 {
            let v_i = self.params.molar_volumes[i];
            let v_j = self.params.molar_volumes[j];
            (v_j / v_i) * (-self.params.lambda.get(&(i, j)).copied().unwrap_or(0.0) / rt).exp()
        };

        let mut ln_gamma = vec![0.0; n];
        for (i, ln_gi) in ln_gamma.iter_mut().enumerate().take(n) {
            // s_i = Σ_j x_j Λ_ij
            let s_i: f64 = (0..n).map(|j| x[j] * lambda(i, j)).sum();
            let mut term2 = 0.0;
            for j in 0..n {
                // s_j = Σ_k x_k Λ_kj
                let s_j: f64 = (0..n).map(|k| x[k] * lambda(k, j)).sum();
                term2 += x[j] * lambda(j, i) / s_j.max(1e-300);
            }
            *ln_gi = -s_i.max(1e-300).ln() + 1.0 - term2;
        }
        Ok(ln_gamma)
    }
}

impl ActivityCoefficientModel for Wilson {
    fn name(&self) -> &str {
        "Wilson"
    }

    fn num_components(&self) -> usize {
        self.num_components
    }

    fn ln_gamma(&self, x: &[f64], temperature: f64) -> Result<Vec<f64>, ThermoError> {
        self.ln_gamma_impl(x, temperature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn methanol_water() -> Wilson {
        // Illustrative methanol/water parameters (J/mol).
        Wilson::from_params(2, WilsonParams::binary(1000.0, 1200.0))
    }

    #[test]
    fn zero_lambda_recovers_ideality() {
        let wilson = Wilson::from_params(2, WilsonParams::binary(0.0, 0.0));
        let gamma = wilson.gamma(&[0.3, 0.7], 340.0).unwrap();
        assert!((gamma[0] - 1.0).abs() < 1e-9);
        assert!((gamma[1] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn pure_component_gamma_is_one() {
        let wilson = methanol_water();
        let g1 = wilson.gamma(&[1.0, 0.0], 337.85).unwrap();
        assert!((g1[0] - 1.0).abs() < 1e-6, "γ = {}", g1[0]);
        let g2 = wilson.gamma(&[0.0, 1.0], 337.85).unwrap();
        assert!((g2[1] - 1.0).abs() < 1e-6, "γ = {}", g2[1]);
    }

    #[test]
    fn finite_dilution_gamma() {
        // Wilson always finite at infinite dilution (no LLE).
        let wilson = methanol_water();
        let g = wilson.gamma(&[1e-6, 1.0 - 1e-6], 337.85).unwrap();
        assert!(
            g[0].is_finite() && g[0] > 0.5 && g[0] < 10.0,
            "γ∞ = {}",
            g[0]
        );
    }

    #[test]
    fn rejects_volume_length_mismatch() {
        let mut params = WilsonParams::binary(500.0, 700.0);
        params.molar_volumes = vec![1e-4];
        let wilson = Wilson::from_params(2, params);
        assert!(wilson.ln_gamma(&[0.5, 0.5], 330.0).is_err());
    }
}
