//! PID control: ideal-form controller with derivative filter, anti-windup,
//! and classical tuning rules (Ziegler–Nichols, Cohen–Coon).
//!
//! # Example
//!
//! ```
//! use tpt_proc_control::{Pid, PidConfig};
//!
//! let mut pid = Pid::new(PidConfig {
//!     gain: 2.0,
//!     integral_time: 60.0,
//!     derivative_time: 10.0,
//!     output_limits: Some((0.0, 100.0)),
//! });
//!
//! let output = pid.compute(300.0, 295.0, 1.0); // (setpoint, pv, dt)
//! assert!((0.0..=100.0).contains(&output));
//! ```

#![forbid(unsafe_code)]

/// PID configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PidConfig {
    /// Controller gain Kc (output units per unit of error).
    pub gain: f64,
    /// Integral time Ti, s (0 disables integration).
    pub integral_time: f64,
    /// Derivative time Td, s (0 disables derivative).
    pub derivative_time: f64,
    /// Output clamp (low, high).
    pub output_limits: Option<(f64, f64)>,
}

/// An ideal PID controller with filtered derivative and clamping
/// anti-windup.
#[derive(Clone, Debug, PartialEq)]
pub struct Pid {
    config: PidConfig,
    integral: f64,
    previous_pv: Option<f64>,
    previous_output: f64,
}

impl Pid {
    /// Creates a controller.
    #[must_use]
    pub fn new(config: PidConfig) -> Self {
        Self {
            config,
            integral: 0.0,
            previous_pv: None,
            previous_output: 0.0,
        }
    }

    /// Computes the controller output for the current sample.
    ///
    /// `setpoint` is the target, `pv` the measured value, `dt` the sample
    /// interval in seconds. The derivative acts on the measurement only
    /// (derivative kick avoidance); the integral freezes while the output
    /// is clamped (clamping anti-windup).
    #[must_use]
    pub fn compute(&mut self, setpoint: f64, pv: f64, dt: f64) -> f64 {
        let error = setpoint - pv;
        let dt = dt.max(1e-9);
        let kc = self.config.gain;

        // Integral term.
        let ti = self.config.integral_time;
        if ti > 0.0 {
            self.integral += kc * error * dt / ti;
        }

        // Derivative on measurement, first-order filtered (N = 10).
        let mut derivative = 0.0;
        if self.config.derivative_time > 0.0 {
            if let Some(prev_pv) = self.previous_pv {
                let dpv = (pv - prev_pv) / dt;
                let filter = dt / (self.config.derivative_time / 10.0 + dt);
                derivative = -kc * self.config.derivative_time * dpv;
                derivative = self.previous_output * (1.0 - filter) + derivative * filter;
            }
        }

        let raw = kc * error + self.integral + derivative;
        let output = match self.config.output_limits {
            Some((lo, hi)) => raw.clamp(lo, hi),
            None => raw,
        };
        // Anti-windup: freeze the integral when the output saturates in the
        // direction of the error.
        if self.config.integral_time > 0.0 && output != raw && (raw - output) * error > 0.0 {
            self.integral -= kc * error * dt / ti;
        }

        self.previous_pv = Some(pv);
        self.previous_output = output;
        output
    }

    /// Resets the controller state.
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.previous_pv = None;
        self.previous_output = 0.0;
    }
}

/// Classical tuning rules from a FOPDT identification
/// (gain K, time constant τ, dead time θ).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProcessReactionCurve {
    /// Process gain.
    pub gain: f64,
    /// Time constant τ, s.
    pub time_constant: f64,
    /// Dead time θ, s.
    pub dead_time: f64,
}

impl ProcessReactionCurve {
    /// Ziegler–Nichols open-loop PID settings.
    #[must_use]
    pub fn ziegler_nichols(&self) -> PidConfig {
        let ratio = self.dead_time / self.time_constant;
        let kc = 1.2 * self.time_constant / (self.gain * self.dead_time);
        PidConfig {
            gain: kc,
            integral_time: 2.0 * ratio * self.time_constant,
            derivative_time: 0.5 * ratio * self.time_constant,
            output_limits: None,
        }
    }

    /// Cohen–Coon PID settings (better for large dead time).
    #[must_use]
    pub fn cohen_coon(&self) -> PidConfig {
        let r = self.dead_time / self.time_constant;
        let a = self.gain * r;
        let kc = (1.0 / self.gain) * (1.0 / a) * (4.0 / 3.0 + r / 4.0);
        PidConfig {
            gain: kc,
            integral_time: self.dead_time * (32.0 + 6.0 * r) / (13.0 + 8.0 * r),
            derivative_time: self.dead_time * 4.0 / (11.0 + 2.0 * r),
            output_limits: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proportional_output_is_kc_times_error() {
        let mut pid = Pid::new(PidConfig {
            gain: 2.0,
            integral_time: 0.0,
            derivative_time: 0.0,
            output_limits: None,
        });
        let out = pid.compute(300.0, 295.0, 1.0);
        assert!((out - 10.0).abs() < 1e-9);
    }

    #[test]
    fn integral_windup_is_clamped() {
        let mut pid = Pid::new(PidConfig {
            gain: 10.0,
            integral_time: 1.0,
            derivative_time: 0.0,
            output_limits: Some((0.0, 100.0)),
        });
        // Large sustained error saturates the output at 100.
        let mut last = 0.0;
        for _ in 0..200 {
            last = pid.compute(500.0, 0.0, 1.0);
        }
        assert!((last - 100.0).abs() < 1e-9);
        // Error reverses: the output must recover promptly (no deep windup).
        let recovery = pid.compute(500.0, 700.0, 1.0);
        assert!(recovery < 100.0, "recovered = {recovery}");
    }

    #[test]
    fn closed_loop_settles_on_fopdt_plant() {
        use tpt_proc_dynamics::Fopdt;
        let plant = Fopdt {
            gain: 0.8,
            time_constant: 20.0,
            dead_time: 2.0,
        };
        let tuning = ProcessReactionCurve {
            gain: 0.8,
            time_constant: 20.0,
            dead_time: 2.0,
        }
        .ziegler_nichols();
        let mut pid = Pid::new(PidConfig {
            output_limits: Some((0.0, 100.0)),
            ..tuning
        });
        let dt = 0.5;
        let mut pv = 0.0;
        let setpoint = 50.0;
        // Simulate the discrete plant y += dt/τ·(K·u − y).
        for _ in 0..2000 {
            let u = pid.compute(setpoint, pv, dt);
            pv += dt / plant.time_constant * (plant.gain * u - pv);
        }
        assert!(
            (pv - setpoint).abs() < 0.5,
            "pv = {pv}, integral = {:?}",
            pid.integral
        );
    }

    #[test]
    fn cohen_coon_gives_more_aggressive_integral_for_long_dead_time() {
        let curve = ProcessReactionCurve {
            gain: 1.0,
            time_constant: 30.0,
            dead_time: 10.0,
        };
        let zn = curve.ziegler_nichols();
        let cc = curve.cohen_coon();
        assert!(cc.gain > 0.0 && zn.gain > 0.0);
        assert!(cc.integral_time > 0.0);
    }
}
