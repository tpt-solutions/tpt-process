//! Centrifugal pump modeling: curves, operating point, affinity laws.
//!
//! Heads in m of fluid, flows in m³/s, power in W.

#![forbid(unsafe_code)]

/// Quadratic pump curve H(Q) = h0 + a·Q + b·Q².
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PumpCurve {
    /// Shutoff head at zero flow, m.
    pub shutoff_head: f64,
    /// Linear coefficient, m/(m³/s).
    pub a: f64,
    /// Quadratic coefficient (negative for a drooping-safe curve), m/(m³/s)².
    pub b: f64,
}

impl PumpCurve {
    /// Head at volumetric flow `flow`, m.
    #[must_use]
    pub fn head_at_flow(&self, flow: f64) -> f64 {
        self.shutoff_head + self.a * flow + self.b * flow * flow
    }

    /// Scales the curve by the affinity laws for a speed ratio
    /// `ratio = N2/N1`: H scales with ratio², Q with ratio.
    #[must_use]
    pub fn affinity_scaled(&self, ratio: f64) -> PumpCurve {
        PumpCurve {
            shutoff_head: self.shutoff_head * ratio * ratio,
            a: self.a * ratio,
            b: self.b,
        }
    }
}

/// Quadratic system curve H_sys(Q) = H_static + k·Q².
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SystemCurve {
    /// Static (elevation + pressure) head, m.
    pub static_head: f64,
    /// Friction coefficient k, m/(m³/s)².
    pub k: f64,
}

impl SystemCurve {
    /// Required head at volumetric flow `flow`, m.
    #[must_use]
    pub fn head_at_flow(&self, flow: f64) -> f64 {
        self.static_head + self.k * flow * flow
    }
}

/// A pump at a fixed shaft speed with a hydraulic efficiency model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pump {
    /// Pump curve.
    pub curve: PumpCurve,
    /// Shaft speed, rpm.
    pub speed_rpm: f64,
    /// Reference hydraulic efficiency at the best-efficiency point flow
    /// `bep_flow`, m³/s; efficiency falls quadratically away from it.
    pub bep_flow: f64,
    /// Peak efficiency in (0, 1].
    pub peak_efficiency: f64,
}

impl Pump {
    /// Hydraulic efficiency at a flow (clamped to (0, 1]).
    #[must_use]
    pub fn efficiency_at_flow(&self, flow: f64) -> f64 {
        let ratio = flow / self.bep_flow.max(1e-9);
        let eff = self.peak_efficiency * (1.0 - 0.5 * (ratio - 1.0).powi(2));
        eff.clamp(0.05, 1.0)
    }

    /// Shaft power P = ρ·g·Q·H/η, W, at a volumetric flow with fluid
    /// density `density` kg/m³.
    #[must_use]
    pub fn power_at_flow(&self, flow: f64, density: f64) -> f64 {
        let head = self.curve.head_at_flow(flow);
        head * flow * 9.80665 * density / self.efficiency_at_flow(flow)
    }

    /// Finds the operating point (flow m³/s, head m) where the pump curve
    /// meets the system curve, by bisection on the residual
    /// `H_pump − H_sys` (decreasing in Q for normal curves).
    ///
    /// Returns `None` when the curves never cross below the shutoff flow
    /// bound `q_max` (e.g. pump cannot overcome the static head).
    #[must_use]
    pub fn operating_point(&self, system: &SystemCurve, q_max: f64) -> Option<(f64, f64)> {
        let residual = |q: f64| self.curve.head_at_flow(q) - system.head_at_flow(q);
        // Residual at 0 is shutoff − static; must be positive to have flow.
        if residual(0.0) <= 0.0 {
            return None;
        }
        let mut lo = 0.0_f64;
        let mut hi = q_max;
        if residual(hi) > 0.0 {
            // System never catches up inside the bound: operating point is
            // at the bound.
            return Some((hi, self.curve.head_at_flow(hi)));
        }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            if residual(mid) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let flow = 0.5 * (lo + hi);
        Some((flow, self.curve.head_at_flow(flow)))
    }

    /// NPSH available, m: `P_atm/ρg + z_suction − vap/ρg − h_losses`.
    #[must_use]
    pub fn npsh_available(
        surface_pressure: f64,
        vapor_pressure: f64,
        static_suction_head: f64,
        suction_losses: f64,
        density: f64,
    ) -> f64 {
        (surface_pressure - vapor_pressure) / (9.80665 * density) + static_suction_head
            - suction_losses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pump() -> Pump {
        Pump {
            curve: PumpCurve {
                shutoff_head: 30.0,
                a: 0.0,
                b: -8000.0, // H = 30 − 8000·Q²
            },
            speed_rpm: 1750.0,
            bep_flow: 0.03,
            peak_efficiency: 0.72,
        }
    }

    #[test]
    fn operating_point_is_the_analytic_crossing() {
        let system = SystemCurve {
            static_head: 10.0,
            k: 8000.0,
        };
        // 30 − 8000Q² = 10 + 8000Q² → Q = √(20/16000) = 0.035355...
        let (q, h) = pump().operating_point(&system, 0.2).unwrap();
        assert!((q - (20.0_f64 / 16000.0).sqrt()).abs() < 1e-10);
        assert!((h - 20.0).abs() < 1e-6);
    }

    #[test]
    fn pump_below_static_head_gives_no_flow() {
        let system = SystemCurve {
            static_head: 50.0,
            k: 100.0,
        };
        assert!(pump().operating_point(&system, 0.2).is_none());
    }

    #[test]
    fn affinity_laws_scale_the_operating_point() {
        // At half speed: Q halves, H quarters for a purely frictional
        // system through the same scaled curve.
        let half = pump();
        let scaled_curve = half.curve.affinity_scaled(0.5);
        assert!((scaled_curve.shutoff_head - 7.5).abs() < 1e-12);
        let system = SystemCurve {
            static_head: 0.0,
            k: 8000.0,
        };
        let full_system = SystemCurve {
            static_head: 0.0,
            k: 8000.0,
        };
        // Full speed against pure friction: H = 30 − 8000Q² = 8000Q² →
        // Q = 0.0433, H = 15.
        let (q1, h1) = half.operating_point(&full_system, 1.0).unwrap();
        // Half speed: H = 7.5 − 8000Q² = 8000Q² → Q = 0.02165, H = 3.75.
        let pump_half = Pump {
            curve: scaled_curve,
            ..half
        };
        let (q2, h2) = pump_half.operating_point(&system, 1.0).unwrap();
        assert!((q2 / q1 - 0.5).abs() < 1e-9);
        assert!((h2 / h1 - 0.25).abs() < 1e-9);
    }

    #[test]
    fn power_is_positive_and_efficiency_bounded() {
        let p = pump();
        let power = p.power_at_flow(0.03, 1000.0);
        // H(0.03) = 30 − 7.2 = 22.8 m; P = 22.8·0.03·9810/0.72 ≈ 9.3 kW.
        assert!((power - 9339.0).abs() < 50.0, "P = {power}");
        assert!(p.efficiency_at_flow(1e9) >= 0.05);
        assert!(p.efficiency_at_flow(p.bep_flow) <= 1.0);
    }

    #[test]
    fn npsh_subtracts_losses() {
        let npsh = Pump::npsh_available(101_325.0, 2340.0, 2.0, 0.5, 998.0);
        // (101325−2340)/(9.807·998) + 2 − 0.5 ≈ 10.15 + 1.5
        assert!((npsh - 11.65).abs() < 0.05, "NPSH = {npsh}");
    }
}
