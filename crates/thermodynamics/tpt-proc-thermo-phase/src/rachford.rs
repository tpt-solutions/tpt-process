//! Rachford-Rice vapor-fraction solve.
//!
//! `g(β) = Σᵢ zᵢ(Kᵢ−1)/(1+β(Kᵢ−1)) = 0` is strictly decreasing on its
//! bracket when the flash is genuinely two-phase, so bisection is
//! bulletproof and deterministic: no divergence, no tuning, ~50 iterations
//! to machine precision. A Newton polish is unnecessary at this cost.

/// Solves the Rachford-Rice equation for the vapor fraction β ∈ [0, 1].
///
/// Returns `(beta, bracketed)`: when the K set has no interior root (the
/// state is single-phase by the current K estimate), β is clamped to the
/// appropriate boundary (0 or 1) with `bracketed = false` so the caller can
/// keep iterating K-values — the classification then firms up as the Ks
/// converge.
pub fn solve(z: &[f64], k: &[f64], max_iterations: u32) -> Result<(f64, bool), crate::FlashError> {
    let g = |beta: f64| -> f64 {
        z.iter()
            .zip(k)
            .map(|(zi, ki)| zi * (ki - 1.0) / (1.0 + beta * (ki - 1.0)))
            .sum()
    };

    // g(0) = Σz(K−1) > 0 and g(1) = Σz(K−1)/K < 0 for a valid two-phase
    // flash.
    let g0 = g(0.0);
    let g1 = g(1.0);
    if g0 <= 0.0 {
        return Ok((0.0, false)); // subcooled liquid by current K
    }
    if g1 >= 0.0 {
        return Ok((1.0, false)); // superheated vapor by current K
    }

    let mut lo = 0.0_f64;
    let mut hi = 1.0_f64;
    for _ in 0..max_iterations {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break; // interval exhausted at floating-point resolution
        }
        let g_mid = g(mid);
        if g_mid > 0.0 {
            lo = mid;
        } else if g_mid < 0.0 {
            hi = mid;
        } else {
            return Ok((mid, true));
        }
    }
    Ok((0.5 * (lo + hi), true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_typical_k_set() {
        let z = [0.4, 0.3, 0.3];
        let k = [3.0, 1.0, 0.5];
        let (beta, bracketed) = solve(&z, &k, 100).unwrap();
        assert!(bracketed);
        let g: f64 = z
            .iter()
            .zip(k)
            .map(|(zi, ki)| zi * (ki - 1.0) / (1.0 + beta * (ki - 1.0)))
            .sum();
        assert!(g.abs() < 1e-10);
    }

    #[test]
    fn clamps_single_phase_k_sets() {
        // All K < 1 → clamped to liquid boundary.
        let (beta, bracketed) = solve(&[0.5, 0.5], &[0.2, 0.3], 10).unwrap();
        assert!(!bracketed);
        assert_eq!(beta, 0.0);
        // All K > 1 → clamped to vapor boundary.
        let (beta, bracketed) = solve(&[0.5, 0.5], &[3.0, 4.0], 10).unwrap();
        assert!(!bracketed);
        assert_eq!(beta, 1.0);
    }

    #[test]
    fn beta_bounds_are_respected() {
        let z = [0.9, 0.1];
        let k = [10.0, 0.01];
        let (beta, _) = solve(&z, &k, 100).unwrap();
        assert!((0.0..=1.0).contains(&beta));
    }
}
