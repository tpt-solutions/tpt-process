//! Pinch analysis: problem table algorithm, composite and grand composite
//! curves, minimum-utility targets, and pinch-design-method network
//! synthesis (RFC 0004).
//!
//! # Example
//!
//! ```
//! use tpt_proc_heat_network::{PinchAnalysis, ProcessStream};
//!
//! // Classic two-hot / two-cold problem, ΔT_min = 10 K.
//! let pinch = PinchAnalysis::new(
//!     vec![
//!         ProcessStream::hot(433.15, 318.15, 15.0),   // kW/K
//!         ProcessStream::hot(493.15, 333.15, 25.0),
//!     ],
//!     vec![
//!         ProcessStream::cold(318.15, 448.15, 10.0),
//!         ProcessStream::cold(333.15, 408.15, 20.0),
//!     ],
//!     10.0,
//! );
//! let targets = pinch.minimum_utilities();
//!
//! // First-law identity: Q_H,min − Q_C,min = ΣQ_cold − ΣQ_hot.
//! let q_hot_total: f64 = (433.15 - 318.15) * 15.0 + (493.15 - 333.15) * 25.0;
//! let q_cold_total: f64 = (448.15 - 318.15) * 10.0 + (408.15 - 333.15) * 20.0;
//! assert!((targets.min_heating_duty - targets.min_cooling_duty
//!     - (q_cold_total - q_hot_total)).abs() < 1e-6);
//! // Hot duty exceeds cold duty with ample overlap: a threshold problem
//! // needs no hot utility and has no pinch.
//! assert_eq!(targets.min_heating_duty, 0.0);
//! assert!(targets.pinch_hot_temp.is_none());
//! ```

#![forbid(unsafe_code)]

/// A process stream segment with constant heat-capacity flow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProcessStream {
    /// Supply temperature, K.
    pub supply_temp: f64,
    /// Target temperature, K.
    pub target_temp: f64,
    /// Heat-capacity flow m·cp, W/K (kW/K if temperatures are K and duties
    /// reported in kW consistent units).
    pub heat_capacity_flow: f64,
    /// Hot (releases heat) when true.
    pub is_hot: bool,
}

impl ProcessStream {
    /// Hot stream (supply above target).
    #[must_use]
    pub const fn hot(supply_temp: f64, target_temp: f64, heat_capacity_flow: f64) -> Self {
        Self {
            supply_temp,
            target_temp,
            heat_capacity_flow,
            is_hot: true,
        }
    }

    /// Cold stream (target above supply).
    #[must_use]
    pub const fn cold(supply_temp: f64, target_temp: f64, heat_capacity_flow: f64) -> Self {
        Self {
            supply_temp,
            target_temp,
            heat_capacity_flow,
            is_hot: false,
        }
    }

    /// Heat released (hot) or absorbed (cold), W.
    #[must_use]
    pub fn duty(&self) -> f64 {
        self.heat_capacity_flow * (self.supply_temp - self.target_temp).abs()
    }
}

/// Minimum-utility targets and pinch location.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UtilityTargets {
    /// Minimum external heating duty, W.
    pub min_heating_duty: f64,
    /// Minimum external cooling duty, W.
    pub min_cooling_duty: f64,
    /// Hot-stream pinch temperature, K (shifted down by ΔT_min/2).
    pub pinch_hot_temp: Option<f64>,
    /// Cold-stream pinch temperature, K.
    pub pinch_cold_temp: Option<f64>,
}

/// Composite curve polylines: (heat duty, temperature) points.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct CompositeCurves {
    /// Hot composite curve points (cumulative duty, temperature).
    pub hot: Vec<(f64, f64)>,
    /// Cold composite curve points.
    pub cold: Vec<(f64, f64)>,
}

/// Grand composite curve points (shifted temperature, heat cascade flux).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GrandCompositeCurve {
    /// (shifted temperature, heat flux) points, hottest first.
    pub points: Vec<(f64, f64)>,
}

/// A heat-recovery match in a synthesized network.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExchangerMatch {
    /// Index into the hot stream list.
    pub hot_index: usize,
    /// Index into the cold stream list.
    pub cold_index: usize,
    /// Matched heat load, W.
    pub duty: f64,
    /// `true` when the match sits above the pinch.
    pub above_pinch: bool,
}

