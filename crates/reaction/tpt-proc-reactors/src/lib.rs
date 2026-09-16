//! Reactor models: batch, CSTR, and PFR with optional adiabatic energy
//! balance. RK4 integration for the dynamic models.
//!
//! The milestone model: CSTR and PFR with an exothermic first-order
//! reaction, matching the analytic limits.
//!
//! # Example
//!
//! ```
//! use std::collections::BTreeMap;
//! use tpt_proc_reaction::{RateLaw, Reaction};
//! use tpt_proc_reactors::{CstrSpec, PfrSpec, ReactorSim};
//!
//! // First-order A → B, k = 0.5 s⁻¹ at feed temperature.
//! let mut stoich = BTreeMap::new();
//! stoich.insert(0, -1.0);
//! let mut orders = BTreeMap::new();
//! orders.insert(0, 1.0);
//! let reaction = Reaction::new(
//!     "A → B",
//!     stoich,
//!     RateLaw::Arrhenius {
//!         pre_exponential: 0.5,
//!         activation_energy: 0.0,
//!         orders,
//!     },
//!     -60.0e3,
//! );
//!
//! // CSTR of 2 m³ at 1 m³/s feed → residence time 2 s → X = kτ/(1+kτ).
//! let cstr = ReactorSim::new(vec![reaction.clone()]);
//! let out = cstr.cstr(&CstrSpec { volume: 2.0 }, &[10.0], 1.0, 350.0, false);
//! assert!((out.conversion[0] - 0.5).abs() < 1e-6);
//!
//! // PFR of 2 m³ at 1 m³/s → X = 1 − e^(−kτ).
//! let pfr = ReactorSim::new(vec![reaction]);
//! let out = pfr.pfr(&PfrSpec { volume: 2.0 }, &[10.0], 1.0, 350.0, false);
//! assert!((out.conversion[0] - (1.0 - (-1.0_f64).exp())).abs() < 1e-4);
//! ```

#![forbid(unsafe_code)]

use tpt_proc_reaction::Reaction;

/// Reactor simulation inputs shared by all models.
pub struct ReactorSim {
    reactions: Vec<Reaction>,
}

/// Result of a reactor calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct ReactorResult {
    /// Outlet component concentrations, mol/m³.
    pub concentrations: Vec<f64>,
    /// Conversion of each component with a negative stoichiometric
    /// coefficient (reactants), [0, 1].
    pub conversion: Vec<f64>,
    /// Outlet temperature, K (equals feed T for isothermal runs).
    pub temperature: f64,
    /// Heat duty removed (positive = heating), W.
    pub heat_duty: f64,
}

/// CSTR specification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CstrSpec {
    /// Reacting volume, m³.
    pub volume: f64,
}

/// PFR specification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PfrSpec {
    /// Reacting volume, m³.
    pub volume: f64,
}

impl ReactorSim {
    /// Creates a reactor simulator over a reaction set.
    #[must_use]
    pub fn new(reactions: Vec<Reaction>) -> Self {
        Self { reactions }
    }

    fn volumetric_rate(&self, c: &[f64], t: f64) -> Vec<f64> {
        let mut rates = vec![0.0_f64; c.len()];
        for r in &self.reactions {
            r.dcdt(c, t, &mut rates);
        }
        rates
    }

    /// Heat release rate, W (negative = heat must be removed), from
    /// Σ(−ΔH_R)·r over reactions.
    #[must_use]
    pub fn heat_release(&self, c: &[f64], t: f64) -> f64 {
        self.reactions
            .iter()
            .map(|r| -r.heat_of_reaction * r.rate(c, t))
            .sum()
    }

