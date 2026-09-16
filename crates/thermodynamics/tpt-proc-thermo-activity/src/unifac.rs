//! UNIFAC group-contribution activity coefficients.
//!
//! The computation implements the original UNIFAC model (Fredenslund,
//! Jones & Prausnitz, 1975): the UNIQUAC combinatorial term over molecular
//! r/q built from subgroup parameters, plus a main-group residual term.
//!
//! # Built-in table coverage
//!
//! [`UnifacTable::builtin_subset`] ships the classic groups needed for
//! hydrocarbon / aromatic / alcohol / water systems (main groups CH2,
//! ACH, OH, CH3OH, H2O with the standard 1975 interaction values). This is
//! a **demonstration subset**: the full UNIFAC matrix spans ~50 main
//! groups and is community-maintained data. Build a fuller table from
//! published sources and pass it to [`Unifac::with_table`] — the math is
//! table-agnostic.
//!
//! # Example
//!
//! ```
//! use tpt_proc_thermo_activity::{GroupCount, Unifac, UnifacTable};
//! use tpt_proc_thermo_core::ActivityCoefficientModel;
//!
//! // Ethanol: CH3 + CH2 + OH ; Water: H2O
//! let ethanol = GroupCount::from_pairs(&[(1, 1.0), (2, 1.0), (14, 1.0)]);
//! let water = GroupCount::from_pairs(&[(16, 1.0)]);
//! let unifac = Unifac::new(
//!     UnifacTable::builtin_subset(),
//!     vec![ethanol, water],
//! );
//!
//! let gamma = unifac.gamma(&[0.5, 0.5], 351.45).unwrap();
//! assert!(gamma[0].is_finite() && gamma[1].is_finite());
//! // Water in ethanol-rich mixtures shows positive deviation.
//! assert!(gamma[1] > 1.0);
//! ```

use std::collections::{BTreeMap, BTreeSet};

use std::result::Result;

use tpt_proc_thermo_core::{ActivityCoefficientModel, ThermoError};

use crate::validate_composition;

/// Subgroup decomposition of one molecule: subgroup id → count.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GroupCount(pub BTreeMap<u16, f64>);

impl GroupCount {
    /// Builds a group count from (subgroup, count) pairs.
    #[must_use]
    pub fn from_pairs(pairs: &[(u16, f64)]) -> Self {
        Self(pairs.iter().copied().collect())
    }
}

/// Main-group parameters (representative R/Q used by the residual term).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MainGroupDef {
    /// Main group name (documentation).
    pub name: &'static str,
    /// Volume parameter R (main-group value from the standard table).
    pub r: f64,
    /// Surface parameter Q (main-group value from the standard table).
    pub q: f64,
}

/// One subgroup definition: parent main group plus volume/surface params.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubgroupDef {
    /// Subgroup name (documentation).
    pub name: &'static str,
    /// Parent main group id.
    pub main_group: u16,
    /// Volume parameter Rₖ.
    pub r: f64,
    /// Surface parameter Qₖ.
    pub q: f64,
}

/// A UNIFAC parameter table: main- and subgroup definitions plus
/// main-group interaction energies a_mk in K.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnifacTable {
    /// Main group definitions keyed by main group id.
    pub main_groups: BTreeMap<u16, MainGroupDef>,
    /// Subgroup definitions keyed by subgroup id.
    pub subgroups: BTreeMap<u16, SubgroupDef>,
    /// Main-group interaction a_mk (K), keyed by ordered (m, k).
    pub interactions: BTreeMap<(u16, u16), f64>,
}

