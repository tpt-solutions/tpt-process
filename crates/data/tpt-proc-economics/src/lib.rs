//! Process economics: capital cost scaling, operating cost, cash-flow
//! analysis (NPV, discounted payback), IRR, and levelized cost.
//!
//! # Example
//!
//! ```
//! use tpt_proc_economics::{cashflows, npv, six_tenths_capex};
//!
//! // Six-tenths rule: doubling capacity raises capex by ~2^0.6 ≈ 1.52.
//! let capex2 = six_tenths_capex(100.0e6, 1.0, 2.0, 0.6);
//! assert!((capex2 - 100.0e6 * 2.0_f64.powf(0.6)).abs() < 1e-3);
//!
//! // A project investing 100 M and returning 25 M/a for 5 years
//! // (25·3.79 ≈ 94.8 M present value) misses a 10% hurdle.
//! let flows = cashflows(-100.0e6, 25.0e6, 5);
//! let value = npv(0.10, &flows);
//! assert!(value < 0.0);
//! ```

#![forbid(unsafe_code)]

/// Capital cost by the six-tenths rule: `C2 = C1·(Q2/Q1)^exponent`.
#[must_use]
pub fn six_tenths_capex(
    capex_reference: f64,
    capacity_reference: f64,
    capacity: f64,
    exponent: f64,
) -> f64 {
    if capacity_reference <= 0.0 {
        return f64::NAN;
    }
    capex_reference * (capacity / capacity_reference).powf(exponent)
}

/// Annual operating cost: fixed plus proportional-to-capacity variable.
#[must_use]
pub fn annual_opex(fixed: f64, variable_per_unit: f64, annual_throughput: f64) -> f64 {
    fixed + variable_per_unit * annual_throughput.max(0.0)
}

/// Net present value of a cash-flow series (year 0 first), with the
/// discount `rate` as a fraction.
#[must_use]
pub fn npv(rate: f64, cashflows: &[f64]) -> f64 {
    cashflows
        .iter()
        .enumerate()
        .map(|(t, cf)| cf / (1.0 + rate).powi(t as i32))
        .sum()
}

/// Builds a simple cash-flow series: an initial investment followed by a
/// constant annual margin for `years`.
#[must_use]
pub fn cashflows(investment: f64, annual_margin: f64, years: u32) -> Vec<f64> {
    let mut flows = vec![investment];
    for _ in 0..years {
        flows.push(annual_margin);
    }
    flows
}

/// Internal rate of return by bisection on the NPV curve.
/// Returns `None` when no sign change exists in the searched rate range.
#[must_use]
pub fn irr(cashflows: &[f64]) -> Option<f64> {
    let npv_at = |rate: f64| npv(rate, cashflows);
    let (mut lo, mut hi) = (-0.9_f64, 10.0_f64);
    let f_lo = npv_at(lo);
    let f_hi = npv_at(hi);
    if f_lo * f_hi > 0.0 {
        return None;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break;
        }
        let f_mid = npv_at(mid);
        if f_lo * f_mid <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
            let _ = f_lo;
        }
    }
    Some(0.5 * (lo + hi))
}

/// Discounted payback time, years, from a cash-flow series.
/// Returns `None` if the initial outflow is never recovered.
#[must_use]
pub fn discounted_payback(rate: f64, cashflows: &[f64]) -> Option<f64> {
    let first = *cashflows.first()?;
    if first >= 0.0 {
        return Some(0.0);
    }
    let mut cumulative = first;
    for (t, &cf) in cashflows.iter().enumerate().skip(1) {
        let discounted = cf / (1.0 + rate).powi(t as i32);
        if cumulative + discounted >= 0.0 {
            let fraction = -cumulative / discounted;
            return Some(t as f64 - 1.0 + fraction);
        }
        cumulative += discounted;
    }
    None
}

/// Levelized cost of a product: (annualized capex + annual opex) /
/// annual production.
#[must_use]
pub fn levelized_cost(
    capex: f64,
    annual_opex: f64,
    annual_production: f64,
    discount_rate: f64,
    life_years: u32,
) -> f64 {
    if annual_production <= 0.0 || life_years == 0 {
        return f64::NAN;
    }
    // Capital recovery factor.
    let r = discount_rate;
    let crf = if r <= 0.0 {
        1.0 / f64::from(life_years)
    } else {
        r / (1.0 - (1.0 + r).powi(-(life_years as i32)))
    };
    (capex * crf + annual_opex) / annual_production
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_tenths_scales_sublinearly() {
        let c2 = six_tenths_capex(100.0e6, 1.0, 2.0, 0.6);
        assert!((c2 / (100.0e6) - 1.5157).abs() < 1e-3, "c2 = {c2}");
    }

    #[test]
    fn npv_of_par_bond() {
        // A bond paying 10% on 1000 for 5 years at a 10% discount is worth
        // par: NPV = 0. Flows: -1000, then 5 × 100 coupons, principal at
        // year 5.
        let mut flows = vec![-1000.0];
        flows.extend([100.0, 100.0, 100.0, 100.0, 1100.0]);
        assert!(
            npv(0.10, &flows).abs() < 1e-6,
            "npv = {}",
            npv(0.10, &flows)
        );
    }

    #[test]
    fn irr_of_simple_project() {
        // -100 now, +130 next year → IRR = 30%.
        let irr_value = irr(&[-100.0, 130.0]).expect("sign change exists");
        assert!((irr_value - 0.30).abs() < 1e-9);
    }

    #[test]
    fn payback_of_simple_project() {
        // -100, +50/a: simple payback in 2 years; discounted at 10% ≈ 2.34.
        let flows = cashflows(-100.0, 50.0, 10);
        let pb = discounted_payback(0.10, &flows).expect("pays back");
        assert!(pb > 2.0 && pb < 3.0, "payback = {pb}");
        // Never recovered → None.
        assert!(discounted_payback(0.10, &[-100.0, 10.0, 10.0]).is_none());
    }

    #[test]
    fn levelized_cost_of_a_small_plant() {
        // Capex 500 M, opex 50 M/a, 2 Mt/a product, 10% discount, 20 y.
        let lc = levelized_cost(500.0e6, 50.0e6, 2.0e6, 0.10, 20);
        // CRF ≈ 0.1175 → annualized ≈ 108.7 M → 54.4 per t.
        assert!(lc > 50.0 && lc < 60.0, "LC = {lc}");
    }
}
