//! Fired equipment: fuel combustion stoichiometry and furnace efficiency.
//!
//! Simplified single-fuel models for pre-design furnace duty and fuel
//! consumption estimates.
//!
//! # Example
//!
//! ```
//! use tpt_proc_fired_equipment::{Combustion, Fuel};
//!
//! // Methane fired with 20% excess air.
//! let combustion = Combustion::new(Fuel::Methane, 0.20);
//! let air = combustion.air_flow(1.0e6); // per 1 MW fired
//! assert!(air > 0.0);
//!
//! let furnace = combustion.furnace_efficiency(433.15); // stack at 160 °C
//! assert!(furnace > 0.80 && furnace < 1.0);
//! ```

#![forbid(unsafe_code)]

/// Supported fuels with their stoichiometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fuel {
    /// Methane CH₄.
    Methane,
    /// Propane C₃H₈.
    Propane,
    /// Natural gas proxy (treated as methane).
    NaturalGas,
    /// Hydrogen H₂.
    Hydrogen,
}

impl Fuel {
    /// Higher heating value, J/mol.
    #[must_use]
    pub const fn heating_value_per_mol(&self) -> f64 {
        match self {
            // HHV values, J/mol (standard references).
            Self::Methane | Self::NaturalGas => 890.4e3,
            Self::Propane => 2219.9e3,
            Self::Hydrogen => 285.8e3,
        }
    }

    /// Moles of O₂ per mole of fuel for complete combustion.
    #[must_use]
    pub const fn oxygen_per_mol(&self) -> f64 {
        match self {
            Self::Methane | Self::NaturalGas => 2.0,
            Self::Propane => 5.0,
            Self::Hydrogen => 0.5,
        }
    }

    /// Moles of CO₂ per mole of fuel.
    #[must_use]
    pub const fn carbon_dioxide_per_mol(&self) -> f64 {
        match self {
            Self::Methane | Self::NaturalGas => 1.0,
            Self::Propane => 3.0,
            Self::Hydrogen => 0.0,
        }
    }
}

/// Air/fuel stoichiometry with excess air.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Combustion {
    /// Fuel being fired.
    pub fuel: Fuel,
    /// Excess air fraction (0.20 = 20% above stoichiometric).
    pub excess_air: f64,
}

/// Air is 21% O₂ by volume; the N₂/O₂ ratio by volume.
const N2_TO_O2: f64 = 79.0 / 21.0;

impl Combustion {
    /// Creates a combustion model.
    #[must_use]
    pub fn new(fuel: Fuel, excess_air: f64) -> Self {
        Self {
            fuel,
            excess_air: excess_air.max(0.0),
        }
    }

    /// Moles of air per mole of fuel.
    #[must_use]
    pub fn air_per_mol_fuel(&self) -> f64 {
        self.fuel.oxygen_per_mol() * (1.0 + self.excess_air) * (1.0 + N2_TO_O2)
    }

    /// Combustion air per unit of fired duty, mol/J.
    #[must_use]
    pub fn air_flow(&self, duty_watts: f64) -> f64 {
        duty_watts / self.fuel.heating_value_per_mol() * self.air_per_mol_fuel()
    }

    /// Fuel consumption per fired duty, mol/J.
    #[must_use]
    pub fn fuel_flow(&self, duty_watts: f64) -> f64 {
        duty_watts / self.fuel.heating_value_per_mol()
    }

    /// Dry flue-gas CO₂ per mole of fuel, mol.
    #[must_use]
    pub fn carbon_dioxide_per_mol_fuel(&self) -> f64 {
        self.fuel.carbon_dioxide_per_mol()
    }

    /// Furnace efficiency by the stack-loss (segmental) method:
    /// η ≈ 1 − stack losses / fired duty with the flue gas raised from
    /// ambient to `stack_temperature`. Sensible heat of the flue gas uses
    /// a mean molar cp of 33 J/(mol·K) over air + products.
    ///
    /// `stack_temperature` in K.
    #[must_use]
    pub fn furnace_efficiency(&self, stack_temperature: f64) -> f64 {
        const AMBIENT: f64 = 298.15;
        const CP_FLUE: f64 = 33.0; // J/(mol·K), mean for flue gas
        let total_gas_mol = 1.0 + self.air_per_mol_fuel(); // per mol fuel
        let stack_loss = total_gas_mol * CP_FLUE * (stack_temperature - AMBIENT);
        let efficiency = 1.0 - stack_loss / self.fuel.heating_value_per_mol();
        efficiency.clamp(0.0, 1.0)
    }

    /// Duty delivered to the process for a fired duty (fuel HHV basis):
    /// Q_process = η · Q_fired, W.
    #[must_use]
    pub fn absorbed_duty(&self, fired_duty: f64, stack_temperature: f64) -> f64 {
        self.furnace_efficiency(stack_temperature) * fired_duty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stoichiometric_air_for_methane() {
        // CH₄ + 2O₂ → CO₂ + 2H₂O; air = 2·(1+79/21) = 9.52 mol.
        let stoich = Combustion::new(Fuel::Methane, 0.0);
        assert!((stoich.air_per_mol_fuel() - 9.5238).abs() < 1e-3);
        // 20% excess: 9.5238·1.2 = 11.43.
        let excess = Combustion::new(Fuel::Methane, 0.20);
        assert!((excess.air_per_mol_fuel() - 9.5238 * 1.2).abs() < 1e-3);
    }

    #[test]
    fn hydrogen_burns_with_half_the_oxygen() {
        let c = Combustion::new(Fuel::Hydrogen, 0.0);
        // H₂ + ½O₂ → H₂O: air = 0.5·4.762 = 2.381.
        assert!((c.air_per_mol_fuel() - 0.5 * (1.0 + N2_TO_O2)).abs() < 1e-9);
        assert_eq!(c.carbon_dioxide_per_mol_fuel(), 0.0);
    }

    #[test]
    fn air_flow_scales_linearly_with_duty() {
        let c = Combustion::new(Fuel::NaturalGas, 0.15);
        let a = c.air_flow(1.0e6);
        assert!((c.air_flow(2.0e6) - 2.0 * a).abs() < 1e-9);
    }

    #[test]
    fn efficiency_falls_with_stack_temperature() {
        let c = Combustion::new(Fuel::Methane, 0.20);
        let hot_stack = c.furnace_efficiency(533.15); // 260 °C
        let cool_stack = c.furnace_efficiency(433.15); // 160 °C
        assert!(cool_stack > hot_stack);
        // Typical fired heater at 160 °C stack, 20% excess: η ≈ 0.85–0.95.
        assert!(cool_stack > 0.80 && cool_stack < 0.99, "η = {cool_stack}");
        assert!(hot_stack < cool_stack);
    }

    #[test]
    fn absorbed_duty_is_fraction_of_fired() {
        let c = Combustion::new(Fuel::Methane, 0.20);
        let eta = c.furnace_efficiency(433.15);
        let absorbed = c.absorbed_duty(1.0e6, 433.15);
        assert!((absorbed - eta * 1.0e6).abs() < 1e-6);
    }
}