impl UnifacTable {
    /// The classic demonstration subset: CH2 (main 1), ACH (main 3), OH
    /// (main 5), CH3OH (main 6), H2O (main 7), with the standard 1975
    /// interaction values. Interaction pairs not present default to 0.
    #[must_use]
    pub fn builtin_subset() -> Self {
        let mut main_groups = BTreeMap::new();
        main_groups.insert(
            1,
            MainGroupDef {
                name: "CH2",
                r: 0.6744,
                q: 0.540,
            },
        );
        main_groups.insert(
            3,
            MainGroupDef {
                name: "ACH",
                r: 0.3652,
                q: 0.120,
            },
        );
        main_groups.insert(
            5,
            MainGroupDef {
                name: "OH",
                r: 1.0000,
                q: 1.200,
            },
        );
        main_groups.insert(
            6,
            MainGroupDef {
                name: "CH3OH",
                r: 1.4311,
                q: 1.432,
            },
        );
        main_groups.insert(
            7,
            MainGroupDef {
                name: "H2O",
                r: 0.9200,
                q: 1.400,
            },
        );

        let mut subgroups = BTreeMap::new();
        subgroups.insert(
            1,
            SubgroupDef {
                name: "CH3",
                main_group: 1,
                r: 0.9011,
                q: 0.848,
            },
        );
        subgroups.insert(
            2,
            SubgroupDef {
                name: "CH2",
                main_group: 1,
                r: 0.6744,
                q: 0.540,
            },
        );
        subgroups.insert(
            3,
            SubgroupDef {
                name: "CH",
                main_group: 1,
                r: 0.4469,
                q: 0.228,
            },
        );
        subgroups.insert(
            9,
            SubgroupDef {
                name: "ACH",
                main_group: 3,
                r: 0.3652,
                q: 0.120,
            },
        );
        subgroups.insert(
            14,
            SubgroupDef {
                name: "OH",
                main_group: 5,
                r: 1.0000,
                q: 1.200,
            },
        );
        subgroups.insert(
            15,
            SubgroupDef {
                name: "CH3OH",
                main_group: 6,
                r: 1.4311,
                q: 1.432,
            },
        );
        subgroups.insert(
            16,
            SubgroupDef {
                name: "H2O",
                main_group: 7,
                r: 0.9200,
                q: 1.400,
            },
        );

        // a_mk in K, standard UNIFAC main-group table (subset).
        let mut interactions = BTreeMap::new();
        interactions.insert((1, 3), -114.1);
        interactions.insert((3, 1), -114.1);
        interactions.insert((1, 5), 986.5);
        interactions.insert((5, 1), 156.4);
        interactions.insert((1, 6), 697.2);
        interactions.insert((6, 1), 16.51);
        interactions.insert((1, 7), 1318.0);
        interactions.insert((7, 1), 300.0);
        interactions.insert((3, 5), 636.1);
        interactions.insert((5, 3), -137.1);
        interactions.insert((3, 6), 637.35);
        interactions.insert((6, 3), -137.1);
        interactions.insert((3, 7), 903.8);
        interactions.insert((7, 3), 329.0);
        interactions.insert((5, 7), 353.5);
        interactions.insert((7, 5), -229.1);
        interactions.insert((6, 7), -181.0);
        interactions.insert((7, 6), 289.6);
        // (5, 6)/(6, 5) not tabulated in this subset; default 0.

        Self {
            main_groups,
            subgroups,
            interactions,
        }
    }
}

/// The UNIFAC model over a fixed component list.
#[derive(Clone, Debug, PartialEq)]
pub struct Unifac {
    table: UnifacTable,
    components: Vec<GroupCount>,
}

impl Unifac {
    /// Builds a UNIFAC model with the built-in subset table.
    #[must_use]
    pub fn new(table: UnifacTable, components: Vec<GroupCount>) -> Self {
        Self { table, components }
    }

    /// Replaces the parameter table.
    #[must_use]
    pub fn with_table(mut self, table: UnifacTable) -> Self {
        self.table = table;
        self
    }

    /// Activity coefficients γᵢ (not logs).
    ///
    /// # Errors
    /// [`ThermoError::InvalidComponent`] on malformed input or unknown
    /// subgroup ids.
    pub fn gamma(&self, x: &[f64], temperature: f64) -> Result<Vec<f64>, ThermoError> {
        self.ln_gamma_impl(x, temperature)
            .map(|ln| ln.iter().map(|g| g.exp()).collect())
    }

    /// Molecule-level r_i = Σ ν R and q_i = Σ ν Q (subgroup parameters).
    fn molecular_r_q(&self, i: usize) -> Result<(f64, f64), ThermoError> {
        let mut r = 0.0;
        let mut q = 0.0;
        for (subgroup, count) in &self.components[i].0 {
            let def = self.table.subgroups.get(subgroup).ok_or_else(|| {
                ThermoError::InvalidComponent(format!(
                    "subgroup {subgroup} not in the UNIFAC table"
                ))
            })?;
            r += count * def.r;
            q += count * def.q;
        }
        Ok((r, q))
    }

    /// Main-group counts of one molecule: main group id → total count.
    fn main_counts(&self, i: usize) -> Result<BTreeMap<u16, f64>, ThermoError> {
        let mut counts = BTreeMap::new();
        for (subgroup, count) in &self.components[i].0 {
            let def = self.table.subgroups.get(subgroup).ok_or_else(|| {
                ThermoError::InvalidComponent(format!(
                    "subgroup {subgroup} not in the UNIFAC table"
                ))
            })?;
            *counts.entry(def.main_group).or_insert(0.0) += count;
        }
        Ok(counts)
    }