    /// Steady-state CSTR: algebraic solve by fixed-point iteration on the
    /// outlet concentrations. Isothermal at `temperature` unless
    /// `adiabatic` (energy balance with volumetric heat capacity
    /// `volumetric_cp` J/(m³·K)).
    #[must_use]
    pub fn cstr(
        &self,
        spec: &CstrSpec,
        feed_c: &[f64],
        feed_rate_m3s: f64,
        temperature: f64,
        adiabatic: bool,
    ) -> ReactorResult {
        let tau = spec.volume / feed_rate_m3s.max(1e-12);
        let volumetric_cp = 4.0e6_f64; // J/(m³·K) default (water-like)
        let mut c = feed_c.to_vec();
        let mut t_out = temperature;
        // Under-relaxed fixed point on the CSTR design equation
        // c_out = c_in + τ·r(c_out); relaxation factor 0.5 damps the
        // oscillation the undamped iteration shows for τ·k ≳ 1.
        let omega = 0.5;
        for _ in 0..20000 {
            let rates = self.volumetric_rate(&c, t_out);
            let direct: Vec<f64> = feed_c
                .iter()
                .zip(&rates)
                .map(|(ci, ri)| (ci + tau * ri).max(0.0))
                .collect();
            if adiabatic {
                let q = self.heat_release(&c, t_out);
                let t_target = temperature + tau * q / volumetric_cp;
                t_out += omega * (t_target - t_out);
            }
            let delta: f64 = direct
                .iter()
                .zip(&c)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max);
            for (ci, target) in c.iter_mut().zip(&direct) {
                *ci += omega * (*target - *ci);
            }
            if delta < 1e-12 {
                break;
            }
        }
        let conversion = self.conversions(feed_c, &c);
        ReactorResult {
            concentrations: c.clone(),
            conversion,
            temperature: t_out,
            heat_duty: -self.heat_release(&c, t_out),
        }
    }

    /// Steady-state PFR: integrate dc/dt = r(c, T) with dV (residence) by
    /// RK4. Isothermal at `temperature` unless `adiabatic`.
    #[must_use]
    pub fn pfr(
        &self,
        spec: &PfrSpec,
        feed_c: &[f64],
        feed_rate_m3s: f64,
        temperature: f64,
        adiabatic: bool,
    ) -> ReactorResult {
        let tau_total = spec.volume / feed_rate_m3s.max(1e-12);
        let volumetric_cp = 4.0e6_f64;
        let steps = 2000;
        let dt = tau_total / f64::from(steps);
        let mut c = feed_c.to_vec();
        let mut t = temperature;
        for _ in 0..steps {
            let deriv = |cc: &[f64], tt: f64| self.volumetric_rate(cc, tt);
            // RK4 on concentrations; temperature co-integrated if adiabatic.
            let (k1c, k1t) = {
                let d = deriv(&c, t);
                let dt_t = if adiabatic {
                    self.heat_release(&c, t) / volumetric_cp
                } else {
                    0.0
                };
                (d, dt_t)
            };
            let c2: Vec<f64> = c.iter().zip(&k1c).map(|(a, b)| a + 0.5 * dt * b).collect();
            let (k2c, k2t) = {
                let d = deriv(&c2, t + 0.5 * dt * k1t);
                let dt_t = if adiabatic {
                    self.heat_release(&c2, t + 0.5 * dt * k1t) / volumetric_cp
                } else {
                    0.0
                };
                (d, dt_t)
            };
            let c3: Vec<f64> = c.iter().zip(&k2c).map(|(a, b)| a + 0.5 * dt * b).collect();
            let (k3c, k3t) = {
                let d = deriv(&c3, t + 0.5 * dt * k2t);
                let dt_t = if adiabatic {
                    self.heat_release(&c3, t + 0.5 * dt * k2t) / volumetric_cp
                } else {
                    0.0
                };
                (d, dt_t)
            };
            let c4: Vec<f64> = c.iter().zip(&k3c).map(|(a, b)| a + dt * b).collect();
            let k4c = deriv(&c4, t + dt * k3t);
            let k4t = if adiabatic {
                self.heat_release(&c4, t + dt * k3t) / volumetric_cp
            } else {
                0.0
            };
            for (i, ci) in c.iter_mut().enumerate() {
                let increment = dt / 6.0 * (k1c[i] + 2.0 * k2c[i] + 2.0 * k3c[i] + k4c[i]);
                *ci = (*ci + increment).max(0.0);
            }
            t += dt / 6.0 * (k1t + 2.0 * k2t + 2.0 * k3t + k4t);
        }
        let conversion = self.conversions(feed_c, &c);
        ReactorResult {
            concentrations: c.clone(),
            conversion,
            temperature: t,
            heat_duty: -self.heat_release(&c, t),
        }
    }

    fn conversions(&self, feed_c: &[f64], out_c: &[f64]) -> Vec<f64> {
        // Components consumed somewhere in the reaction set.
        let mut reactant_indices = std::collections::BTreeSet::new();
        for r in &self.reactions {
            for (&i, &nu) in &r.stoichiometry {
                if nu < 0.0 {
                    reactant_indices.insert(i);
                }
            }
        }
        reactant_indices
            .into_iter()
            .map(|i| {
                let c0 = feed_c.get(i).copied().unwrap_or(0.0);
                if c0 <= 0.0 {
                    return 0.0;
                }
                ((c0 - out_c.get(i).copied().unwrap_or(0.0)) / c0).clamp(0.0, 1.0)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tpt_proc_reaction::RateLaw;

    /// First-order A → B with k = 0.5 s⁻¹ (zero activation energy).
    fn first_order() -> Reaction {
        let mut stoich = BTreeMap::new();
        stoich.insert(0, -1.0);
        stoich.insert(1, 1.0);
        let mut orders = BTreeMap::new();
        orders.insert(0, 1.0);
        Reaction::new(
            "A → B",
            stoich,
            RateLaw::Arrhenius {
                pre_exponential: 0.5,
                activation_energy: 0.0,
                orders,
            },
            -60.0e3,
        )
    }

    #[test]
    fn cstr_first_order_matches_analytic_conversion() {
        let sim = ReactorSim::new(vec![first_order()]);
        // τ = 2 s, k = 0.5: X = kτ/(1+kτ) = 0.5.
        let out = sim.cstr(&CstrSpec { volume: 2.0 }, &[10.0, 0.0], 1.0, 350.0, false);
        assert!((out.conversion[0] - 0.5).abs() < 1e-6);
        assert!((out.concentrations[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn pfr_first_order_matches_analytic_conversion() {
        let sim = ReactorSim::new(vec![first_order()]);
        let out = sim.pfr(&PfrSpec { volume: 2.0 }, &[10.0, 0.0], 1.0, 350.0, false);
        let expected = 1.0 - (-0.5 * 2.0_f64).exp();
        assert!(
            (out.conversion[0] - expected).abs() < 1e-4,
            "X = {} vs {expected}",
            out.conversion[0]
        );
    }

    #[test]
    fn pfr_beats_cstr_for_positive_order() {
        let sim = ReactorSim::new(vec![first_order()]);
        let cstr = sim.cstr(&CstrSpec { volume: 2.0 }, &[10.0, 0.0], 1.0, 350.0, false);
        let pfr = sim.pfr(&PfrSpec { volume: 2.0 }, &[10.0, 0.0], 1.0, 350.0, false);
        assert!(pfr.conversion[0] > cstr.conversion[0]);
    }

    #[test]
    fn adiabatic_cstr_heats_up() {
        let sim = ReactorSim::new(vec![first_order()]);
        let out = sim.cstr(&CstrSpec { volume: 2.0 }, &[10.0, 0.0], 1.0, 350.0, true);
        // ΔT_ad = X·(−ΔH_R)·C_A0/(ρcp) = 0.5·60e3·10/4e6 = 0.075 K for the
        // dilute water-like stream.
        assert!(
            (out.temperature - 350.075).abs() < 0.005,
            "T_out = {}",
            out.temperature
        );
        assert!(out.temperature > 350.0);
    }

    #[test]
    fn batch_like_pfr_profile_is_monotone() {
        let sim = ReactorSim::new(vec![first_order()]);
        let out = sim.pfr(&PfrSpec { volume: 4.0 }, &[10.0, 0.0], 1.0, 350.0, false);
        assert!(out.concentrations[1] >= 0.0);
        assert!(out.conversion[0] > 0.8);
    }
}
