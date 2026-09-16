//! Gas compression: isentropic and polytropic work, discharge temperature.
//!
//! Ideal-gas treatment with real efficiency factors — the standard
//! pre-design method for centrifugal and reciprocating machines.

#![forbid(unsafe_code)]

/// Compression service parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompressorService {
    /// Suction pressure, Pa (absolute).
    pub suction_pressure: f64,
    /// Discharge pressure, Pa (absolute).
    pub discharge_pressure: f64,
    /// Suction temperature, K.
    pub suction_temperature: f64,
    /// Ideal-gas heat-capacity ratio cp/cv.
    pub kappa: f64,
    /// Isentropic (adiabatic) efficiency in (0, 1].
    pub isentropic_efficiency: f64,
    /// Compressibility factor at suction (≈ 1 for ideal gas).
    pub compressibility: f64,
}

/// Results of a compression calculation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompressionResult {
    /// Isentropic head, J/kg: `Z·(k/(k−1))·R_s·T1·(r^((k−1)/k) − 1)`.
    pub head_isentropic: f64,
    /// Actual (shaft) specific work, J/kg, = head/η.
    pub work_actual: f64,
    /// Actual discharge temperature, K.
    pub discharge_temperature: f64,
    /// Pressure ratio.
    pub pressure_ratio: f64,
}

impl CompressorService {
    /// Evaluates the compression service.
    ///
    /// `gas_constant_specific` is the specific gas constant R_s = R/M in
    /// J/(kg·K).
    #[must_use]
    pub fn evaluate(&self, gas_constant_specific: f64) -> CompressionResult {
        let ratio = self.discharge_pressure / self.suction_pressure;
        let k = self.kappa;
        let exponent = (k - 1.0) / k;
        let head = self.compressibility
            * (k / (k - 1.0))
            * gas_constant_specific
            * self.suction_temperature
            * (ratio.powf(exponent) - 1.0);
        let work = head / self.isentropic_efficiency;
        // Actual outlet T from the actual work: T2 = T1 + W/(cp·Z),
        // cp = k/(k−1)·R_s.
        let cp = (k / (k - 1.0)) * gas_constant_specific;
        let t2 = self.suction_temperature + work / (cp * self.compressibility);
        CompressionResult {
            head_isentropic: head,
            work_actual: work,
            discharge_temperature: t2,
            pressure_ratio: ratio,
        }
    }

    /// Shaft power for a mass flow in kg/s, W.
    #[must_use]
    pub fn power(&self, mass_flow: f64, gas_constant_specific: f64) -> f64 {
        self.evaluate(gas_constant_specific).work_actual * mass_flow
    }

    /// Polytropic head with exponent n from the polytropic efficiency
    /// η_p: (n−1)/n = (k−1)/(k·η_p), so n/(n−1) = k·η_p/(k−1).
    /// `polytropic_efficiency` in (0, 1].
    #[must_use]
    pub fn polytropic_head(&self, gas_constant_specific: f64, polytropic_efficiency: f64) -> f64 {
        let ratio = self.discharge_pressure / self.suction_pressure;
        let k = self.kappa;
        let exponent = (k - 1.0) / (k * polytropic_efficiency);
        let n_over_n_minus_1 = (k * polytropic_efficiency) / (k - 1.0);
        self.compressibility
            * n_over_n_minus_1
            * gas_constant_specific
            * self.suction_temperature
            * (ratio.powf(exponent) - 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> CompressorService {
        CompressorService {
            suction_pressure: 101_325.0,
            discharge_pressure: 405_300.0, // ratio 4
            suction_temperature: 300.0,
            kappa: 1.4,
            isentropic_efficiency: 0.75,
            compressibility: 1.0,
        }
    }

    #[test]
    fn isentropic_head_matches_analytic() {
        let rs = 8.314462618 / 0.02897; // air
        let r = service().evaluate(rs);
        let expected = (1.4 / 0.4) * rs * 300.0 * (4.0_f64.powf(0.4 / 1.4) - 1.0);
        assert!((r.head_isentropic - expected).abs() < 1e-6);
        assert!((r.work_actual - expected / 0.75).abs() < 1e-6);
    }

    #[test]
    fn ideal_isentropic_discharge_temperature() {
        // η = 1: T2 = T1·r^((k−1)/k) = 300·4^0.2857 ≈ 445 K.
        let svc = CompressorService {
            isentropic_efficiency: 1.0,
            ..service()
        };
        let rs = 8.314462618 / 0.02897;
        let r = svc.evaluate(rs);
        let expected = 300.0 * 4.0_f64.powf(0.4 / 1.4);
        assert!((r.discharge_temperature - expected).abs() < 1.0);
    }

    #[test]
    fn efficiency_raises_discharge_temperature() {
        let rs = 8.314462618 / 0.02897;
        let ideal = CompressorService {
            isentropic_efficiency: 1.0,
            ..service()
        }
        .evaluate(rs)
        .discharge_temperature;
        let real = service().evaluate(rs).discharge_temperature;
        assert!(real > ideal);
    }

    #[test]
    fn polytropic_head_exceeds_isentropic() {
        let rs = 8.314462618 / 0.02897;
        let svc = service();
        let poly = svc.polytropic_head(rs, 0.72);
        assert!(poly > svc.evaluate(rs).head_isentropic);
    }
}
