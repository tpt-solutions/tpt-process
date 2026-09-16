//! Control-valve sizing: Cv/Kv conversions, liquid and gas sizing,
//! inherent characteristics.

#![forbid(unsafe_code)]

/// Inherent flow characteristic of a valve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValveCharacteristic {
    /// Cv ∝ opening.
    Linear,
    /// Cv = Cv_max·R^(x−1) with rangeability R = 50 (equal percentage).
    EqualPercentage,
    /// Quick-opening: high gain near closed.
    QuickOpening,
}

/// A control valve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlValve {
    /// Full-open flow coefficient Cv (US: gpm water at 1 psi ΔP).
    pub cv_max: f64,
    /// Inherent characteristic.
    pub characteristic: ValveCharacteristic,
}

impl ControlValve {
    /// Creates a valve.
    #[must_use]
    pub const fn new(cv_max: f64, characteristic: ValveCharacteristic) -> Self {
        Self {
            cv_max,
            characteristic,
        }
    }

    /// Effective Cv at an opening x ∈ [0, 1].
    #[must_use]
    pub fn cv_at(&self, opening: f64) -> f64 {
        let x = opening.clamp(0.0, 1.0);
        match self.characteristic {
            ValveCharacteristic::Linear => self.cv_max * x,
            ValveCharacteristic::EqualPercentage => self.cv_max * 50.0_f64.powf(x - 1.0),
            ValveCharacteristic::QuickOpening => {
                // Common plug characterization: Cv/Cv_max = 1 − (1−x)².
                self.cv_max * (1.0 - (1.0 - x) * (1.0 - x))
            }
        }
    }

    /// Liquid flow through the valve, m³/s, from the ISA simplified
    /// equation with a choking (recovery) factor:
    /// `Q = Cv·√(ΔP_psi/(SG·(1 − F_ch)))` in gpm, converted to m³/s.
    /// `delta_p` is in Pa (converted to psi internally).
    ///
    /// `f_choking` ∈ [0, 1): 0 for ordinary service; near cavitation the
    /// effective driving ΔP is reduced.
    #[must_use]
    pub fn liquid_flow_m3s(
        &self,
        opening: f64,
        delta_p: f64,
        specific_gravity: f64,
        f_choking: f64,
    ) -> f64 {
        if delta_p <= 0.0 || specific_gravity <= 0.0 {
            return 0.0;
        }
        let f = f_choking.clamp(0.0, 0.99);
        let dp_psi = delta_p / 6894.76;
        let q_gpm = self.cv_at(opening) * (dp_psi / (specific_gravity * (1.0 - f))).sqrt();
        q_gpm * 6.30902e-5 // gpm → m³/s
    }

    /// Cv required for a liquid flow (inverse of
    /// [`ControlValve::liquid_flow_m3s`] with no choking). `delta_p` in Pa.
    #[must_use]
    pub fn required_cv_liquid(flow_m3s: f64, delta_p: f64, specific_gravity: f64) -> f64 {
        if delta_p <= 0.0 || specific_gravity <= 0.0 {
            return f64::INFINITY;
        }
        let q_gpm = flow_m3s / 6.30902e-5;
        let dp_psi = delta_p / 6894.76;
        q_gpm * (specific_gravity / dp_psi).sqrt()
    }

    /// Subcritical gas flow, kg/s, from the classic Cv equation:
    /// `Q_scfh = 1360·Cv·√((ΔP·P2)/(SG·T))` (P in psia, T in °R), converted
    /// to kg/s with standard-density SG.
    #[must_use]
    pub fn gas_flow_kgs(
        &self,
        opening: f64,
        p1_pa: f64,
        p2_pa: f64,
        specific_gravity: f64,
        temperature_k: f64,
    ) -> f64 {
        let (p1, p2) = (p1_pa / 6894.76, p2_pa / 6894.76); // psia
        if p1 <= p2 || specific_gravity <= 0.0 || temperature_k <= 0.0 {
            return 0.0;
        }
        let dp = p1 - p2;
        let q_scfh = 1360.0
            * self.cv_at(opening)
            * (dp * p2 / (specific_gravity * (temperature_k * 1.8))).sqrt();
        // scfh (60 °F, 1 atm) → kg/s via air-standard density 1.206 kg/m³
        // times SG.
        let mass_kgh = q_scfh / 35.3147 * 1.206 * specific_gravity;
        mass_kgh / 3600.0
    }
}

/// Kv ↔ Cv conversions (Kv: m³/h water at 1 bar ΔP).
#[must_use]
pub fn cv_to_kv(cv: f64) -> f64 {
    cv * 0.865
}

/// See [`cv_to_kv`].
#[must_use]
pub fn kv_to_cv(kv: f64) -> f64 {
    kv / 0.865
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cv_definition_roundtrip() {
        // Cv is the gpm of water at 1 psi drop: 10 Cv → 10 gpm →
        // 10·6.30902e-5 m³/s.
        let valve = ControlValve::new(10.0, ValveCharacteristic::Linear);
        let q = valve.liquid_flow_m3s(1.0, 1.0 * 6894.76, 1.0, 0.0);
        assert!((q - 10.0 * 6.30902e-5).abs() < 1e-10);
        // Required Cv inverts it.
        let cv = ControlValve::required_cv_liquid(10.0 * 6.30902e-5, 6894.76, 1.0);
        assert!((cv - 10.0).abs() < 1e-9);
    }

    #[test]
    fn linear_characteristic_is_proportional() {
        let valve = ControlValve::new(100.0, ValveCharacteristic::Linear);
        assert!((valve.cv_at(0.5) - 50.0).abs() < 1e-12);
        assert_eq!(valve.cv_at(0.0), 0.0);
        assert_eq!(valve.cv_at(1.0), 100.0);
    }

    #[test]
    fn equal_percentage_gain_grows_with_opening() {
        let valve = ControlValve::new(100.0, ValveCharacteristic::EqualPercentage);
        let low = valve.cv_at(0.2 + 0.05) - valve.cv_at(0.2);
        let high = valve.cv_at(0.8 + 0.05) - valve.cv_at(0.8);
        assert!(high > low);
        assert!((valve.cv_at(1.0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn gas_flow_grows_with_dp_then_respects_subcritical() {
        let valve = ControlValve::new(50.0, ValveCharacteristic::Linear);
        let small = valve.gas_flow_kgs(1.0, 200e3, 190e3, 0.7, 300.0);
        let large = valve.gas_flow_kgs(1.0, 200e3, 150e3, 0.7, 300.0);
        assert!(small > 0.0 && large > small);
        // Reverse ΔP gives no flow.
        assert_eq!(valve.gas_flow_kgs(1.0, 100e3, 150e3, 0.7, 300.0), 0.0);
    }

    #[test]
    fn kv_conversion_factor() {
        assert!((cv_to_kv(100.0) - 86.5).abs() < 1e-9);
        assert!((kv_to_cv(86.5) - 100.0).abs() < 1e-9);
    }
}
