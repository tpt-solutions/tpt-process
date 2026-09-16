//! Bridge from `tpt-process` pinch/utility results into `tpt-energy`
//! system models (`tpt-nrg-core`).
//!
//! The process side computes minimum heating and cooling duties (pinch
//! targets, W). The energy side plans electrical systems in MW with
//! buses, generators, and loads. This crate translates between them:
//!
//! - **Cooling duty** → electrical chiller load: `Q_cold / (COP × 1000)`
//!   MW of electricity.
//! - **Heating duty** → optionally served by a CHP unit registered as a
//!   `Generator` sized by its electrical output
//!   (`Q_heat × η_elec / 1000` MW); the balance is a thermal-fuel load.
//!
//! # Example
//!
//! ```
//! use tpt_nrg_core::{Bus, BusType, EnergySystem};
//! use tpt_proc_energy::UtilityBridge;
//! use tpt_proc_heat_network::{PinchAnalysis, ProcessStream};
//!
//! let pinch = PinchAnalysis::new(
//!     vec![ProcessStream::hot(433.15, 323.15, 10.0)],
//!     vec![ProcessStream::cold(318.15, 428.15, 10.0)],
//!     10.0,
//! );
//! let targets = pinch.minimum_utilities();
//!
//! let mut system = EnergySystem::new("plant", "Process plant", 100.0, 60.0);
//! system.add_bus(Bus::new(1, "Utility bus", BusType::Pq)).unwrap();
//!
//! let bridge = UtilityBridge::default();
//! let applied = bridge
//!     .apply(&mut system, 1, &targets, 2.0e6) // 2 MW of process power demand
//!     .unwrap();
//! assert!(applied.chiller_load_mw > 0.0);
//! ```

#![forbid(unsafe_code)]

use tpt_nrg_core::{BusType, EnergySystem, Generator, GeneratorType, Load};
use tpt_proc_heat_network::UtilityTargets;

/// Assumptions translating thermal duties into electrical system
/// quantities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UtilityBridge {
    /// Chiller coefficient of performance (thermal removed per electrical
    /// energy), dimensionless. Typical 3.5–5.
    pub chiller_cop: f64,
    /// Electrical efficiency of the CHP unit serving heating duty,
    /// fraction. Typical 0.35–0.45. Set to 0 to disable CHP registration.
    pub chp_electrical_efficiency: f64,
    /// Fraction of the heating duty that can be served by electric
    /// heating when no CHP is registered.
    pub electric_heating_fraction: f64,
}

impl Default for UtilityBridge {
    fn default() -> Self {
        Self {
            chiller_cop: 4.0,
            chp_electrical_efficiency: 0.40,
            electric_heating_fraction: 0.0,
        }
    }
}

/// What the bridge registered on the energy system.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppliedUtilities {
    /// Electrical chiller load, MW (positive = demand).
    pub chiller_load_mw: f64,
    /// Electrical heating load, MW (0 when the CHP covers heating).
    pub heating_load_mw: f64,
    /// CHP electrical capacity registered, MW (0 when disabled).
    pub chp_capacity_mw: f64,
    /// The process power demand passed through unchanged, MW.
    pub process_load_mw: f64,
}

impl UtilityBridge {
    /// Creates a bridge with explicit assumptions.
    #[must_use]
    pub const fn new(
        chiller_cop: f64,
        chp_electrical_efficiency: f64,
        electric_heating_fraction: f64,
    ) -> Self {
        Self {
            chiller_cop,
            chp_electrical_efficiency,
            electric_heating_fraction,
        }
    }

    /// Electrical chiller load for a cooling duty, MW.
    #[must_use]
    pub fn chiller_load_mw(&self, cooling_duty_w: f64) -> f64 {
        if self.chiller_cop <= 0.0 {
            return 0.0;
        }
        cooling_duty_w.max(0.0) / self.chiller_cop / 1.0e6
    }

    /// CHP electrical capacity for a heating duty, MW.
    #[must_use]
    pub fn chp_capacity_mw(&self, heating_duty_w: f64) -> f64 {
        heating_duty_w.max(0.0) * self.chp_electrical_efficiency / 1.0e6
    }

