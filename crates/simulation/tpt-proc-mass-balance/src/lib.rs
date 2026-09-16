//! Mass balance accounting and weighted data reconciliation.
//!
//! # Example
//!
//! ```
//! use tpt_proc_mass_balance::{reconcile_flows, BalanceReport};
//!
//! // A splitter node: measured 100 in, 48 + 55 out (103) — over-closed.
//! let report = reconcile_flows(&[("feed", 100.0, 2.0), ("p1", 48.0, 2.0), ("p2", 55.0, 2.0)],
//!                              &[("feed", 1.0), ("p1", -1.0), ("p2", -1.0)]);
//! assert!(report.residual.abs() < 1e-9);
//! ```
//!
//! where each constraint is (stream name, sign) and each measurement is
//! (name, value, standard deviation).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// A single mass-balance node report.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct BalanceReport {
    /// Total inflow, kg/s.
    pub inflow: f64,
    /// Total outflow, kg/s.
    pub outflow: f64,
    /// Inflow − outflow residual, kg/s (zero for a closed balance).
    pub residual: f64,
    /// Relative closure error |residual| / inflow.
    pub closure_error: f64,
}

/// Computes the overall mass balance of a node.
#[must_use]
pub fn overall_balance(inflows: &[f64], outflows: &[f64]) -> BalanceReport {
    let in_total: f64 = inflows.iter().sum();
    let out_total: f64 = outflows.iter().sum();
    let residual = in_total - out_total;
    BalanceReport {
        inflow: in_total,
        outflow: out_total,
        residual,
        closure_error: if in_total > 0.0 {
            residual.abs() / in_total
        } else {
            0.0
        },
    }
}

/// Component-wise balance over multiple nodes.
///
/// `component_flows[node] = (inflows, outflows)` in kg/s of the component.
#[must_use]
pub fn component_balance(
    component_flows: &BTreeMap<String, (Vec<f64>, Vec<f64>)>,
) -> BTreeMap<String, BalanceReport> {
    component_flows
        .iter()
        .map(|(name, (ins, outs))| (name.clone(), overall_balance(ins, outs)))
        .collect()
}

/// Weighted least-squares flow reconciliation for one node.
///
/// Measurements `(name, value, sigma)` are adjusted minimally (in a
/// χ²-sense weighted by 1/σ²) so that the signed constraint
/// `Σ signᵢ·xᵢ = 0` closes exactly. Returns the reconciled values with
/// the residual of the adjustment.
///
/// # Panics
/// Never — degenerate inputs (zero total weight) return the raw
/// measurements with the un-closed residual folded into the report.
#[must_use]
pub fn reconcile_flows(
    measurements: &[(&str, f64, f64)],
    constraints: &[(&str, f64)],
) -> Reconciliation {
    let mut values: BTreeMap<String, f64> = BTreeMap::new();
    let mut weights: BTreeMap<String, f64> = BTreeMap::new();
    for (name, value, sigma) in measurements {
        values.insert((*name).to_string(), *value);
        weights.insert(
            (*name).to_string(),
            if *sigma > 0.0 {
                1.0 / (sigma * sigma)
            } else {
                1e12
            },
        );
    }
    // Constraint residual with current values.
    let mut imbalance = 0.0_f64;
    let mut weight_sum = 0.0_f64;
    for (name, sign) in constraints {
        let v = values.get(*name).copied().unwrap_or(0.0);
        imbalance += sign * v;
        weight_sum += weights.get(*name).copied().unwrap_or(1.0);
    }
    // Closed-form projection: each measured value moves by
    // Δxᵢ = −(signᵢ/ωᵢ²)·(imbalance / Σ 1/ωᵢ)... normalized:
    // Δxᵢ = −signᵢ·(imbalance)/(ωᵢ·Σ(signⱼ²/ωⱼ)) — with sign² = 1:
    // Δxᵢ = −signᵢ·imbalance/(ωᵢ·W) where W = Σ 1/ωⱼ.
    let inverse_weight_sum: f64 = constraints
        .iter()
        .map(|(name, _)| 1.0 / weights.get(*name).copied().unwrap_or(1.0))
        .sum();
    for (name, sign) in constraints {
        let w = weights.get(*name).copied().unwrap_or(1.0);
        let delta = -sign * imbalance / (w * inverse_weight_sum.max(1e-300));
        if let Some(v) = values.get_mut(*name) {
            *v += delta;
        }
    }
    let closed: f64 = constraints
        .iter()
        .map(|(name, sign)| sign * values.get(*name).copied().unwrap_or(0.0))
        .sum();
    Reconciliation {
        values,
        residual: closed,
        weight_sum,
    }
}

/// Reconciled measurement set.
#[derive(Clone, Debug, PartialEq)]
pub struct Reconciliation {
    /// Reconciled values by stream name.
    pub values: BTreeMap<String, f64>,
    /// Post-reconciliation constraint residual (≈ 0 when successful).
    pub residual: f64,
    /// Total statistical weight used.
    pub weight_sum: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_node_reports_zero_residual() {
        let report = overall_balance(&[100.0], &[60.0, 40.0]);
        assert!(report.residual.abs() < 1e-12);
        assert_eq!(report.closure_error, 0.0);
    }

    #[test]
    fn unbalanced_node_reports_closure() {
        let report = overall_balance(&[100.0], &[60.0, 30.0]);
        assert!((report.residual - 10.0).abs() < 1e-12);
        assert!((report.closure_error - 0.1).abs() < 1e-12);
    }

    #[test]
    fn component_balance_is_per_component() {
        let mut flows = BTreeMap::new();
        flows.insert("ethanol".to_string(), (vec![50.0], vec![30.0, 20.0]));
        flows.insert("water".to_string(), (vec![150.0], vec![150.0]));
        let reports = component_balance(&flows);
        assert!(reports["ethanol"].residual.abs() < 1e-12);
        assert!(reports["water"].residual.abs() < 1e-12);
    }

    #[test]
    fn reconciliation_closes_the_balance() {
        let report = reconcile_flows(
            &[("feed", 100.0, 2.0), ("p1", 48.0, 2.0), ("p2", 55.0, 2.0)],
            &[("feed", 1.0), ("p1", -1.0), ("p2", -1.0)],
        );
        assert!(report.residual.abs() < 1e-9, "residual {}", report.residual);
        // Outlets (103) exceed feed (100) by 3; the LSQ projection moves
        // every equally-weighted value by 1: feed 101, outlets 47/54.
        assert!(
            (report.values["feed"] - 101.0).abs() < 1e-9,
            "feed {}",
            report.values["feed"]
        );
        assert!((report.values["p1"] - 47.0).abs() < 1e-9);
        assert!((report.values["p2"] - 54.0).abs() < 1e-9);
    }

    #[test]
    fn precise_measurements_move_less() {
        // feed measured tightly, outlets loosely: outlets absorb the error.
        let report = reconcile_flows(
            &[("feed", 100.0, 0.01), ("p1", 48.0, 5.0), ("p2", 55.0, 5.0)],
            &[("feed", 1.0), ("p1", -1.0), ("p2", -1.0)],
        );
        assert!((report.values["feed"] - 100.0).abs() < 1e-4);
        assert!((report.values["p1"] + report.values["p2"] - 100.0).abs() < 1e-4);
    }
}