    /// All main groups present anywhere in the component list.
    fn all_main_groups(&self) -> Result<Vec<u16>, ThermoError> {
        let mut set = BTreeSet::new();
        for i in 0..self.components.len() {
            for key in self.main_counts(i)?.keys() {
                set.insert(*key);
            }
        }
        Ok(set.into_iter().collect())
    }

    /// Residual term ln Γ_m for one phase composition, over main groups.
    fn residual_ln_gamma(
        &self,
        main_counts_per_component: &[BTreeMap<u16, f64>],
        x: &[f64],
        temperature: f64,
    ) -> Result<BTreeMap<u16, f64>, ThermoError> {
        let groups = self.all_main_groups()?;

        // Group mole fractions X_m and area fractions θ_m.
        let mut total_count = 0.0;
        let mut x_count: BTreeMap<u16, f64> = BTreeMap::new();
        for (i, counts) in main_counts_per_component.iter().enumerate() {
            for m in &groups {
                let nu = counts.get(m).copied().unwrap_or(0.0);
                *x_count.entry(*m).or_insert(0.0) += x[i] * nu;
                total_count += x[i] * nu;
            }
        }
        let mut theta = BTreeMap::new();
        for m in &groups {
            let q_m = self.table.main_groups.get(m).map(|d| d.q).unwrap_or(1.0);
            theta.insert(*m, q_m * x_count[m] / total_count.max(1e-300));
        }

        // ln Γ_m = Q_m [1 − ln(Σ_n θ_n Ψ_nm) − Σ_n θ_n Ψ_mn/Σ_p θ_p Ψ_np]
        let psi = |m: u16, k: u16| -> f64 {
            (-self.table.interactions.get(&(m, k)).copied().unwrap_or(0.0) / temperature).exp()
        };
        let mut ln_gamma = BTreeMap::new();
        for k in &groups {
            let q_k = self.table.main_groups.get(k).map(|d| d.q).unwrap_or(1.0);
            let s1 = groups
                .iter()
                .map(|n| theta[n] * psi(*n, *k))
                .sum::<f64>()
                .max(1e-300);
            let mut s2 = 0.0;
            for n in &groups {
                let s_n = groups
                    .iter()
                    .map(|p| theta[p] * psi(*p, *n))
                    .sum::<f64>()
                    .max(1e-300);
                s2 += theta[n] * psi(*k, *n) / s_n;
            }
            ln_gamma.insert(*k, q_k * (1.0 - s1.ln() - s2));
        }
        Ok(ln_gamma)
    }

    fn ln_gamma_impl(&self, x: &[f64], temperature: f64) -> Result<Vec<f64>, ThermoError> {
        validate_composition(x)?;
        if x.len() != self.components.len() {
            return Err(ThermoError::InvalidComponent(format!(
                "composition has {} components; model built for {}",
                x.len(),
                self.components.len()
            )));
        }
        let n = x.len();
        let z = 10.0;

        // Molecular parameters.
        let mut r_vec = Vec::with_capacity(n);
        let mut q_vec = Vec::with_capacity(n);
        for i in 0..n {
            let (r, q) = self.molecular_r_q(i)?;
            r_vec.push(r);
            q_vec.push(q);
        }
        let sum_rx: f64 = x.iter().zip(&r_vec).map(|(xi, ri)| xi * ri).sum();
        let sum_qx: f64 = x.iter().zip(&q_vec).map(|(xi, qi)| xi * qi).sum();

        // Residual terms: mixture and per pure component.
        let main_counts: Vec<BTreeMap<u16, f64>> = (0..n)
            .map(|i| self.main_counts(i))
            .collect::<Result<Vec<_>, ThermoError>>()?;
        let mix_residual = self.residual_ln_gamma(&main_counts, x, temperature)?;
        let mut pure_residual = Vec::with_capacity(n);
        for i in 0..n {
            let mut pure_x = vec![0.0; n];
            pure_x[i] = 1.0;
            pure_residual.push(self.residual_ln_gamma(&main_counts, &pure_x, temperature)?);
        }

        let mut ln_gamma = vec![0.0; n];
        for (i, ln_gi) in ln_gamma.iter_mut().enumerate().take(n) {
            let x_i = x[i].max(1e-12);
            let phi = r_vec[i] * x_i / sum_rx.max(1e-300);
            let theta = q_vec[i] * x_i / sum_qx.max(1e-300);
            let l_i = (z / 2.0) * (r_vec[i] - q_vec[i]) - (r_vec[i] - 1.0);
            let sum_xl: f64 = x
                .iter()
                .enumerate()
                .map(|(j, xj)| {
                    let l_j = (z / 2.0) * (r_vec[j] - q_vec[j]) - (r_vec[j] - 1.0);
                    xj * l_j
                })
                .sum();
            let combinatorial = (phi / x_i).ln()
                + (z / 2.0) * q_vec[i] * (theta.max(1e-300) / phi.max(1e-300)).ln()
                + l_i
                - (phi / x_i) * sum_xl;

            // Residual: Σ_m ν_mi·(ln Γ_m^mix − ln Γ_m^pure).
            let mut residual = 0.0;
            for (m, count) in &main_counts[i] {
                let mix_val = mix_residual.get(m).copied().unwrap_or(0.0);
                let pure_val = pure_residual[i].get(m).copied().unwrap_or(0.0);
                residual += count * (mix_val - pure_val);
            }
            *ln_gi = combinatorial + residual;
        }
        Ok(ln_gamma)
    }
}