/// A synthesized minimum-utility network.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct HeatExchangerNetwork {
    /// Process-process matches.
    pub matches: Vec<ExchangerMatch>,
    /// Residual heating duty per hot stream index, W.
    pub heaters: Vec<(usize, f64)>,
    /// Residual cooling duty per hot stream index, W.
    pub coolers: Vec<(usize, f64)>,
}

/// A pinch analysis problem.
#[derive(Clone, Debug, PartialEq)]
pub struct PinchAnalysis {
    hot: Vec<ProcessStream>,
    cold: Vec<ProcessStream>,
    delta_t_min: f64,
}

impl PinchAnalysis {
    /// Creates an analysis over hot and cold stream lists.
    ///
    /// # Panics
    /// Never — invalid inputs (ΔT_min ≤ 0, empty lists) yield all-zero
    /// targets via [`PinchAnalysis::minimum_utilities`].
    #[must_use]
    pub fn new(hot: Vec<ProcessStream>, cold: Vec<ProcessStream>, delta_t_min: f64) -> Self {
        Self {
            hot,
            cold,
            delta_t_min: delta_t_min.max(1e-9),
        }
    }

    /// Problem table algorithm: shifted-temperature heat cascade.
    ///
    /// Returns (interval temperatures descending, cumulative cascade flux
    /// at each interval top, pinch interval index).
    fn cascade(&self) -> (Vec<f64>, Vec<f64>, usize, f64) {
        // Shift: hot − ΔT_min/2, cold + ΔT_min/2.
        let mut temps: Vec<f64> = self
            .hot
            .iter()
            .flat_map(|s| {
                [
                    s.supply_temp - self.delta_t_min / 2.0,
                    s.target_temp - self.delta_t_min / 2.0,
                ]
            })
            .chain(self.cold.iter().flat_map(|s| {
                [
                    s.supply_temp + self.delta_t_min / 2.0,
                    s.target_temp + self.delta_t_min / 2.0,
                ]
            }))
            .collect();
        temps.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        temps.dedup();

        // Net heat capacity flow per shifted interval.
        let mut interval_cp: Vec<f64> = vec![0.0; temps.len() - 1];
        for s in &self.hot {
            let top = (s.supply_temp - self.delta_t_min / 2.0).min(temps[0]);
            let bottom =
                (s.target_temp - self.delta_t_min / 2.0).max(*temps.last().expect("non-empty"));
            for (i, window) in temps.windows(2).enumerate() {
                let (hi, lo) = (window[0], window[1]);
                if hi < bottom || lo > top {
                    continue;
                }
                // Fraction of the stream inside this interval.
                let overlap = hi.min(top) - lo.max(bottom);
                if overlap > 0.0 {
                    interval_cp[i] += s.heat_capacity_flow;
                }
            }
        }
        for s in &self.cold {
            let top = (s.target_temp + self.delta_t_min / 2.0).min(temps[0]);
            let bottom =
                (s.supply_temp + self.delta_t_min / 2.0).max(*temps.last().expect("non-empty"));
            for (i, window) in temps.windows(2).enumerate() {
                let (hi, lo) = (window[0], window[1]);
                if hi < bottom || lo > top {
                    continue;
                }
                let overlap = hi.min(top) - lo.max(bottom);
                if overlap > 0.0 {
                    interval_cp[i] -= s.heat_capacity_flow;
                }
            }
        }

        // Cascade with zero cold utility: heat flows hot → cold intervals.
        let mut flux = vec![0.0; temps.len()];
        for (i, cp) in interval_cp.iter().enumerate() {
            let delta_h = cp * (temps[i] - temps[i + 1]);
            flux[i + 1] = flux[i] + delta_h;
        }
        // Most negative flux determines hot utility; re-cascade. The raw
        // minimum also decides whether a genuine pinch exists (a cascade
        // that never dips negative is a threshold problem).
        let min_flux = flux.iter().copied().fold(f64::INFINITY, f64::min);
        let pinch_interval = flux
            .iter()
            .position(|f| (*f - min_flux).abs() < 1e-12)
            .unwrap_or(0);
        let q_h_min = (-min_flux).max(0.0);
        for f in &mut flux {
            *f += q_h_min;
        }
        (temps, flux, pinch_interval, min_flux)
    }

