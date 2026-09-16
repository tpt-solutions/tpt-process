//! Pure-component vapor pressure by EOS fugacity equality.
//!
//! At saturation, `ln φ_liq = ln φ_vap` for the pure component. The
//! residual `g(P) = ln φ_liq − ln φ_vap` decreases monotonically in P
//! between the two three-root regions bracketing the dome, so a bracketed
//! bisection converges deterministically in ~60 steps to machine precision.
//!
//! Domain: 0.3·Tc < T < Tc. Below ~0.4·Tc the cubic saturation pressure
//! deviates sharply from experiment (documented limitation); above Tc no
//! two-phase equilibrium exists.

use tpt_proc_core::Composition;
use tpt_proc_thermo_core::{Result, ThermoError};

use crate::CubicEos;

/// Pure-component saturation pressure, Pa.
///
/// # Errors
/// [`ThermoError::StateOutOfDomain`] outside 0.3·Tc..Tc;
/// [`ThermoError::Numeric`] when the bracket fails.
pub fn pure_vapor_pressure(eos: &CubicEos, i: usize, temperature: f64) -> Result<f64> {
    let component = &eos.components()[i];
    let tc = component.critical_temperature;
    if temperature <= 0.3 * tc || temperature >= tc {
        return Err(ThermoError::StateOutOfDomain(format!(
            "{}: vapor pressure requested at T={temperature} K (valid ~0.3·Tc..Tc = {:.1}..{tc:.1})",
            component.name,
            0.3 * tc
        )));
    }

    let pure = Composition::pure(eos.components().len(), i)
        .map_err(|e| ThermoError::Numeric(e.to_string()))?;

    // g(P) = ln φ_liq − ln φ_vap: positive at low P, negative at high P.
    let residual = |p: f64| -> Result<f64> {
        let roots = eos.compressibility_roots(&pure, temperature, p)?;
        if roots.len() < 2 {
            return Err(ThermoError::Numeric(format!(
                "single root at P={p}: outside the two-phase dome"
            )));
        }
        let z_l = roots[0];
        let z_v = *roots.last().expect("len >= 2");
        let ln_l = eos.ln_phi_at_root(&pure, temperature, p, z_l)?[0];
        let ln_v = eos.ln_phi_at_root(&pure, temperature, p, z_v)?[0];
        Ok(ln_l - ln_v)
    };

    // Scan upward from below the dome: the residual is defined (three real
    // roots) only inside the metastable interval around Psat, where it is
    // positive below saturation and negative above. Remember the last
    // positive point; the first negative point completes the bracket.
    let p_crit = component.critical_pressure;
    let mut p = p_crit * 1e-7;
    let mut last_positive: Option<f64> = None;
    let mut bracket: Option<(f64, f64)> = None;
    while p < 2.0 * p_crit {
        if let Ok(g) = residual(p) {
            if g > 0.0 {
                last_positive = Some(p);
            } else if g < 0.0 {
                if let Some(lo) = last_positive {
                    bracket = Some((lo, p));
                    break;
                }
            }
        }
        p *= 1.4;
    }
    let Some((lo, hi)) = bracket else {
        return Err(ThermoError::Numeric(
            "could not bracket the saturation pressure".into(),
        ));
    };

    // Bisection to machine precision.
    let mut lo = lo;
    let mut hi = hi;
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break; // interval exhausted at floating-point resolution
        }
        let g_mid = residual(mid)?;
        if g_mid > 0.0 {
            lo = mid;
        } else if g_mid < 0.0 {
            hi = mid;
        } else {
            return Ok(mid);
        }
    }
    let p_sat = 0.5 * (lo + hi);
    Ok(p_sat)
}
