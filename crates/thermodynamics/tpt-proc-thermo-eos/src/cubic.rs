//! Robust cubic root solver.
//!
//! Solves `z³ + c2·z² + c1·z + c0 = 0` analytically (depressed cubic +
//! trigonometric method for the three-real-root case). Analytic roots are
//! the right choice inside a flash iteration: no convergence parameter to
//! tune, no iteration to fail, bit-identical results.

/// Real roots of `z³ + c2 z² + c1 z + c0 = 0`, ascending, duplicates
/// included (1, 2, or 3 entries).
#[must_use]
pub fn solve_cubic(c2: f64, c1: f64, c0: f64) -> Vec<f64> {
    // Depress: z = x − c2/3 → x³ + p·x + q = 0.
    let p = c1 - c2 * c2 / 3.0;
    let q = 2.0 * c2 * c2 * c2 / 27.0 - c2 * c1 / 3.0 + c0;
    let shift = -c2 / 3.0;

    // Discriminant of the depressed cubic, with a scale-relative zero test:
    // near-multiple roots produce |D| at rounding-noise level, and the
    // double/triple-root closed forms are far more accurate than Cardano
    // there.
    let discriminant = -4.0 * p * p * p - 27.0 * q * q;
    let discriminant_scale = 4.0 * p.abs().powi(3) + 27.0 * q * q;
    let discriminant = if discriminant.abs() < 1e-12 * discriminant_scale {
        0.0
    } else {
        discriminant
    };

    if discriminant > 0.0 {
        // Three distinct real roots (trigonometric/Viète form).
        let m = 2.0 * (-p / 3.0).sqrt();
        let theta = (3.0 * q / (p * m)).acos() / 3.0;
        let r1 = shift + m * (theta + 2.0 * std::f64::consts::PI / 3.0).cos();
        let r2 = shift + m * theta.cos();
        let r3 = shift + m * (theta - 2.0 * std::f64::consts::PI / 3.0).cos();
        let mut roots = vec![r1, r2, r3];
        roots.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        roots
    } else if discriminant < 0.0 {
        // One real root (Cardano): Δ = (q/2)² + (p/3)³ = −D/108 > 0.
        let s = (-discriminant / 108.0).sqrt();
        let t = (-q / 2.0 + s).cbrt();
        let u = (-q / 2.0 - s).cbrt();
        vec![shift + t + u]
    } else {
        // Discriminant == 0: multiple real roots.
        if p.abs() < 1e-300 {
            // Triple root at the shift.
            return vec![shift, shift, shift];
        }
        let r_double = -3.0 * q / (2.0 * p); // double root
        let r_simple = -2.0 * r_double; // simple root
        sort3(r_double + shift, r_double + shift, r_simple + shift)
    }
}

fn sort3(a: f64, b: f64, c: f64) -> Vec<f64> {
    let mut v = vec![a, b, c];
    v.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9 * (1.0 + a.abs().max(b.abs()))
    }

    #[test]
    fn three_real_roots() {
        // (z−1)(z−2)(z−3) = z³ − 6z² + 11z − 6
        let roots = solve_cubic(-6.0, 11.0, -6.0);
        assert_eq!(roots.len(), 3);
        assert!(approx(roots[0], 1.0) && approx(roots[1], 2.0) && approx(roots[2], 3.0));
    }

    #[test]
    fn one_real_root() {
        // (z−2)(z²+1) = z³ − 2z² + z − 2
        let roots = solve_cubic(-2.0, 1.0, -2.0);
        assert_eq!(roots.len(), 1);
        assert!(approx(roots[0], 2.0));
    }

    #[test]
    fn double_root() {
        // (z−1)²(z+2) = z³ − 3z + 2
        let roots = solve_cubic(0.0, -3.0, 2.0);
        assert!(approx(roots[0], -2.0) && approx(roots[1], 1.0) && approx(roots[2], 1.0));
    }

    #[test]
    fn triple_root() {
        // (z−1)³ = z³ − 3z² + 3z − 1
        let roots = solve_cubic(-3.0, 3.0, -1.0);
        assert_eq!(roots.len(), 3);
        assert!(roots.iter().all(|r| approx(*r, 1.0)));
    }

    #[test]
    fn linear_degenerate() {
        // z³ − z² = z²(z−1)
        let roots = solve_cubic(-1.0, 0.0, 0.0);
        assert!(approx(roots[0], 0.0) && approx(roots[1], 0.0) && approx(roots[2], 1.0));
    }
}