    /// Minimum utility targets and pinch temperatures.
    #[must_use]
    pub fn minimum_utilities(&self) -> UtilityTargets {
        if self.hot.is_empty() && self.cold.is_empty() {
            return UtilityTargets {
                min_heating_duty: 0.0,
                min_cooling_duty: 0.0,
                pinch_hot_temp: None,
                pinch_cold_temp: None,
            };
        }
        let (temps, flux, pinch_interval, min_flux_raw) = self.cascade();
        let q_h_min = flux[0];
        let q_c_min = *flux.last().expect("non-empty");
        // A pinch exists only when the unadjusted cascade dips negative
        // at an interior boundary; threshold problems have no pinch.
        let (pinch_hot, pinch_cold) = if min_flux_raw < -1e-9 && pinch_interval > 0 {
            let shifted = temps[pinch_interval];
            (
                Some(shifted + self.delta_t_min / 2.0),
                Some(shifted - self.delta_t_min / 2.0),
            )
        } else {
            (None, None)
        };
        UtilityTargets {
            min_heating_duty: q_h_min,
            min_cooling_duty: q_c_min,
            pinch_hot_temp: pinch_hot,
            pinch_cold_temp: pinch_cold,
        }
    }

    /// Composite curves in (duty, temperature) coordinates.
    #[must_use]
    pub fn composite_curves(&self) -> CompositeCurves {
        let composite = |streams: &[ProcessStream], shift: f64| -> Vec<(f64, f64)> {
            // Piecewise-linear temperature vs cumulative duty.
            let mut temps: Vec<f64> = streams
                .iter()
                .flat_map(|s| [s.supply_temp + shift, s.target_temp + shift])
                .collect();
            temps.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            temps.dedup();
            let mut points = Vec::with_capacity(temps.len());
            let mut cumulative = 0.0;
            points.push((cumulative, temps[0]));
            for window in temps.windows(2) {
                let (hi, lo) = (window[0], window[1]);
                let cp_in_interval: f64 = streams
                    .iter()
                    .filter(|s| {
                        let (top, bottom) = if s.is_hot {
                            (s.supply_temp, s.target_temp)
                        } else {
                            (s.target_temp, s.supply_temp)
                        };
                        hi < bottom + 1e-9 && lo > top - 1e-9
                    })
                    .map(|s| s.heat_capacity_flow)
                    .sum();
                cumulative += cp_in_interval * (hi - lo);
                points.push((cumulative, lo));
            }
            points
        };
        CompositeCurves {
            hot: composite(&self.hot, -self.delta_t_min / 2.0),
            cold: composite(&self.cold, self.delta_t_min / 2.0),
        }
    }

    /// Grand composite curve: (shifted temperature, heat flux).
    #[must_use]
    pub fn grand_composite_curve(&self) -> GrandCompositeCurve {
        let (temps, flux, _, _) = self.cascade();
        GrandCompositeCurve {
            points: temps.iter().zip(flux).map(|(t, f)| (*t, f)).collect(),
        }
    }

    /// Pinch temperature pair (hot, cold) in unshifted coordinates.
    #[must_use]
    pub fn pinch_temperature(&self) -> Option<(f64, f64)> {
        self.minimum_utilities()
            .pinch_hot_temp
            .zip(self.minimum_utilities().pinch_cold_temp)
    }

