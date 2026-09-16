//! Dynamic simulation: RK4 integration, first-order-plus-dead-time (FOPDT)
//! process models, and step-response analysis.
//!
//! # Example
//!
//! ```
//! use tpt_proc_dynamics::{rk4, IntegratorState};
//!
//! // Exponential decay dx/dt = −x, from x = 1 over 1 s.
//! let state = IntegratorState { time: 0.0, variables: vec![1.0] };
//! let final_state = rk4(state, 1.0, 100, |_, x| vec![-x[0]]);
//! assert!((final_state.variables[0] - (-1.0_f64).exp()).abs() < 1e-9);
//! ```

#![forbid(unsafe_code)]

/// An integration state: time plus the state vector.
#[derive(Clone, Debug, PartialEq)]
pub struct IntegratorState {
    /// Current time.
    pub time: f64,
    /// State variables.
    pub variables: Vec<f64>,
}

/// Fixed-step RK4 integration of `dx/dt = f(t, x)` from `state` over
/// `duration`, in `steps` steps. Returns the final state.
#[must_use]
pub fn rk4(
    state: IntegratorState,
    duration: f64,
    steps: u32,
    f: impl Fn(f64, &[f64]) -> Vec<f64>,
) -> IntegratorState {
    let steps = steps.max(1);
    let h = duration / f64::from(steps);
    let mut t = state.time;
    let mut x = state.variables;
    for _ in 0..steps {
        let k1 = f(t, &x);
        let x2: Vec<f64> = x
            .iter()
            .zip(&k1)
            .map(|(xi, ki)| xi + 0.5 * h * ki)
            .collect();
        let k2 = f(t + 0.5 * h, &x2);
        let x3: Vec<f64> = x
            .iter()
            .zip(&k2)
            .map(|(xi, ki)| xi + 0.5 * h * ki)
            .collect();
        let k3 = f(t + 0.5 * h, &x3);
        let x4: Vec<f64> = x.iter().zip(&k3).map(|(xi, ki)| xi + h * ki).collect();
        let k4 = f(t + h, &x4);
        for i in 0..x.len() {
            x[i] += h / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
        t += h;
    }
    IntegratorState {
        time: t,
        variables: x,
    }
}

/// A first-order-plus-dead-time (FOPDT) process model:
/// `τ·dy/dt + y = K·u(t − θ)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fopdt {
    /// Process gain (output per unit input at steady state).
    pub gain: f64,
    /// Time constant, s.
    pub time_constant: f64,
    /// Dead time, s.
    pub dead_time: f64,
}

impl Fopdt {
    /// Simulates the FOPDT response to a piecewise-constant input
    /// `u(t)` sampled at each step, returning `samples` output values
    /// over `duration` seconds. Dead time is handled by input buffering.
    #[must_use]
    pub fn simulate(
        &self,
        u: impl Fn(f64) -> f64,
        initial_output: f64,
        duration: f64,
        samples: usize,
    ) -> Vec<f64> {
        let samples = samples.max(1);
        let dt = duration / samples as f64;
        // History buffer for dead time (rounded to whole steps), pre-filled
        // with the assumed pre-step input (0).
        let delay_steps = (self.dead_time / dt).round() as usize;
        let mut history: Vec<f64> = vec![0.0; delay_steps];
        let mut y = initial_output;
        let mut out = Vec::with_capacity(samples);
        for k in 0..samples {
            let t = f64::from(k as u32) * dt;
            history.push(u(t));
            let u_delayed = history[k];
            // Exact exponential update of the linear first-order ODE.
            let alpha = (-dt / self.time_constant.max(1e-9)).exp();
            y = alpha * y + (1.0 - alpha) * self.gain * u_delayed;
            out.push(y);
        }
        out
    }

    /// The 2% settling time (≈ 4·τ, dead time excluded).
    #[must_use]
    pub const fn settling_time(&self) -> f64 {
        4.0 * self.time_constant
    }
}

/// Step-response metrics of a simulated trajectory.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepResponse {
    /// Final (steady-state) value reached.
    pub final_value: f64,
    /// Time to first reach 63.2% of the total change (≈ τ for FOPDT).
    pub time_constant_estimate: f64,
}

/// Estimates step-response metrics from a uniformly sampled trajectory.
#[must_use]
pub fn step_response_metrics(samples: &[f64], initial_value: f64, dt: f64) -> StepResponse {
    let Some(&final_value) = samples.last() else {
        return StepResponse {
            final_value: initial_value,
            time_constant_estimate: 0.0,
        };
    };
    let change = final_value - initial_value;
    let target = initial_value + 0.632 * change;
    let mut tau = samples.len() as f64 * dt;
    for (i, &v) in samples.iter().enumerate() {
        if (change > 0.0 && v >= target) || (change < 0.0 && v <= target) {
            tau = (i + 1) as f64 * dt;
            break;
        }
    }
    StepResponse {
        final_value,
        time_constant_estimate: tau,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rk4_solves_exponential_decay() {
        let state = IntegratorState {
            time: 0.0,
            variables: vec![1.0],
        };
        let final_state = rk4(state, 1.0, 100, |_, x| vec![-x[0]]);
        assert!((final_state.variables[0] - (-1.0_f64).exp()).abs() < 1e-9);
    }

    #[test]
    fn rk4_integrates_time_dependent_forcing() {
        // dx/dt = t: x(2) = 2 from x(0) = 0.
        let state = IntegratorState {
            time: 0.0,
            variables: vec![0.0],
        };
        let final_state = rk4(state, 2.0, 100, |t, _| vec![t]);
        assert!((final_state.variables[0] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn fopdt_reaches_k_times_input() {
        let model = Fopdt {
            gain: 2.0,
            time_constant: 10.0,
            dead_time: 1.0,
        };
        let out = model.simulate(|_| 5.0, 0.0, 100.0, 500);
        assert!((out[499] - 10.0).abs() < 0.01, "y = {}", out[499]);
    }

    #[test]
    fn dead_time_delays_the_response() {
        let with = Fopdt {
            gain: 1.0,
            time_constant: 5.0,
            dead_time: 5.0,
        };
        let without = Fopdt {
            gain: 1.0,
            time_constant: 5.0,
            dead_time: 0.0,
        };
        let out_with = with.simulate(|_| 1.0, 0.0, 50.0, 100);
        let out_without = without.simulate(|_| 1.0, 0.0, 50.0, 100);
        // At t = 4 s (before the 5 s dead time) the dead-time output is
        // unmoved; the no-dead-time output has already risen.
        assert!(out_with[8] < 0.05);
        assert!(out_without[8] > 0.4);
        // Both are within 1% of the steady-state gain at the horizon.
        assert!((out_with[99] - 1.0).abs() < 0.01);
        assert!((out_without[99] - 1.0).abs() < 0.01);
    }

    #[test]
    fn step_metrics_estimate_the_time_constant() {
        let model = Fopdt {
            gain: 1.0,
            time_constant: 8.0,
            dead_time: 0.0,
        };
        let out = model.simulate(|_| 1.0, 0.0, 40.0, 400);
        let metrics = step_response_metrics(&out, 0.0, 0.1);
        assert!((metrics.time_constant_estimate - 8.0).abs() < 0.5);
        assert!((metrics.final_value - 1.0).abs() < 0.01);
    }
}
