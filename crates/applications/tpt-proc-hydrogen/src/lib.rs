//! Hydrogen production models: water electrolysis, steam methane
//! reforming (SMR), autothermal reforming (ATR), and water-gas shift.
//!
//! Pre-design energy accounting only — not a detailed reactor model.
//!
//! # Example
//!
//! ```
//! use tpt_proc_hydrogen::{Electrolyzer, ElectrolyzerType};
//!
//! let pem = Electrolyzer::new(ElectrolyzerType::Pem);
//! let sec = pem.specific_energy_consumption(); // kWh/kg H2
//! assert!((47.0..=60.0).contains(&sec));
//!
//! // A 100 MW plant produces roughly 2 t/h.
//! let rate = pem.production_rate(100.0e6); // kg/s
//! assert!(rate > 0.5 && rate < 0.7);
//! ```

#![forbid(unsafe_code)]

/// Electrolyzer technology.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElectrolyzerType {
    /// Alkaline water electrolysis (mature, ~50–55 kWh/kg system level).
    Alkaline,
    /// Proton exchange membrane (~52–58 kWh/kg).
    Pem,
    /// Solid oxide high-temperature (~37–45 kWh/kg with heat input).
    SolidOxide,
}

/// Lower heating value of hydrogen, MJ/kg (energy content basis).
pub const HYDROGEN_LHV_MJ_KG: f64 = 120.0;
/// Higher heating value of hydrogen, MJ/kg.
pub const HYDROGEN_HHV_MJ_KG: f64 = 141.8;

/// An electrolysis plant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Electrolyzer {
    /// Technology.
    pub electrolyzer_type: ElectrolyzerType,
    /// Stack efficiency fraction of the thermodynamic ideal (typical
    /// 0.6–0.75 system LHV basis).
    pub stack_efficiency: f64,
}

impl Electrolyzer {
    /// Creates an electrolyzer with the technology's typical efficiency.
    #[must_use]
    pub fn new(electrolyzer_type: ElectrolyzerType) -> Self {
        let stack_efficiency = match electrolyzer_type {
            ElectrolyzerType::Alkaline => 0.67,
            ElectrolyzerType::Pem => 0.65,
            ElectrolyzerType::SolidOxide => 0.90,
        };
        Self {
            electrolyzer_type,
            stack_efficiency,
        }
    }

    /// Specific energy consumption, kWh/kg H₂:
    /// SEC = LHV(H₂)/η with LHV = 120 MJ/kg and 1 kWh = 3.6 MJ.
    #[must_use]
    pub fn specific_energy_consumption(&self) -> f64 {
        HYDROGEN_LHV_MJ_KG / self.stack_efficiency / 3.6
    }

    /// Hydrogen production rate for an input power, kg/s.
    #[must_use]
    pub fn production_rate(&self, power_watts: f64) -> f64 {
        let sec_kwh_kg = self.specific_energy_consumption();
        power_watts / 1000.0 / sec_kwh_kg / 3600.0
    }

    /// Water feed requirement, kg water per kg H₂ (stoichiometry 8.94,
    /// with a purification margin).
    #[must_use]
    pub fn water_consumption(&self, kg_per_kg_margin: f64) -> f64 {
        8.936 * (1.0 + kg_per_kg_margin.max(0.0))
    }
}

/// Steam methane reforming energy model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Smr {
    /// Reformer thermal efficiency (fuel → H₂ LHV), typically 0.74–0.86.
    pub thermal_efficiency: f64,
}

/// An autothermal reformer (adds oxygen; near thermoneutral).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atr {
    /// Overall efficiency, typically 0.78–0.88.
    pub overall_efficiency: f64,
}

impl Smr {
    /// Natural gas energy input per kg H₂, MJ (LHV basis).
    #[must_use]
    pub fn fuel_per_kg_h2(&self) -> f64 {
        HYDROGEN_LHV_MJ_KG / self.thermal_efficiency
    }

