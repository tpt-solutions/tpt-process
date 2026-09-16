//! Psychrometrics for moist air at near-atmospheric pressures.
//!
//! Reference conditions: ASHRAE-style correlations, dry air and water
//! vapor as ideal gases. Temperatures in K internally; the API takes and
//! returns SI (Pa, J/kg dry air, kg water/kg dry air).
//!
//! # Example
//!
//! ```
//! use tpt_proc_hvac::Psychrometrics;
//!
//! let ps = Psychrometrics::new(101_325.0);
//! // Saturation pressure of water at 20 °C ≈ 2.34 kPa.
//! let p_sat = ps.saturation_pressure(293.15);
//! assert!((p_sat - 2339.0).abs() < 30.0);
//!
//! // 20 °C, 50% RH → humidity ratio ≈ 0.00726 kg/kg.
//! let w = ps.humidity_ratio_from_rh(293.15, 0.5);
//! assert!((w - 0.00726).abs() < 2e-4);
//!
//! // Moist-air enthalpy ≈ 38.6 kJ/kg dry air.
//! let h = ps.enthalpy(293.15, w);
//! assert!((h - 38_600.0).abs() < 500.0);
//! ```

#![forbid(unsafe_code)]

/// Standard atmospheric pressure, Pa.
pub const STANDARD_ATMOSPHERE: f64 = 101_325.0;
/// Specific heat of dry air, J/(kg·K).
pub const CP_DRY_AIR: f64 = 1006.0;
/// Specific heat of water vapor, J/(kg·K).
pub const CP_VAPOR: f64 = 1860.0;
/// Reference enthalpy of vapor (h_fg at 0 °C), J/kg water.
pub const H_VAPORIZATION: f64 = 2_501_000.0;

/// Psychrometric state computations at a fixed total pressure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Psychrometrics {
    /// Total (barometric) pressure, Pa.
    pub pressure: f64,
}

impl Psychrometrics {
    /// Creates a psychrometric engine at `pressure` Pa.
    #[must_use]
    pub const fn new(pressure: f64) -> Self {
        Self { pressure }
    }

    /// Saturation pressure of pure water, Pa, by the ASHRAE/Magnus-style
    /// correlation (valid 273–373 K within ~0.1%).
    #[must_use]
    pub fn saturation_pressure(&self, temperature: f64) -> f64 {
        let t_c = temperature - 273.15;
        611.21 * ((18.678 - t_c / 234.5) * (t_c / (257.14 + t_c))).exp()
    }

    /// Humidity ratio W from a relative humidity φ ∈ [0, 1] at
    /// `temperature` (K): W = 0.62198·φ·p_sat/(p − φ·p_sat).
    #[must_use]
    pub fn humidity_ratio_from_rh(&self, temperature: f64, relative_humidity: f64) -> f64 {
        let p_sat = self.saturation_pressure(temperature);
        let p_v = relative_humidity.clamp(0.0, 1.0) * p_sat;
        0.621_98 * p_v / (self.pressure - p_v).max(1.0)
    }

    /// Relative humidity from a humidity ratio.
    #[must_use]
    pub fn relative_humidity(&self, temperature: f64, humidity_ratio: f64) -> f64 {
        let p_sat = self.saturation_pressure(temperature);
        let p_v = humidity_ratio * self.pressure / (0.621_98 + humidity_ratio);
        (p_v / p_sat).clamp(0.0, 1.0)
    }

    /// Specific enthalpy of moist air per kg dry air, J/kg:
    /// h = cp_air·T + W·(h_fg + cp_vapor·T) with the 0 K reference.
    #[must_use]
    pub fn enthalpy(&self, temperature: f64, humidity_ratio: f64) -> f64 {
        // Reference at 0 °C: h = 1.006·T[°C] + W·(2501 + 1.86·T[°C]) kJ/kg.
        let t_c = temperature - 273.15;
        1006.0 * t_c + humidity_ratio * (2_501_000.0 + 1860.0 * t_c)
    }

    /// Specific volume of moist air, m³/kg dry air:
    /// v = R_air·T·(1 + 1.6078·W)/p.
    #[must_use]
    pub fn specific_volume(&self, temperature: f64, humidity_ratio: f64) -> f64 {
        287.055 * temperature * (1.0 + 1.607_8 * humidity_ratio) / self.pressure
    }

