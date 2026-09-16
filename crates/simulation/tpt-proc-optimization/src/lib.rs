//! Unconstrained optimizers: golden-section search, Nelder-Mead, and
//! gradient descent. Zero dependencies.
//!
//! # Example
//!
//! ```
//! use tpt_proc_optimization::{golden_section, nelder_mead};
//!
//! // Minimize (x − 2)² on [0, 5].
//! let (x, fx) = golden_section(|x| (x - 2.0).powi(2), 0.0, 5.0, 1e-9);
//! assert!((x - 2.0).abs() < 1e-6);
//!
//! // Minimize the 2-D Rosenbrock-style bowl.
//! let start = [3.0, 3.0];
//! let (point, fx) = nelder_mead(|p| p[0].powi(2) + 10.0 * p[1].powi(2), &start, 1e-10, 2000);
//! assert!(fx < 1e-8);
//! ```

#![forbid(unsafe_code)]

/// Golden-section minimization of a unimodal `f` on `[a, b]`.
/// Returns (x*, f(x*)).
#[must_use]
pub fn golden_section(f: impl Fn(f64) -> f64, a: f64, b: f64, tolerance: f64) -> (f64, f64) {
    let ratio = (5.0_f64.sqrt() - 1.0) / 2.0; // 1/φ ≈ 0.618
    let (mut lo, mut hi) = (a.min(b), a.max(b));
    if (hi - lo) <= tolerance {
        let x = 0.5 * (lo + hi);
        return (x, f(x));
    }
    let mut c = hi - ratio * (hi - lo);
    let mut d = lo + ratio * (hi - lo);
    let mut fc = f(c);
    let mut fd = f(d);
    while (hi - lo) > tolerance {
        if fc < fd {
            hi = d;
            d = c;
            fd = fc;
            c = hi - ratio * (hi - lo);
            fc = f(c);
        } else {
            lo = c;
            c = d;
            fc = fd;
            d = lo + ratio * (hi - lo);
            fd = f(d);
        }
    }
    let x = 0.5 * (lo + hi);
    (x, f(x))
}

/// Nelder-Mead simplex minimization over an n-dimensional box.
/// Returns (best point, f(best)).
#[must_use]
pub fn nelder_mead(
    f: impl Fn(&[f64]) -> f64,
    start: &[f64],
    tolerance: f64,
    max_iterations: u32,
) -> (Vec<f64>, f64) {
    let n = start.len().max(1);
    // Initial simplex: start plus one unit-offset vertex per dimension.
    let mut vertices: Vec<Vec<f64>> = vec![start.to_vec()];
    for i in 0..n {
        let mut v = start.to_vec();
        let scale = if start[i].abs() > 0.0 {
            start[i] * 0.05 + 1e-4
        } else {
            1e-2
        };
        v[i] += scale;
        vertices.push(v);
    }
    let mut values: Vec<f64> = vertices.iter().map(|v| f(v)).collect();

    for _ in 0..max_iterations {
        // Sort by value.
        let mut order: Vec<usize> = (0..vertices.len()).collect();
        order.sort_by(|&i, &j| {
            values[i]
                .partial_cmp(&values[j])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        vertices = order.iter().map(|&i| vertices[i].clone()).collect();
        values = order.iter().map(|&i| values[i]).collect();

        if (values.last().expect("non-empty") - values[0]).abs() < tolerance {
            break;
        }

        // Centroid of all but the worst.
        let m = vertices.len() - 1;
        let mut centroid = vec![0.0; n];
        for v in &vertices[..m] {
            for (ci, &vi) in centroid.iter_mut().zip(v) {
                *ci += vi / m as f64;
            }
        }
        let reflect = |coeff: f64| -> Vec<f64> {
            let worst = &vertices[m];
            centroid
                .iter()
                .zip(worst)
                .map(|(c, w)| c + coeff * (c - w))
                .collect()
        };

        let worst = values[m];
        let second_worst = values[m - 1];
        let reflected = reflect(1.0);
        let fr = f(&reflected);
        if fr < values[0] {
            // Expand.
            let expanded = reflect(2.0);
            let fe = f(&expanded);
            if fe < fr {
                vertices[m] = expanded;
                values[m] = fe;
            } else {
                vertices[m] = reflected;
                values[m] = fr;
            }
        } else if fr < second_worst {
            vertices[m] = reflected;
            values[m] = fr;
        } else {
            // Contract.
            let contracted = reflect(0.5);
            let fc = f(&contracted);
            if fc < worst {
                vertices[m] = contracted;
                values[m] = fc;
            } else {
                // Shrink toward the best.
                let best = vertices[0].clone();
                for v in vertices.iter_mut().skip(1) {
                    for (vi, b) in v.iter_mut().zip(&best) {
                        *vi = (*vi + b) / 2.0;
                    }
                }
                values = vertices.iter().map(|v| f(v)).collect();
            }
        }
    }

    // Final best.
    let mut best_i = 0;
    for (i, &v) in values.iter().enumerate() {
        if v < values[best_i] {
            best_i = i;
        }
    }
    (vertices[best_i].clone(), values[best_i])
}

/// Gradient descent with a fixed step: minimize `f` given `grad`.
/// Returns the final point.
#[must_use]
pub fn gradient_descent(
    grad: impl Fn(&[f64]) -> Vec<f64>,
    start: &[f64],
    step: f64,
    iterations: u32,
) -> Vec<f64> {
    let mut x = start.to_vec();
    for _ in 0..iterations {
        let g = grad(&x);
        let mut moved = false;
        for (xi, gi) in x.iter_mut().zip(&g) {
            let next = *xi - step * gi;
            if (next - *xi).abs() > 1e-15 {
                moved = true;
            }
            *xi = next;
        }
        if !moved {
            break;
        }
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_section_finds_quadratic_minimum() {
        let (x, fx) = golden_section(|v| (v - 2.0).powi(2), 0.0, 5.0, 1e-9);
        assert!((x - 2.0).abs() < 1e-5);
        assert!(fx.abs() < 1e-9);
    }

    #[test]
    fn golden_section_unordered_bounds() {
        let (x, _) = golden_section(|v| (v + 1.0).powi(2), 3.0, -4.0, 1e-9);
        assert!((x + 1.0).abs() < 1e-5);
    }

    #[test]
    fn nelder_mead_solves_rosenbrock() {
        // Rosenbrock with a = 1, b = 100 (minimum at (1, 1)).
        let rosenbrock = |p: &[f64]| (1.0 - p[0]).powi(2) + 100.0 * (p[1] - p[0] * p[0]).powi(2);
        let (point, fx) = nelder_mead(rosenbrock, &[-1.2, 1.0], 1e-12, 5000);
        assert!(fx < 1e-8, "fx = {fx}");
        assert!((point[0] - 1.0).abs() < 1e-2 && (point[1] - 1.0).abs() < 1e-2);
    }

    #[test]
    fn gradient_descent_descends_a_bowl() {
        let grad = |p: &[f64]| vec![2.0 * p[0], 20.0 * p[1]];
        let x = gradient_descent(grad, &[5.0, 3.0], 0.05, 500);
        assert!(x[0].abs() < 0.1 && x[1].abs() < 0.1, "x = {x:?}");
    }
}