    /// CO₂ emissions, kg CO₂ per kg H₂ (unabated). Stoichiometric
    /// process CO₂ is 5.5 kg/kg H₂ (CH₄ + 2H₂O → CO₂ + 4H₂); the fired
    /// fuel is the total input minus the ~100 MJ/kg feedstock share, at
    /// 0.055 kg CO₂/MJ of natural gas.
    #[must_use]
    pub fn co2_per_kg_h2(&self) -> f64 {
        let total_input = HYDROGEN_LHV_MJ_KG / self.thermal_efficiency;
        let fired = (total_input - 100.0).max(0.0);
        5.5 + fired * 0.055
    }

    /// Hydrogen rate for a natural gas feed energy rate, kg/s.
    #[must_use]
    pub fn production_rate(&self, fuel_energy_watts: f64) -> f64 {
        fuel_energy_watts / 1.0e6 * self.thermal_efficiency / HYDROGEN_LHV_MJ_KG
    }
}

impl Atr {
    /// Oxygen consumption per kg H₂, kg O₂ (partial oxidation share).
    #[must_use]
    pub fn oxygen_per_kg_h2(&self) -> f64 {
        0.35 / self.overall_efficiency
    }

    /// Hydrogen rate for a natural gas feed energy rate, kg/s.
    #[must_use]
    pub fn production_rate(&self, fuel_energy_watts: f64) -> f64 {
        fuel_energy_watts / 1.0e6 * self.overall_efficiency / HYDROGEN_LHV_MJ_KG
    }
}

/// Water-gas shift equilibrium approach: fraction of CO converted per
/// stage at the given temperature (high-temperature ~90%, low-temperature
/// ~97% at typical conditions — modeled as a simple approach factor).
#[must_use]
pub fn water_gas_shift_conversion(temperature_k: f64) -> f64 {
    // Empirical: conversion falls with temperature above ~500 K.
    (1.0 - (temperature_k - 473.15) / 800.0).clamp(0.55, 0.97)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn electrolysis_sec_in_expected_range() {
        let pem = Electrolyzer::new(ElectrolyzerType::Pem);
        let sec = pem.specific_energy_consumption();
        assert!((47.0..=60.0).contains(&sec), "SEC = {sec}");
        let soec = Electrolyzer::new(ElectrolyzerType::SolidOxide);
        assert!(soec.specific_energy_consumption() < sec);
    }

    #[test]
    fn production_rate_scales_with_power() {
        let pem = Electrolyzer::new(ElectrolyzerType::Pem);
        let r100 = pem.production_rate(100.0e6);
        assert!((pem.production_rate(200.0e6) - 2.0 * r100).abs() < 1e-12);
        // 100 MW at ~55 kWh/kg → ~1.8 t/h ≈ 0.5 kg/s.
        assert!(r100 > 0.4 && r100 < 0.7, "rate = {r100}");
    }

    #[test]
    fn water_consumption_near_stoichiometry() {
        let pem = Electrolyzer::new(ElectrolyzerType::Pem);
        let w = pem.water_consumption(0.1);
        assert!(w > 8.936 && w < 10.0);
    }

    #[test]
    fn smr_emissions_in_industrial_range() {
        let smr = Smr {
            thermal_efficiency: 0.80,
        };
        let co2 = smr.co2_per_kg_h2();
        assert!(co2 > 7.5 && co2 < 11.0, "CO₂ = {co2}");
        // Better efficiency → less CO₂ per kg H₂.
        let better = Smr {
            thermal_efficiency: 0.86,
        };
        assert!(better.co2_per_kg_h2() < co2);
    }

    #[test]
    fn atr_uses_oxygen_and_outperforms_smr() {
        let atr = Atr {
            overall_efficiency: 0.85,
        };
        let smr = Smr {
            thermal_efficiency: 0.80,
        };
        assert!(atr.production_rate(1.0e9) > smr.production_rate(1.0e9));
        assert!(atr.oxygen_per_kg_h2() > 0.3);
    }

    #[test]
    fn wgs_conversion_falls_with_temperature() {
        assert!(water_gas_shift_conversion(473.15) > water_gas_shift_conversion(673.15));
        assert!((0.55..=0.97).contains(&water_gas_shift_conversion(673.15)));
    }
}
