//! Unit-operation specifications.
//!
//! [`UnitOperation`] is the *data* description of a unit as it appears in a
//! flowsheet: which kind of equipment it is and its design parameters. The
//! *behavior* (how inlets map to outlets) lives in `tpt-proc-units` and the
//! physics crates; this split lets flowsheets be built, serialized, and
//! analyzed before any solver is attached.

use crate::ids::UnitId;

/// Configuration of a shell-and-tube or generic heat exchanger.
#[derive(Clone, Debug, PartialEq)]
pub struct HeatExchangerConfig {
    /// Heat-transfer area, m².
    pub area: f64,
    /// Overall heat-transfer coefficient, W/(m²·K).
    pub overall_u: f64,
    /// Fouling factor, m²·K/W (applied to the hot side by convention).
    pub fouling: f64,
    /// Duty specification: `Some(duty)` in W fixes the duty (heater mode);
    /// `None` lets the rating calculation determine it.
    pub duty: Option<f64>,
}

/// Configuration of a centrifugal or positive-displacement pump.
#[derive(Clone, Debug, PartialEq)]
pub struct PumpConfig {
    /// Pump curve head coefficients `H(Q) = a + b·Q + c·Q²` with H in m and
    /// Q in m³/s.
    pub curve: [f64; 3],
    /// Shaft speed, rpm.
    pub speed_rpm: f64,
    /// Reference isentropic/hydraulic efficiency in (0, 1].
    pub efficiency: f64,
    /// Optional fixed pressure boost, Pa (overrides the curve when set).
    pub pressure_rise: Option<f64>,
}

/// Configuration of a compressor (fan/blower/compressor).
#[derive(Clone, Debug, PartialEq)]
pub struct CompressorConfig {
    /// Adiabatic (isentropic) efficiency in (0, 1].
    pub isentropic_efficiency: f64,
    /// Discharge pressure, Pa (fixed-pressure mode).
    pub discharge_pressure: Option<f64>,
    /// Fixed pressure ratio (overrides discharge pressure when set).
    pub pressure_ratio: Option<f64>,
}

/// Configuration of a control or isolation valve.
#[derive(Clone, Debug, PartialEq)]
pub struct ValveConfig {
    /// Flow coefficient Cv (US units, gpm water at 1 psi ΔP).
    pub cv: f64,
    /// Valve opening in [0, 1].
    pub opening: f64,
    /// Inherent characteristic.
    pub characteristic: ValveCharacteristic,
}

/// Inherent flow characteristic of a valve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValveCharacteristic {
    /// Cv proportional to opening.
    Linear,
    /// Cv = Cv_max · exp(x·(R−1)) with rangeability R = 50.
    EqualPercentage,
    /// Quick-opening: high gain near closed.
    QuickOpening,
}

/// Configuration of a chemical reactor.
#[derive(Clone, Debug, PartialEq)]
pub struct ReactorConfig {
    /// Reactor mode.
    pub mode: ReactorMode,
    /// Reacting volume, m³.
    pub volume: f64,
    /// Operating mode for the energy balance.
    pub thermal: ReactorThermal,
    /// Heat-transfer coefficient × area, W/K (for `ReactorThermal::IsothermalWithUa`).
    pub ua: f64,
}

/// Reactor topology.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactorMode {
    /// Stirred tank, steady state.
    Cstr,
    /// Plug-flow tubular reactor.
    Pfr,
    /// Filled with catalyst; Ergun pressure drop applies.
    PackedBed,
    /// Fluidized catalyst bed.
    FluidizedBed,
}

/// Reactor energy mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactorThermal {
    /// No heat exchange with surroundings.
    Adiabatic,
    /// Temperature clamped to the value of the operating temperature.
    Isothermal,
    /// Heat exchange with a utility at fixed temperature via the given UA.
    IsothermalWithUa,
}

/// Configuration of a distillation or absorption column.
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnConfig {
    /// Number of equilibrium stages (including reboiler/condenser as
    /// configured).
    pub num_stages: u32,
    /// 1-based feed stage.
    pub feed_stage: u32,
    /// Reflux ratio L/D; `None` for absorption columns.
    pub reflux_ratio: Option<f64>,
    /// Column top pressure, Pa.
    pub pressure: f64,
}

/// Configuration of a gas absorber / stripper.
#[derive(Clone, Debug, PartialEq)]
pub struct AbsorberConfig {
    /// Number of theoretical stages.
    pub num_stages: u32,
    /// Solvent-to-gas molar flow ratio (L/G).
    pub lg_ratio: f64,
    /// Column top pressure, Pa.
    pub pressure: f64,
}

/// Configuration of a crystallizer.
#[derive(Clone, Debug, PartialEq)]
pub struct CrystallizerConfig {
    /// Operating temperature, K.
    pub temperature: f64,
    /// Residence time, s.
    pub residence_time: f64,
    /// Seed crystal loading, kg crystals per kg solvent.
    pub seeding: f64,
}