    /// Dew-point temperature, K, by inverting the saturation correlation
    /// (bisection between 173 K and the dry-bulb temperature).
    #[must_use]
    pub fn dew_point(&self, temperature: f64, humidity_ratio: f64) -> f64 {
        let p_v = humidity_ratio * self.pressure / (0.621_98 + humidity_ratio);
        let (mut lo, mut hi) = (173.15_f64, temperature);
        if p_v <= 0.0 {
            return lo;
        }
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            if self.saturation_pressure(mid) < p_v {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    /// Thermodynamic wet-bulb temperature, K, by the adiabatic saturation
    /// approximation: find T_wb where the enthalpy of air saturated at
    /// T_wb plus the liquid make-up term equals the inlet enthalpy.
    #[must_use]
    pub fn wet_bulb(&self, temperature: f64, humidity_ratio: f64) -> f64 {
        let h_target = self.enthalpy(temperature, humidity_ratio);
        let residual = |t: f64| self.enthalpy(t, self.humidity_ratio_from_rh(t, 1.0)) - h_target;
        let (mut lo, mut hi) = (173.15_f64, temperature);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            if residual(mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    /// Adiabatic mixing of two moist-air streams (per kg dry air basis).
    /// Returns the mixed (dry-bulb, humidity ratio).
    #[must_use]
    pub fn mixing(
        &self,
        dry_bulb_a: f64,
        w_a: f64,
        mda_kg_s: f64,
        dry_bulb_b: f64,
        w_b: f64,
        mdb_kg_s: f64,
    ) -> (f64, f64) {
        let total = mda_kg_s + mdb_kg_s;
        if total <= 0.0 {
            return (dry_bulb_a, w_a);
        }
        let w = (mda_kg_s * w_a + mdb_kg_s * w_b) / total;
        let t = (mda_kg_s * dry_bulb_a + mdb_kg_s * dry_bulb_b) / total;
        (t, w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturation_pressure_reference_points() {
        let ps = Psychrometrics::new(STANDARD_ATMOSPHERE);
        // 0, 20, 100 °C: 0.611, 2.339, 101.3 kPa.
        assert!((ps.saturation_pressure(273.15) - 611.2).abs() < 5.0);
        assert!((ps.saturation_pressure(293.15) - 2339.0).abs() < 30.0);
        assert!((ps.saturation_pressure(373.15) - 101_325.0).abs() < 1500.0);
    }

    #[test]
    fn humidity_ratio_and_rh_inverse() {
        let ps = Psychrometrics::new(STANDARD_ATMOSPHERE);
        let w = ps.humidity_ratio_from_rh(293.15, 0.5);
        assert!((w - 0.00726).abs() < 2e-4, "W = {w}");
        let rh = ps.relative_humidity(293.15, w);
        assert!((rh - 0.5).abs() < 1e-3);
        // Saturated air: RH = 1.
        let w_sat = ps.humidity_ratio_from_rh(293.15, 1.0);
        assert!((ps.relative_humidity(293.15, w_sat) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn enthalpy_reference() {
        let ps = Psychrometrics::new(STANDARD_ATMOSPHERE);
        // 20 °C, W = 0.00726: h ≈ 1.006·20 + 0.00726·(2501 + 1.86·20)
        //                   ≈ 38.6 kJ/kg.
        let h = ps.enthalpy(293.15, 0.00726);
        assert!((h - 38_600.0).abs() < 500.0, "h = {h}");
    }

    #[test]
    fn dew_point_below_dry_bulb() {
        let ps = Psychrometrics::new(STANDARD_ATMOSPHERE);
        let w = ps.humidity_ratio_from_rh(293.15, 0.5);
        let dew = ps.dew_point(293.15, w);
        // 50% RH at 20 °C: dew point ≈ 9.3 °C.
        assert!((dew - 282.4).abs() < 1.5, "dew = {dew}");
        assert!(dew < 293.15);
    }

    #[test]
    fn wet_bulb_between_dew_and_dry() {
        let ps = Psychrometrics::new(STANDARD_ATMOSPHERE);
        let w = ps.humidity_ratio_from_rh(293.15, 0.5);
        let dew = ps.dew_point(293.15, w);
        let twb = ps.wet_bulb(293.15, w);
        assert!(dew < twb && twb < 293.15, "dew {dew}, twb {twb}");
    }

    #[test]
    fn mixing_conserves_mass_and_energy() {
        let ps = Psychrometrics::new(STANDARD_ATMOSPHERE);
        let (t, w) = ps.mixing(303.15, 0.010, 1.0, 288.15, 0.005, 1.0);
        // Equal flows: linear averages.
        assert!((t - 295.65).abs() < 1e-9);
        assert!((w - 0.0075).abs() < 1e-12);
    }

    #[test]
    fn specific_volume_of_dry_air() {
        let ps = Psychrometrics::new(STANDARD_ATMOSPHERE);
        let v = ps.specific_volume(293.15, 0.0);
        assert!((v - 0.8306).abs() < 0.01, "v = {v}");
    }
}
