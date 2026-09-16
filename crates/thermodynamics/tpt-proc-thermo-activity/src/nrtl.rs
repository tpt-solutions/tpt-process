//! NRTL (non-random two-liquid) activity coefficients.

use std::result::Result;

use tpt_proc_thermo_core::{ActivityCoefficientModel, ThermoError};

use crate::{validate_composition, PairMap};
use tpt_proc_thermo_core::R;

/// NRTL binary parameters.
///
/// `tau` entries are (g_ij − g_jj) energies in J/mol, stored per ordered
/// pair — τ_ij and τ_ji are independent (asymmetric); missing directions
/// default to zero and the diagonal is zero (pure-component limit). The
/// dimensionless τ used in the model is `tau_ij/(R·T)`. `alpha` entries
/// are symmetric non-randomness factors (typically 0.20–0.47); pairs
/// without an entry use [`NrtlParams::default_alpha`].
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NrtlParams {
    /// (g_ij − g_jj), J/mol, keyed by ordered pair (i, j).
    pub tau: PairMap,
    /// Non-randomness factors α_ij, keyed by unordered pair.
    pub alpha: PairMap,
    /// α used when a pair has no explicit entry.
    pub default_alpha: f64,
}

impl NrtlParams {
    /// Parameters for a single binary (0, 1): common α and both τ
    /// directions in J/mol.
    #[must_use]
    pub fn binary(alpha: f64, tau_01: f64, tau_10: f64) -> Self {
        let mut params = Self {
            tau: PairMap::new(),
            alpha: PairMap::new(),
            default_alpha: 0.30,
        };
        params.set(0, 1, alpha, tau_01, tau_10);
        params
    }

    /// Sets a full binary: α(i,j) and both τ directions (J/mol).
    pub fn set(&mut self, i: usize, j: usize, alpha: f64, tau_ij: f64, tau_ji: f64) {
        self.alpha.insert((i.min(j), i.max(j)), alpha);
        self.tau.insert((i, j), tau_ij);
        self.tau.insert((j, i), tau_ji);
    }

    fn tau_entry(&self, i: usize, j: usize) -> Option<f64> {
        self.tau.get(&(i, j)).copied()
    }

    fn alpha_of(&self, i: usize, j: usize) -> f64 {
        if i == j {
            0.0
        } else {
            self.alpha
                .get(&(i.min(j), i.max(j)))
                .copied()
                .unwrap_or(self.default_alpha)
        }
    }
}

/// The NRTL model.
#[derive(Clone, Debug, PartialEq)]
pub struct Nrtl {
    num_components: usize,
    params: NrtlParams,
}

impl Nrtl {
    /// Builds an NRTL model over `num_components` components.
    #[must_use]
    pub fn new(num_components: usize) -> Self {
        Self {
            num_components,
            params: NrtlParams::default(),
        }
    }

    /// Convenience constructor from a parameter set.
    #[must_use]
    pub fn from_params(num_components: usize, params: NrtlParams) -> Self {
        Self {
            num_components,
            params,
        }
    }

    /// Activity coefficients γᵢ (not logs).
    ///
    /// # Errors
    /// [`ThermoError::InvalidComponent`] on malformed composition.
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
        let n = x.len();
        let rt = R * temperature;

        // Dimensionless τ_ij and Boltzmann factors G_ij = exp(−α_ij·τ_ij).
        let mut tau = vec![vec![0.0; n]; n];
        let mut gmat = vec![vec![1.0; n]; n];
        for (i, tau_row) in tau.iter_mut().enumerate().take(n) {
            for (j, t_ij) in tau_row.iter_mut().enumerate().take(n) {
                *t_ij = self.params.tau_entry(i, j).unwrap_or(0.0) / rt;
                let alpha = self.params.alpha_of(i, j);
                gmat[i][j] = (-alpha * *t_ij).exp();
            }
        }
        // d[j] = Σ_k x_k·G_kj — shared denominators.
        let d: Vec<f64> = (0..n)
            .map(|j| (0..n).map(|k| x[k] * gmat[k][j]).sum())
            .collect();