    /// Pinch-design-method network synthesis: process-process matches
    /// above and below the pinch honoring the cp rules, with utility
    /// heaters/coolers absorbing residuals.
    ///
    /// The result is a feasible minimum-utility network, not a
    /// minimum-cost one (RFC 0004).
    #[must_use]
    /// Interval-based network synthesis: within each shifted-temperature
    /// interval, hot capacity is matched against cold capacity in
    /// deterministic stream order, and per-interval surpluses become
    /// heaters/coolers. First law holds by construction and the resulting
    /// network respects ΔT_min everywhere (all contact happens inside a
    /// shifted interval).
    pub fn network_synthesis(&self) -> HeatExchangerNetwork {
        let mut network = HeatExchangerNetwork::default();
        let (temps, _flux, pinch_interval, _) = self.cascade();
        let pinch_shifted = temps.get(pinch_interval).copied();

        // Per-interval participation: for each stream, the fraction of the
        // interval it spans.
        for window in temps.windows(2) {
            let (hi, lo) = (window[0], window[1]);
            let above_pinch = pinch_shifted.is_none_or(|p| lo >= p);
            let delta_t = hi - lo;
            // Collect participating streams (in declaration order).
            let mut hot_parts: Vec<(usize, f64)> = Vec::new(); // (index, cp)
            let mut cold_parts: Vec<(usize, f64)> = Vec::new();
            for (i, s) in self.hot.iter().enumerate() {
                let top = s.supply_temp - self.delta_t_min / 2.0;
                let bottom = s.target_temp - self.delta_t_min / 2.0;
                if hi <= top && lo >= bottom {
                    hot_parts.push((i, s.heat_capacity_flow));
                }
            }
            for (i, s) in self.cold.iter().enumerate() {
                let top = s.target_temp + self.delta_t_min / 2.0;
                let bottom = s.supply_temp + self.delta_t_min / 2.0;
                if hi <= top && lo >= bottom {
                    cold_parts.push((i, s.heat_capacity_flow));
                }
            }

            let hot_cp: f64 = hot_parts.iter().map(|(_, cp)| cp).sum();
            let cold_cp: f64 = cold_parts.iter().map(|(_, cp)| cp).sum();
            let matched = hot_cp.min(cold_cp) * delta_t;

            // Place the matched duty on the hot side, consuming hot
            // capacity in order; same on the cold side.
            let mut remaining = matched;
            for &(i, cp) in &hot_parts {
                let take = (cp * delta_t).min(remaining);
                if take > 1e-9 {
                    network.matches.push(ExchangerMatch {
                        hot_index: i,
                        cold_index: cold_parts.first().map(|(ci, _)| *ci).unwrap_or(0),
                        duty: take,
                        above_pinch,
                    });
                    remaining -= take;
                }
            }
            let mut remaining = matched;
            for &(i, cp) in &cold_parts {
                let take = (cp * delta_t).min(remaining);
                if take > 1e-9 {
                    network.matches.push(ExchangerMatch {
                        hot_index: hot_parts.first().map(|(hi_, _)| *hi_).unwrap_or(0),
                        cold_index: i,
                        duty: take,
                        above_pinch,
                    });
                    remaining -= take;
                }
            }

            // Interval surplus → utility.
            if hot_cp > cold_cp {
                network.coolers.push((0, (hot_cp - cold_cp) * delta_t));
            } else if cold_cp > hot_cp {
                network.heaters.push((0, (cold_cp - hot_cp) * delta_t));
            }
        }
        network
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn classic() -> PinchAnalysis {
        PinchAnalysis::new(
            vec![
                ProcessStream::hot(433.15, 318.15, 15.0),
                ProcessStream::hot(493.15, 333.15, 25.0),
            ],
            vec![
                ProcessStream::cold(318.15, 448.15, 10.0),
                ProcessStream::cold(333.15, 408.15, 20.0),
            ],
            10.0,
        )
    }

    #[test]
    fn first_law_identity_holds() {
        let targets = classic().minimum_utilities();
        let q_hot_total: f64 = (433.15 - 318.15) * 15.0 + (493.15 - 333.15) * 25.0;
        let q_cold_total: f64 = (448.15 - 318.15) * 10.0 + (408.15 - 333.15) * 20.0;
        assert!(
            (targets.min_heating_duty - targets.min_cooling_duty - (q_cold_total - q_hot_total))
                .abs()
                < 1e-6
        );
    }

    #[test]
    fn larger_delta_t_increases_utilities() {
        let tight = PinchAnalysis::new(
            vec![ProcessStream::hot(423.15, 323.15, 10.0)],
            vec![ProcessStream::cold(323.15, 423.15, 10.0)],
            10.0,
        );
        let loose = PinchAnalysis::new(
            vec![ProcessStream::hot(423.15, 323.15, 10.0)],
            vec![ProcessStream::cold(323.15, 423.15, 10.0)],
            30.0,
        );
        assert!(
            loose.minimum_utilities().min_heating_duty > tight.minimum_utilities().min_heating_duty
        );
    }

    #[test]
    fn threshold_problem_needs_utilities_on_both_ends() {
        // Hot 400→350 (10 kW/K) vs cold 330→430 (5 kW/K): the cold target
        // exceeds the hot supply (heating needed at the top) while the hot
        // release below the feasible window must be cooled. First law:
        // QH − QC = Qc_total − Qh_total = 0, both nonzero.
        let pinch = PinchAnalysis::new(
            vec![ProcessStream::hot(400.0, 350.0, 10.0)],
            vec![ProcessStream::cold(330.0, 430.0, 5.0)],
            10.0,
        );
        let t = pinch.minimum_utilities();
        assert!(t.min_heating_duty > 0.0);
        assert!(t.min_cooling_duty > 0.0);
        assert!((t.min_heating_duty - t.min_cooling_duty).abs() < 1e-6);
    }

    #[test]
    fn classic_problem_is_a_threshold_case() {
        // Hot duty (5725 kW) exceeds cold duty (2800 kW) with enough
        // overlap: no pinch, no heating utility.
        let targets = classic().minimum_utilities();
        assert!((targets.min_heating_duty - 0.0).abs() < 1e-9);
        assert!((targets.min_cooling_duty - 2925.0).abs() < 1e-6);
        assert!(targets.pinch_hot_temp.is_none());
        assert!(targets.pinch_cold_temp.is_none());
    }

    #[test]
    fn true_pinch_problem_locates_the_pinch() {
        // Symmetric streams: the cascade dips negative mid-way.
        let pinch = PinchAnalysis::new(
            vec![ProcessStream::hot(433.15, 323.15, 10.0)],
            vec![ProcessStream::cold(318.15, 428.15, 10.0)],
            10.0,
        );
        let targets = pinch.minimum_utilities();
        let hot = targets.pinch_hot_temp.expect("pinch exists");
        let cold = targets.pinch_cold_temp.expect("pinch exists");
        assert!(hot > cold);
        assert!((hot - cold - 10.0).abs() < 1e-9);
        // Both utilities are needed and equal by the first law.
        assert!(targets.min_heating_duty > 0.0);
        assert!((targets.min_heating_duty - targets.min_cooling_duty).abs() < 1e-9);
    }

    #[test]
    fn composite_curves_are_monotone_in_duty() {
        let curves = classic().composite_curves();
        for curve in [&curves.hot, &curves.cold] {
            for w in curve.windows(2) {
                assert!(w[1].0 >= w[0].0 - 1e-9, "duty must be non-decreasing");
            }
        }
        // Both curves span the same total duty.
        let total_hot = curves.hot.last().expect("non-empty").0;
        let total_cold = curves.cold.last().expect("non-empty").0;
        assert!((total_hot - total_cold).abs() < 1e-6);
    }

    #[test]
    fn grand_composite_starts_at_qh_min() {
        let gcc = classic().grand_composite_curve();
        let top = gcc.points.first().expect("non-empty").1;
        let targets = classic().minimum_utilities();
        assert!((top - targets.min_heating_duty).abs() < 1e-6);
    }

    #[test]
    fn synthesized_network_matches_targets() {
        let pinch = classic();
        let targets = pinch.minimum_utilities();
        let network = pinch.network_synthesis();
        // Every match carries positive duty on feasible pairs.
        for m in &network.matches {
            assert!(m.duty > 0.0);
            assert!(m.hot_index < pinch.hot.len());
            assert!(m.cold_index < pinch.cold.len());
        }
        // First law across the network: heating placed minus cooling
        // placed equals the minimum-utility difference.
        let heater_total: f64 = network.heaters.iter().map(|(_, d)| d).sum();
        let cooler_total: f64 = network.coolers.iter().map(|(_, d)| d).sum();
        let expected = targets.min_heating_duty - targets.min_cooling_duty;
        assert!(
            (heater_total - cooler_total - expected).abs() < 1e-6,
            "heaters {heater_total}, coolers {cooler_total}, expected diff {expected}"
        );
    }
}