    /// Registers the utility demands implied by pinch `targets` on
    /// `system` at bus `bus_id`, alongside the base `process_load_w`.
    ///
    /// Adds up to three elements: the process electrical load, the chiller
    /// load, and (when [`UtilityBridge::chp_electrical_efficiency`] > 0
    /// and heating duty exists) a CHP generator sized to its electrical
    /// output.
    ///
    /// # Errors
    /// Propagates `tpt_nrg_core::CoreError` from the energy-system
    /// registration calls.
    pub fn apply(
        &self,
        system: &mut EnergySystem,
        bus_id: usize,
        targets: &UtilityTargets,
        process_load_w: f64,
    ) -> Result<AppliedUtilities, tpt_nrg_core::CoreError> {
        let next_id = system.buses.len() + system.loads.len() + system.generators.len() + 1;

        // Base process load.
        let process_mw = process_load_w.max(0.0) / 1.0e6;
        system.add_load(Load::new(
            next_id,
            "process-base",
            bus_id,
            process_mw,
            process_mw * 0.2,
        ))?;

        // Chiller electricity from the cooling duty.
        let chiller_mw = self.chiller_load_mw(targets.min_cooling_duty);
        if chiller_mw > 0.0 {
            system.add_load(Load::new(
                next_id + 1,
                "process-chiller",
                bus_id,
                chiller_mw,
                chiller_mw * 0.3,
            ))?;
        }

        // Heating: CHP generator when enabled, else electric-heating load.
        let (heating_load_mw, chp_mw) =
            if self.chp_electrical_efficiency > 0.0 && targets.min_heating_duty > 0.0 {
                let capacity = self.chp_capacity_mw(targets.min_heating_duty);
                if capacity > 0.0 {
                    system.add_generator(
                        Generator::new(
                            next_id + 2,
                            "process-chp",
                            GeneratorType::Thermal,
                            capacity,
                            capacity * 0.5,
                        )
                        .at_bus(bus_id),
                    )?;
                }
                (0.0, capacity)
            } else {
                let electric =
                    targets.min_heating_duty.max(0.0) * self.electric_heating_fraction / 1.0e6;
                if electric > 0.0 {
                    system.add_load(Load::new(
                        next_id + 2,
                        "process-electric-heating",
                        bus_id,
                        electric,
                        0.0,
                    ))?;
                }
                (electric, 0.0)
            };

        Ok(AppliedUtilities {
            chiller_load_mw: chiller_mw,
            heating_load_mw,
            chp_capacity_mw: chp_mw,
            process_load_mw: process_mw,
        })
    }
}

/// Convenience: reads the bus type needed for a pure-load utility bus.
#[must_use]
pub const fn utility_bus_type() -> BusType {
    BusType::Pq
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_nrg_core::Bus;
    use tpt_proc_heat_network::{PinchAnalysis, ProcessStream};

    fn targets() -> UtilityTargets {
        let pinch = PinchAnalysis::new(
            vec![ProcessStream::hot(433.15, 323.15, 10.0)],
            vec![ProcessStream::cold(318.15, 428.15, 10.0)],
            10.0,
        );
        pinch.minimum_utilities()
    }

    fn system() -> EnergySystem {
        let mut system = EnergySystem::new("plant", "Process plant", 100.0, 60.0);
        system
            .add_bus(Bus::new(1, "Utility bus", BusType::Pq))
            .unwrap();
        system
    }

    #[test]
    fn chiller_load_is_duty_over_cop() {
        let bridge = UtilityBridge::default();
        // 4 MW thermal / COP 4 = 1 MW electric.
        assert!((bridge.chiller_load_mw(4.0e6) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn apply_registers_loads_and_chp() {
        let bridge = UtilityBridge::default();
        let mut system = system();
        let applied = bridge.apply(&mut system, 1, &targets(), 10.0e6).unwrap();

        assert!((applied.process_load_mw - 10.0).abs() < 1e-9);
        assert!(applied.chiller_load_mw > 0.0);
        assert!(applied.chp_capacity_mw > 0.0);
        assert_eq!(system.loads.len(), 2); // base + chiller
        assert_eq!(system.generators.len(), 1); // CHP
    }

    #[test]
    fn without_chp_heating_becomes_electric_load() {
        let bridge = UtilityBridge::new(4.0, 0.0, 1.0);
        let mut system = system();
        let applied = bridge.apply(&mut system, 1, &targets(), 10.0e6).unwrap();
        assert_eq!(applied.chp_capacity_mw, 0.0);
        assert_eq!(system.generators.len(), 0);
        assert_eq!(system.loads.len(), 3); // base + chiller + electric heating
                                           // First law of the bridge: heating electricity = duty/1000.
        assert!((applied.heating_load_mw - targets().min_heating_duty / 1.0e6).abs() < 1e-9);
    }

    #[test]
    fn zero_duties_add_nothing_beyond_the_process_load() {
        let empty = UtilityTargets {
            min_heating_duty: 0.0,
            min_cooling_duty: 0.0,
            pinch_hot_temp: None,
            pinch_cold_temp: None,
        };
        let bridge = UtilityBridge::default();
        let mut system = system();
        let applied = bridge.apply(&mut system, 1, &empty, 5.0e6).unwrap();
        assert_eq!(applied.chiller_load_mw, 0.0);
        assert_eq!(system.loads.len(), 1);
        assert_eq!(system.generators.len(), 0);
    }
}