        let mut ln_gamma = vec![0.0; n];
        for (i, ln_gi) in ln_gamma.iter_mut().enumerate().take(n) {
            let term1: f64 =
                (0..n).map(|j| x[j] * tau[j][i] * gmat[j][i]).sum::<f64>() / d[i].max(1e-300);
            let mut term2 = 0.0;
            for j in 0..n {
                let inner: f64 = (0..n).map(|m| x[m] * tau[m][j] * gmat[m][j]).sum();
                term2 +=
                    (x[j] * gmat[i][j] / d[j].max(1e-300)) * (tau[i][j] - inner / d[j].max(1e-300));
            }
            *ln_gi = term1 + term2;
        }
        Ok(ln_gamma)
    }
}

impl ActivityCoefficientModel for Nrtl {
    fn name(&self) -> &str {
        "NRTL"
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
    use crate::excess_gibbs_over_rt;

    fn ethanol_water() -> Nrtl {
        // Illustrative ethanol/water parameters (J/mol).
        Nrtl::from_params(2, NrtlParams::binary(0.30, 728.0, 385.0))
    }

    #[test]
    fn zero_tau_recovers_ideality() {
        let mut params = NrtlParams::default();
        params.set(0, 1, 0.3, 0.0, 0.0);
        let nrtl = Nrtl::from_params(2, params);
        let gamma = nrtl.gamma(&[0.5, 0.5], 350.0).unwrap();
        assert!((gamma[0] - 1.0).abs() < 1e-10);
        assert!((gamma[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn pure_component_gamma_is_one() {
        let nrtl = ethanol_water();
        let gamma = nrtl.gamma(&[1.0, 0.0], 351.45).unwrap();
        assert!((gamma[0] - 1.0).abs() < 1e-6, "γ0 = {}", gamma[0]);
        let gamma = nrtl.gamma(&[0.0, 1.0], 351.45).unwrap();
        assert!((gamma[1] - 1.0).abs() < 1e-6, "γ1 = {}", gamma[1]);
    }

    #[test]
    fn ethanol_water_is_positive_deviation() {
        let nrtl = ethanol_water();
        let gamma = nrtl.gamma(&[0.5, 0.5], 351.45).unwrap();
        assert!(gamma[0] > 1.0, "γ_ethanol = {}", gamma[0]);
        assert!(gamma[1] > 0.8, "γ_water = {}", gamma[1]);
    }

    #[test]
    fn gibbs_duhem_residual() {
        // x1·∂lnγ1/∂x1 + x2·∂lnγ2/∂x1 ≈ 0 along a binary composition path
        // (Gibbs–Duhem at constant T, P — an internal consistency check of
        // the implementation).
        let nrtl = ethanol_water();
        let t = 351.45;
        let h = 1e-6;
        let x1 = 0.3;
        let plus = nrtl.ln_gamma(&[x1 + h, 1.0 - x1 - h], t).unwrap();
        let minus = nrtl.ln_gamma(&[x1 - h, 1.0 - x1 + h], t).unwrap();
        let dg1 = (plus[0] - minus[0]) / (2.0 * h);
        let dg2 = (plus[1] - minus[1]) / (2.0 * h);
        let residual = x1 * dg1 + (1.0 - x1) * dg2;
        assert!(residual.abs() < 1e-4, "Gibbs-Duhem residual = {residual}");
    }

    #[test]
    fn excess_gibbs_sign() {
        let nrtl = ethanol_water();
        let gamma = nrtl.gamma(&[0.5, 0.5], 351.45).unwrap();
        let ln_gamma: Vec<f64> = gamma.iter().map(|g| g.ln()).collect();
        assert!(excess_gibbs_over_rt(&ln_gamma, &[0.5, 0.5]) > 0.0);
    }

    #[test]
    fn rejects_wrong_length() {
        let nrtl = ethanol_water();
        assert!(nrtl.ln_gamma(&[0.5], 350.0).is_err());
    }
}
