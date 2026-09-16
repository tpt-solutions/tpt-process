//! McCabe-Thiele binary distillation with constant relative volatility.

use crate::FugSpec;

/// McCabe-Thiele stage count result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MccabeResult {
    /// Equilibrium stages (including the reboiler; fractional).
    pub stages: f64,
    /// Feed stage position (1-based from the top).
    pub feed_stage: f64,
    /// Minimum reflux ratio from the pinch at the q-line intersection.
    pub min_reflux: f64,
}

/// Equilibrium curve for constant relative volatility: y = αx/(1+(α−1)x).
#[must_use]
pub fn equilibrium_y(x: f64, alpha: f64) -> f64 {
    alpha * x / (1.0 + (alpha - 1.0) * x)
}

/// Inverse equilibrium: x from y.
#[must_use]
pub fn equilibrium_x(y: f64, alpha: f64) -> f64 {
    y / (alpha - (alpha - 1.0) * y)
}

/// McCabe-Thiele stepping for a binary column at saturated-liquid feed.
///
/// Uses the FUG-consistent minimum reflux (pinch at the q-line/equilibrium
/// intersection), rectifying line through (x_D, x_D) with slope
/// R/(R+1), stripping line through (x_B, x_B) and the q-line
/// intersection. Steps stages until the intersection of the operating
/// lines is crossed (feed stage), then until x_B.
///
/// # Errors
/// Returns `Err` when compositions are not ordered 0 < x_B < x_D < 1 or
/// the reflux ratio is below the minimum (lines cannot reach).
pub fn mccabe_thiele_stages(
    spec: &FugSpec,
    x_distillate: f64,
    x_bottoms: f64,
) -> Result<MccabeResult, String> {
    let alpha = spec.relative_volatility;
    let z_f = spec.feed_mole_fraction;
    if !(x_bottoms < z_f && z_f < x_distillate && x_bottoms > 0.0 && x_distillate < 1.0) {
        return Err(format!(
            "compositions must satisfy 0 < x_B < z_F < x_D < 1 (got {x_bottoms}, {z_f}, {x_distillate})"
        ));
    }
    // q-line (saturated liquid): x = z_F. Pinch: equilibrium at x = z_F.
    let y_pinch = equilibrium_y(z_f, alpha);
    // R_min: rectifying line from (x_D, x_D) through (z_F, y_pinch):
    // slope = (x_D − y_pinch)/(x_D − z_F); R_min = slope/(1 − slope).
    let slope_min = (x_distillate - y_pinch) / (x_distillate - z_f);
    if slope_min >= 1.0 {
        return Err("pinch slope ≥ 1: separation infeasible at this α".into());
    }
    let r_min = slope_min / (1.0 - slope_min);
    let r = spec.actual_reflux_ratio;
    if r <= r_min {
        return Err(format!(
            "reflux {r} must exceed R_min {r_min:.4} for stepping"
        ));
    }

    // Operating lines.
    let rectifying = |x: f64| (r / (r + 1.0)) * x + x_distillate / (r + 1.0);
    // Stripping line passes through (x_B, x_B) and (z_F, rectifying(z_F)).
    let strip_slope = (rectifying(z_f) - x_bottoms) / (z_f - x_bottoms);
    let stripping = |x: f64| x_bottoms + strip_slope * (x - x_bottoms);

    // Step stages from the top.
    let mut y = x_distillate;
    let mut stages = 0.0_f64;
    let mut feed_stage = 0.0_f64;
    let mut above_feed = true;
    for _ in 0..500 {
        // Horizontal step to the equilibrium curve.
        let x = equilibrium_x(y, alpha);
        stages += 1.0;
        if above_feed && x < z_f {
            feed_stage = stages;
            above_feed = false;
        }
        if x <= x_bottoms {
            // Fractional last stage: interpolate within this step.
            let overshoot = (x_bottoms - x) / (y - equilibrium_y(x, alpha)).max(1e-12);
            stages -= overshoot.clamp(0.0, 1.0);
            if above_feed {
                feed_stage = stages;
            }
            break;
        }
        // Vertical step to the operating line (the operating line lies
        // below the equilibrium curve at every stage by design).
        y = if above_feed {
            rectifying(x)
        } else {
            stripping(x)
        };
    }
    if stages <= 0.0 || feed_stage <= 0.0 {
        return Err("stepping failed to reach the bottoms composition".into());
    }
    Ok(MccabeResult {
        stages,
        feed_stage,
        min_reflux: r_min,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> FugSpec {
        FugSpec {
            relative_volatility: 2.4,
            feed_mole_fraction: 0.5,
            light_key_recovery: 0.95,
            heavy_key_recovery: 0.05,
            actual_reflux_ratio: 1.5,
        }
    }

    #[test]
    fn equilibrium_curve_inverts() {
        for &x in &[0.1_f64, 0.3, 0.5, 0.7, 0.9] {
            let y = equilibrium_y(x, 2.4);
            assert!((equilibrium_x(y, 2.4) - x).abs() < 1e-9);
            assert!(y > x, "y must exceed x for α > 1 (x = {x})");
        }
        assert_eq!(equilibrium_y(0.0, 2.4), 0.0);
        assert_eq!(equilibrium_y(1.0, 2.4), 1.0);
    }

    #[test]
    fn stages_agree_with_fug_order_of_magnitude() {
        // 95/95 split of a 50/50 feed: x_D = 0.95, x_B = 0.05.
        let result = mccabe_thiele_stages(&spec(), 0.95, 0.05).unwrap();
        let fug = crate::fug_shortcut(&spec()).unwrap();
        // McCabe (constant α) and FUG should be within ~30% of each other.
        assert!(
            (result.stages - fug.actual_stages).abs() < 0.35 * fug.actual_stages,
            "McCabe {} vs FUG {}",
            result.stages,
            fug.actual_stages
        );
        // Feed stage sits in the middle for a symmetric split.
        assert!(
            (result.feed_stage - result.stages / 2.0).abs() < result.stages * 0.25,
            "feed stage {} of {}",
            result.feed_stage,
            result.stages
        );
    }

    #[test]
    fn rejects_infeasible_reflux() {
        // R_min for α = 2.4, pinch at z_F = 0.5: y_pinch = 0.7059;
        // slope = (0.95−0.7059)/(0.95−0.5) = 0.5425 → R_min = 1.186.
        assert!(mccabe_thiele_stages(&spec(), 0.95, 0.05).is_ok());
        assert!(mccabe_thiele_stages(
            &FugSpec {
                actual_reflux_ratio: 1.0,
                ..spec()
            },
            0.95,
            0.05
        )
        .is_err());
    }

    #[test]
    fn rejects_bad_composition_order() {
        assert!(mccabe_thiele_stages(&spec(), 0.3, 0.05).is_err());
        assert!(mccabe_thiele_stages(&spec(), 0.98, 0.6).is_err());
    }
}