/// A unit operation as specified in a flowsheet.
///
/// The enum variants carry only design data; behavior is provided by the
/// `tpt-proc-units` trait implementations and the physics crates.
#[derive(Clone, Debug, PartialEq)]
pub enum UnitOperation {
    /// Adiabatic merger of two or more streams.
    Mixer {
        /// Unit identifier.
        id: UnitId,
        /// Number of inlet ports (≥ 2).
        inlets: usize,
    },
    /// Flow divider with fixed or spec-driven split ratios.
    Splitter {
        /// Unit identifier.
        id: UnitId,
        /// Fraction of the feed routed to each outlet; normalized.
        split_ratios: Vec<f64>,
    },
    /// Heat exchanger, heater, or cooler.
    HeatExchanger {
        /// Unit identifier.
        id: UnitId,
        /// Design parameters.
        config: HeatExchangerConfig,
    },
    /// Liquid pump.
    Pump {
        /// Unit identifier.
        id: UnitId,
        /// Design parameters.
        config: PumpConfig,
    },
    /// Gas compressor.
    Compressor {
        /// Unit identifier.
        id: UnitId,
        /// Design parameters.
        config: CompressorConfig,
    },
    /// Valve or let-down.
    Valve {
        /// Unit identifier.
        id: UnitId,
        /// Design parameters.
        config: ValveConfig,
    },
    /// Chemical reactor.
    Reactor {
        /// Unit identifier.
        id: UnitId,
        /// Design parameters.
        config: ReactorConfig,
    },
    /// Distillation column.
    DistillationColumn {
        /// Unit identifier.
        id: UnitId,
        /// Design parameters.
        config: ColumnConfig,
    },
    /// Vapor-liquid separator operating as a flash drum at fixed T, P.
    Flash {
        /// Unit identifier.
        id: UnitId,
    },
    /// Gas absorber.
    Absorber {
        /// Unit identifier.
        id: UnitId,
        /// Design parameters.
        config: AbsorberConfig,
    },
    /// Crystallizer.
    Crystallizer {
        /// Unit identifier.
        id: UnitId,
        /// Design parameters.
        config: CrystallizerConfig,
    },
}

impl UnitOperation {
    /// The unit's identifier.
    #[must_use]
    pub const fn id(&self) -> UnitId {
        match self {
            Self::Mixer { id, .. }
            | Self::Splitter { id, .. }
            | Self::HeatExchanger { id, .. }
            | Self::Pump { id, .. }
            | Self::Compressor { id, .. }
            | Self::Valve { id, .. }
            | Self::Reactor { id, .. }
            | Self::DistillationColumn { id, .. }
            | Self::Flash { id }
            | Self::Absorber { id, .. }
            | Self::Crystallizer { id, .. } => *id,
        }
    }

    /// Number of inlet ports the unit exposes.
    #[must_use]
    pub fn num_inlets(&self) -> usize {
        match self {
            Self::Mixer { inlets, .. } => *inlets,
            Self::Absorber { .. } | Self::DistillationColumn { .. } => 2,
            Self::Flash { .. } | Self::Crystallizer { .. } => 1,
            _ => 1,
        }
    }

    /// Number of outlet ports the unit exposes.
    #[must_use]
    pub fn num_outlets(&self) -> usize {
        match self {
            Self::Splitter { split_ratios, .. } => split_ratios.len(),
            Self::Flash { .. } | Self::DistillationColumn { .. } => 2,
            Self::Absorber { .. } => 2,
            _ => 1,
        }
    }

    /// Human-readable kind name (for diagnostics and PFD export).
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Mixer { .. } => "mixer",
            Self::Splitter { .. } => "splitter",
            Self::HeatExchanger { .. } => "heat-exchanger",
            Self::Pump { .. } => "pump",
            Self::Compressor { .. } => "compressor",
            Self::Valve { .. } => "valve",
            Self::Reactor { .. } => "reactor",
            Self::DistillationColumn { .. } => "column",
            Self::Flash { .. } => "flash",
            Self::Absorber { .. } => "absorber",
            Self::Crystallizer { .. } => "crystallizer",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixer_ports() {
        let u = UnitOperation::Mixer {
            id: UnitId(1),
            inlets: 3,
        };
        assert_eq!(u.num_inlets(), 3);
        assert_eq!(u.num_outlets(), 1);
        assert_eq!(u.id(), UnitId(1));
        assert_eq!(u.kind(), "mixer");
    }

    #[test]
    fn flash_has_two_outlets() {
        let u = UnitOperation::Flash { id: UnitId(2) };
        assert_eq!(u.num_outlets(), 2);
        assert_eq!(u.kind(), "flash");
    }
}