impl ActivityCoefficientModel for Unifac {
    fn name(&self) -> &str {
        "UNIFAC"
    }

    fn num_components(&self) -> usize {
        self.components.len()
    }

    fn ln_gamma(&self, x: &[f64], temperature: f64) -> Result<Vec<f64>, ThermoError> {
        self.ln_gamma_impl(x, temperature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn water() -> GroupCount {
        GroupCount::from_pairs(&[(16, 1.0)])
    }

    fn ethanol() -> GroupCount {
        GroupCount::from_pairs(&[(1, 1.0), (2, 1.0), (14, 1.0)])
    }

    #[test]
    fn pure_component_gamma_is_one() {
        let unifac = Unifac::new(UnifacTable::builtin_subset(), vec![water()]);
        let gamma = unifac.gamma(&[1.0], 373.15).unwrap();
        assert!((gamma[0] - 1.0).abs() < 1e-6, "γ = {}", gamma[0]);
    }

    #[test]
    fn water_in_ethanol_shows_positive_deviation() {
        let unifac = Unifac::new(UnifacTable::builtin_subset(), vec![ethanol(), water()]);
        let gamma = unifac.gamma(&[0.9, 0.1], 351.45).unwrap();
        assert!(gamma[1] > 1.0, "γ_water = {}", gamma[1]);
        assert!(gamma[0].is_finite());
    }

    #[test]
    fn alkane_mixture_near_ideal() {
        // Pentane-like (2 CH3 + 3 CH2) vs hexane-like (2 CH3 + 4 CH2):
        // same main family → residual cancels; combinatorial is mild.
        let pentane = GroupCount::from_pairs(&[(1, 2.0), (2, 3.0)]);
        let hexane = GroupCount::from_pairs(&[(1, 2.0), (2, 4.0)]);
        let unifac = Unifac::new(UnifacTable::builtin_subset(), vec![pentane, hexane]);
        let gamma = unifac.gamma(&[0.5, 0.5], 340.0).unwrap();
        assert!(gamma[0] > 0.8 && gamma[0] < 1.3, "γ = {}", gamma[0]);
        assert!(gamma[1] > 0.8 && gamma[1] < 1.3, "γ = {}", gamma[1]);
    }

    #[test]
    fn unknown_subgroup_rejected() {
        let bogus = GroupCount::from_pairs(&[(999, 1.0)]);
        let unifac = Unifac::new(UnifacTable::builtin_subset(), vec![bogus]);
        assert!(unifac.ln_gamma(&[1.0], 300.0).is_err());
    }

    #[test]
    fn deviations_shrink_with_temperature() {
        let unifac = Unifac::new(UnifacTable::builtin_subset(), vec![ethanol(), water()]);
        let g_low = unifac.gamma(&[0.5, 0.5], 300.0).unwrap();
        let g_high = unifac.gamma(&[0.5, 0.5], 500.0).unwrap();
        let dev_low = (g_low[0] - 1.0).abs() + (g_low[1] - 1.0).abs();
        let dev_high = (g_high[0] - 1.0).abs() + (g_high[1] - 1.0).abs();
        assert!(dev_high < dev_low, "low={dev_low} high={dev_high}");
    }
}
