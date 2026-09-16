//! UNIQUAC activity coefficients (Abrams & Prausnitz, 1975).

use std::result::Result;

use tpt_proc_thermo_core::{ActivityCoefficientModel, ThermoError};

use crate::{validate_composition, PairMap};
use tpt_proc_thermo_core::R;

/// UNIQUAC parameters.
///
/// `u_pair` holds (u_ij − u_jj) energies in J/mol per ordered pair (zero
/// diagonal). `r` (volume) and `q` (surface) are pure-component UNIQUAC
/// constants; defaults of 1.0 reproduce an ideal combinatorial term.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct UniquacParams {
    /// (u_ij − u_jj), J/mol, per ordered pair (i, j).
    pub u_pair: PairMap,
    /// Volume parameters rᵢ.
    pub r: Vec<f64>,
    /// Surface parameters qᵢ.
    pub q: Vec<f64>,
}

impl UniquacParams {
    /// Single-binary convenience (r = q = 1 for both components).
    #[must_use]
    pub fn binary(u_01: f64, u_10: f64) -> Self {
        let mut params = Self {
            u_pair: PairMap::new(),
            r: vec![1.0, 1.0],
            q: vec![1.0, 1.0],
        };
        params.u_pair.insert((0, 1), u_01);
        params.u_pair.insert((1, 0), u_10);
        params
    }
}

/// Coordination number used by the combinatorial term.
const Z: f64 = 10.0;

/// The UNIQUAC model.
#[derive(Clone, Debug, PartialEq)]
pub struct Uniquac {
    num_components: usize,
    params: UniquacParams,
}

impl Uniquac {
    /// Builds a UNIQUAC model over `num_components` components with unit
    /// r/q defaults.
    #[must_use]
    pub fn new(num_components: usize) -> Self {
        Self {
            num_components,
            params: UniquacParams {
                u_pair: PairMap::new(),
                r: vec![1.0; num_components],
                q: vec![1.0; num_components],
            },
        }
    }

    /// Convenience constructor from parameters.
    #[must_use]
    pub fn from_params(num_components: usize, params: UniquacParams) -> Self {
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
        if x.len() != self.num_components
            || self.params.r.len() != x.len()
            || self.params.q.len() != x.len()
        {
            return Err(ThermoError::InvalidComponent(
                "composition/r/q lengths must all match the model size".into(),
            ));
        }
        let n = x.len();
        let rt = R * temperature;
        let r = &self.params.r;
        let q = &self.params.q;

        let sum_rx: f64 = x.iter().zip(r).map(|(xi, ri)| xi * ri).sum();
        let sum_qx: f64 = x.iter().zip(q).map(|(xi, qi)| xi * qi).sum();
        let phi: Vec<f64> = x.iter().zip(r).map(|(xi, ri)| xi * ri / sum_rx).collect();
        let theta: Vec<f64> = x.iter().zip(q).map(|(xi, qi)| xi * qi / sum_qx).collect();
        let l: Vec<f64> = (0..n)
            .map(|i| (Z / 2.0) * (r[i] - q[i]) - (r[i] - 1.0))
            .collect();
        let sum_xl: f64 = x.iter().zip(&l).map(|(xi, li)| xi * li).sum();

        // τ_ij = exp(−(u_ij − u_jj)/(RT)); τ_ii = 1.
        let tau = |i: usize, j: usize| -> f64 {
            if i == j {
                1.0
            } else {
                (-self.params.u_pair.get(&(i, j)).copied().unwrap_or(0.0) / rt).exp()
            }
        };

        let mut ln_gamma = vec![0.0; n];
        for (i, ln_gi) in ln_gamma.iter_mut().enumerate().take(n) {
            // Guard exact zeros so zero-fraction components get their
            // finite infinite-dilution limit rather than NaN.
            let x_i = x[i].max(1e-12);
            // Combinatorial term.
            let combinatorial = (phi[i] / x_i).ln()
                + (Z / 2.0) * q[i] * (theta[i].max(1e-300) / phi[i].max(1e-300)).ln()
                + l[i]
                - (phi[i] / x_i) * sum_xl;

            // Residual term.
            let s_i: f64 = (0..n).map(|k| theta[k] * tau(k, i)).sum();
            let mut inner = 0.0;
            for k in 0..n {
                let s_k: f64 = (0..n).map(|m| theta[m] * tau(m, k)).sum();
                inner += theta[k] * tau(i, k) / s_k.max(1e-300);
            }
            let residual = q[i] * (1.0 - s_i.max(1e-300).ln() - inner);
            *ln_gi = combinatorial + residual;
        }
        Ok(ln_gamma)
    }
}

impl ActivityCoefficientModel for Uniquac {
    fn name(&self) -> &str {
        "UNIQUAC"
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

    #[test]
    fn zero_u_with_unit_rq_recovers_ideality() {
        let uniquac = Uniquac::new(2);
        let gamma = uniquac.gamma(&[0.4, 0.6], 340.0).unwrap();
        assert!((gamma[0] - 1.0).abs() < 1e-9, "γ0 = {}", gamma[0]);
        assert!((gamma[1] - 1.0).abs() < 1e-9, "γ1 = {}", gamma[1]);
    }

    #[test]
    fn pure_component_gamma_is_one_with_real_geometry() {
        // Acetone(0)/water(1)-style geometry with zero interactions.
        let params = UniquacParams {
            u_pair: PairMap::new(),
            r: vec![2.5735, 0.92],
            q: vec![2.336, 1.4],
        };
        let uniquac = Uniquac::from_params(2, params);
        let g = uniquac.gamma(&[1.0, 0.0], 330.0).unwrap();
        assert!((g[0] - 1.0).abs() < 1e-6, "γ0 = {}", g[0]);
        let g = uniquac.gamma(&[0.0, 1.0], 330.0).unwrap();
        assert!((g[1] - 1.0).abs() < 1e-6, "γ1 = {}", g[1]);
    }

    #[test]
    fn nonzero_u_gives_finite_non_unity() {
        let uniquac = Uniquac::from_params(2, UniquacParams::binary(1200.0, -800.0));
        let gamma = uniquac.gamma(&[0.5, 0.5], 350.0).unwrap();
        assert!(gamma[0].is_finite() && gamma[0] > 0.0);
        assert!(gamma[1].is_finite() && gamma[1] > 0.0);
        assert!((gamma[0] - 1.0).abs() > 1e-4);
    }

    #[test]
    fn rejects_mismatched_geometry() {
        let mut params = UniquacParams::binary(100.0, 200.0);
        params.r = vec![1.0];
        let uniquac = Uniquac::from_params(2, params);
        assert!(uniquac.ln_gamma(&[0.5, 0.5], 350.0).is_err());
    }
}
