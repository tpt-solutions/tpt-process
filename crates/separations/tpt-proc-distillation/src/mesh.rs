//! Equilibrium-stage (MESH) column simulation at constant molal overflow.
//!
//! Lewis-Sorel stage-to-stage method for a binary at constant relative
//! volatility: guess the top-liquid composition, step down the column with
//! the rectifying and stripping operating lines (constant molar overflow
//! supplies the M lines, `y = αx/(1+(α−1)x)` the E and S relations), and
//! bisect on the guess until the reboiler liquid matches the bottoms from
//! the overall balance. The overall material balance then holds exactly by
//! construction.

use crate::FugSpec;

/// Column specification for stage-to-stage simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColumnSpec {
    /// Number of equilibrium stages including the reboiler (a total
    /// condenser is assumed on top and not counted).
    pub num_stages: u32,
    /// 1-based feed stage.
    pub feed_stage: u32,
    /// Reflux ratio L/D.
    pub reflux_ratio: f64,
    /// Distillate molar cut D/F ∈ (0, 1).
    pub distillate_cut: f64,
}

/// Simulation outcome.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshResult {
    /// Light-key mole fraction in the distillate.
    pub x_distillate: f64,
    /// Light-key mole fraction in the bottoms.
    pub x_bottoms: f64,
    /// Liquid light-key mole fraction per stage, top → bottom.
    pub liquid_profile: Vec<f64>,
    /// Converged flag.
    pub converged: bool,
    /// Outer bisection evaluations.
    pub iterations: u32,
}

/// Runs the MESH simulation.
///
/// # Errors
/// Returns `Err` for inconsistent specifications (stages, cut, reflux).
pub fn simulate(spec: &ColumnSpec, feed: &FugSpec) -> Result<MeshResult, String> {
    let n = spec.num_stages as usize;
    let f = spec.feed_stage as usize;
    if n < 3 || f < 1 || f > n {
        return Err("need at least 3 stages with 1 ≤ feed_stage ≤ num_stages".into());
    }
    if !(0.0..1.0).contains(&spec.distillate_cut) {
        return Err("distillate cut must be in (0, 1)".into());
    }
    if spec.reflux_ratio <= 0.0 {
        return Err("reflux ratio must be positive".into());
    }
    let alpha = feed.relative_volatility;
    let z = feed.feed_mole_fraction;

    // Normalized flows (F = 1), q = 1: vapor constant, liquid step at feed.
    let d = spec.distillate_cut;
    let b = 1.0 - d;
    let v = (spec.reflux_ratio + 1.0) * d;
    let l_above = spec.reflux_ratio * d;

    let eq = |x: f64| alpha * x / (1.0 + (alpha - 1.0) * x);
    let inv_eq = |y: f64| y / (alpha - (alpha - 1.0) * y);

    // Steps down the column from a guessed x1 (top-liquid composition).
    // Returns (bottom liquid stepped, bottoms from overall balance).
    let run = |x1: f64| -> (f64, f64, Vec<f64>) {
        let y1 = eq(x1);
        let x_d = y1; // total condenser
        let x_b_bal = (z - d * x_d) / b;
        let mut profile = vec![x1];
        let mut x = x1;
        // Stages 2..=n: vapor from the stage above has composition from
        // the section operating line; the liquid leaving the stage is its
        // equilibrium liquid.
        for stage in 2..=n {
            let y_n = if stage <= f {
                // Rectifying line through (x_D, x_D).
                (l_above / v) * x + x_d * (d / v)
            } else {
                // Stripping line through (x_B_bal, x_B_bal).
                ((l_above + 1.0) / v) * x - (b / v) * x_b_bal
            };
            if y_n <= 0.0 || y_n >= 1.0 {
                return (x, x_b_bal, profile.clone()); // off the feasible path
            }
            x = inv_eq(y_n);
            profile.push(x);
        }
        (x, x_b_bal, profile)
    };

    // Residual in x₁: bottom stepped − bottoms from balance (increasing).
    let residual = |x1: f64| -> f64 {
        let (bottom, x_b_bal, _) = run(x1);
        bottom - x_b_bal
    };

    // Bracket x₁ in (z, 0.999).
    let mut lo = z + 1e-6;
    let mut hi = 0.999_f64;
    let mut r_lo = residual(lo);
    let mut r_hi = residual(hi);
    let mut iterations = 0;
    while (r_lo > 0.0 || r_hi < 0.0) && iterations < 40 {
        iterations += 1;
        if r_lo > 0.0 {
            lo = z + (lo - z) * 0.5;
            r_lo = residual(lo);
        }
        if r_hi < 0.0 {
            hi = hi + (1.0 - hi) * 0.5;
            r_hi = residual(hi);
        }
    }
    if r_lo > 0.0 || r_hi < 0.0 {
        return Err("could not bracket the top composition; column spec infeasible".into());
    }
    let mut converged = false;
    for _ in 0..80 {
        iterations += 1;
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            converged = true;
            break;
        }
        if residual(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    let x1 = 0.5 * (lo + hi);
    let (bottom, x_b_bal, profile) = run(x1);
    Ok(MeshResult {
        x_distillate: eq(x1),
        x_bottoms: (0.5 * (bottom + x_b_bal)).clamp(0.0, 1.0),
        liquid_profile: profile,
        converged,
        iterations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> ColumnSpec {
        ColumnSpec {
            num_stages: 14,
            feed_stage: 7,
            reflux_ratio: 2.0,
            distillate_cut: 0.5,
        }
    }

    fn feed() -> FugSpec {
        FugSpec {
            relative_volatility: 2.4,
            feed_mole_fraction: 0.5,
            light_key_recovery: 0.95,
            heavy_key_recovery: 0.05,
            actual_reflux_ratio: 2.0,
        }
    }

    #[test]
    fn column_separates_and_closes_mass_balance() {
        let result = simulate(&spec(), &feed()).unwrap();
        assert!(result.converged);
        // Light key enriches toward the top.
        assert!(result.x_distillate > 0.6, "x_D = {}", result.x_distillate);
        assert!(result.x_bottoms < 0.4, "x_B = {}", result.x_bottoms);
        // Overall component balance: D·x_D + B·x_B = F·z = 0.5 at cut 0.5.
        let recovered = 0.5 * result.x_distillate + 0.5 * result.x_bottoms;
        assert!(
            (recovered - 0.5).abs() < 0.05,
            "component balance closure {recovered}"
        );
        // Profile is monotonically decreasing top → bottom.
        for w in result.liquid_profile.windows(2) {
            assert!(w[1] <= w[0] + 1e-6, "profile must decrease: {w:?}");
        }
    }

    #[test]
    fn higher_reflux_separates_better() {
        let low_r = simulate(
            &ColumnSpec {
                reflux_ratio: 1.0,
                ..spec()
            },
            &feed(),
        )
        .unwrap();
        let high_r = simulate(
            &ColumnSpec {
                reflux_ratio: 4.0,
                ..spec()
            },
            &feed(),
        )
        .unwrap();
        assert!(
            high_r.x_distillate - high_r.x_bottoms > low_r.x_distillate - low_r.x_bottoms,
            "more reflux should widen the split"
        );
    }

    #[test]
    fn rejects_bad_specifications() {
        assert!(simulate(
            &ColumnSpec {
                num_stages: 2,
                ..spec()
            },
            &feed()
        )
        .is_err());
        assert!(simulate(
            &ColumnSpec {
                distillate_cut: 1.5,
                ..spec()
            },
            &feed()
        )
        .is_err());
    }
}
