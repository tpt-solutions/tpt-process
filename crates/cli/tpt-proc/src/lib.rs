//! `tpt-proc` library core: config schema, behavior adapters, and the
//! config→flowsheet runner. The `main.rs` binary is a thin argv/JSON shell
//! over [`run_config_str`].
//!
//! # Config format
//!
//! A TOML document describes one flowsheet: an optional thermodynamic
//! model, boundary streams, unit operations, and connections. See
//! `configs/recycle-demo.toml` for a complete example.
//!
//! # Supported unit kinds
//!
//! [`UnitKind::Mixer`] and [`UnitKind::Splitter`] run on flow bookkeeping
//! alone. [`UnitKind::Flash`], [`UnitKind::Heater`], [`UnitKind::Cooler`],
//! [`UnitKind::Valve`], and [`UnitKind::Pump`] re-equilibrate their outlet
//! with a PT flash, which requires `[thermo]` in the config.

use std::sync::Arc;

use serde::Deserialize;
use tpt_proc_core::{
    Composition, CoreError, FlowRate, Flowsheet, MaterialStream, PhaseState, PortId, StreamId,
    UnitId,
};
use tpt_proc_flowsheet::{FlowsheetError, FlowsheetResult, FlowsheetSolver};
use tpt_proc_thermo_core::{Component, PropertyPackage};
use tpt_proc_thermo_database::ChemicalDatabase;
use tpt_proc_thermo_eos::CubicEos;
use tpt_proc_thermo_phase::{FlashError, FlashSolver};
use tpt_proc_units::{Splitter, UnitBehavior, UnitError, UnitRegistry};

/// A runner failure: config, thermodynamics, or solver stage.
#[derive(Debug)]
pub enum CliError {
    /// The TOML config could not be read or parsed.
    Config(String),
    /// The config referenced something the engine cannot provide.
    Unknown(String),
    /// Stream/unit data was internally inconsistent.
    Invalid(String),
    /// The underlying property package or flash failed.
    Thermo(String),
    /// The solver failed structurally.
    Flowsheet(FlowsheetError),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(m) => write!(f, "config error: {m}"),
            Self::Unknown(m) => write!(f, "unknown {m}"),
            Self::Invalid(m) => write!(f, "invalid flowsheet: {m}"),
            Self::Thermo(e) => write!(f, "thermodynamics failure: {e}"),
            Self::Flowsheet(e) => write!(f, "solver failure: {e}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<FlowsheetError> for CliError {
    fn from(e: FlowsheetError) -> Self {
        Self::Flowsheet(e)
    }
}

/// Equation of state backing the property package.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EosChoice {
    /// Peng-Robinson (default).
    #[default]
    PengRobinson,
    /// Soave-Redlich-Kwong.
    Srk,
    /// Ideal solution (Raoult/Trouton package, no EOS attached).
    Ideal,
}

/// Unit kinds the runner can instantiate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnitKind {
    /// Adiabatic merger of streams.
    Mixer,
    /// Flow divider with fixed ratios.
    Splitter,
    /// Flash drum at fixed T and/or P (two outlets: vapor, liquid).
    Flash,
    /// Heater to a target outlet temperature.
    Heater,
    /// Cooler to a target outlet temperature.
    Cooler,
    /// Let-down valve to a fixed outlet pressure.
    Valve,
    /// Pump with a fixed pressure rise.
    Pump,
}

/// Top-level TOML config.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Flowsheet identity.
    pub flowsheet: FlowsheetSpec,
    /// Thermodynamic model (optional; needed by flash-derived units).
    #[serde(default)]
    pub thermo: Option<ThermoSpec>,
    /// Boundary streams.
    #[serde(default)]
    pub streams: Vec<StreamSpec>,
    /// Unit operations.
    #[serde(default)]
    pub units: Vec<UnitSpec>,
    /// Connections (feeds, internal links, product withdrawals).
    #[serde(default)]
    pub connections: Vec<ConnectionSpec>,
}

/// `[flowsheet]` section.
#[derive(Debug, Deserialize)]
pub struct FlowsheetSpec {
    /// Numeric flowsheet id.
    pub id: u64,
    /// Human-readable name.
    #[serde(default)]
    pub name: String,
}

/// `[thermo]` section.
#[derive(Debug, Deserialize)]
pub struct ThermoSpec {
    /// Component names from the built-in database, in package order.
    pub components: Vec<String>,
    /// Equation of state (default Peng-Robinson).
    #[serde(default)]
    pub eos: EosChoice,
}

/// One `[[streams]]` entry: a fully specified boundary feed.
#[derive(Debug, Deserialize)]
pub struct StreamSpec {
    /// Numeric stream id.
    pub id: u64,
    /// Human-readable name (defaults to `S<id>`).
    #[serde(default)]
    pub name: Option<String>,
    /// Feed temperature, K.
    pub temperature_k: f64,
    /// Feed pressure, Pa.
    pub pressure_pa: f64,
    /// Molar flow, mol/s.
    pub molar_flow_mol_s: f64,
    /// Mole fractions aligned with `[thermo] components`.
    pub mole_fractions: Vec<f64>,
}

/// One `[[units]]` entry.
#[derive(Debug, Deserialize)]
pub struct UnitSpec {
    /// Equipment kind.
    pub kind: UnitKind,
    /// Numeric unit id.
    pub id: u64,
    /// Mixer: number of inlet ports (≥ 2).
    #[serde(default)]
    pub inlets: Option<usize>,
    /// Splitter: fraction of feed to each outlet (normalized).
    #[serde(default)]
    pub ratios: Option<Vec<f64>>,
    /// Flash: fixed temperature, K (`None` = pass inlet T).
    #[serde(default)]
    pub temperature_k: Option<f64>,
    /// Flash/valve: fixed outlet pressure, Pa (`None` = pass inlet P).
    #[serde(default)]
    pub pressure_pa: Option<f64>,
    /// Pump: pressure rise, Pa.
    #[serde(default)]
    pub pressure_rise_pa: Option<f64>,
}

/// One `[[connections]]` entry.
#[derive(Debug, Deserialize)]
pub struct ConnectionSpec {
    /// Carrying stream id.
    pub stream: u64,
    /// Producing unit (`None` = boundary feed).
    #[serde(default)]
    pub from_unit: Option<u64>,
    /// Producing outlet port (default 0).
    #[serde(default)]
    pub from_port: u64,
    /// Consuming unit (`None` = product withdrawal).
    #[serde(default)]
    pub to_unit: Option<u64>,
    /// Consuming inlet port (default 0).
    #[serde(default)]
    pub to_port: u64,
}

/// PT-flash-backed outlet adapter: re-equilibrates the inlet at a fixed
/// and/or inherited (T, P). Also serves heater/cooler (target T), valve
/// (target P), and pump (P rise) via the same mechanism.
struct FlashUnit {
    solver: FlashSolver,
    kind: &'static str,
    temperature: Option<f64>,
    pressure: Option<f64>,
    pressure_rise: Option<f64>,
    outlets: usize,
}

impl FlashUnit {
    /// Resolves the (T, P) to flash at: explicit specs win, otherwise the
    /// inlet state carries over (pump adds its rise to the inlet P).
    fn reflash(
        &self,
        feed: &MaterialStream,
    ) -> Result<tpt_proc_thermo_phase::FlashResult, UnitError> {
        let ok = |v: f64| v.is_finite() && v > 0.0;
        let t = self
            .temperature
            .filter(|t| ok(*t))
            .or_else(|| {
                let t = feed.temperature();
                ok(t).then_some(t)
            })
            .ok_or_else(|| {
                UnitError::MissingStreamData("inlet has no finite temperature".into())
            })?;
        let p = self
            .pressure
            .or_else(|| self.pressure_rise.map(|rise| feed.pressure() + rise))
            .filter(|p| ok(*p))
            .or_else(|| {
                let p = feed.pressure();
                ok(p).then_some(p)
            })
            .ok_or_else(|| {
                UnitError::MissingStreamData("inlet has no finite positive pressure".into())
            })?;
        let composition = feed
            .composition()
            .ok_or_else(|| UnitError::MissingStreamData("inlet has no composition".into()))?;
        self.solver
            .pt_flash(composition, t, p)
            .map_err(|e: FlashError| UnitError::Unsupported(e.to_string()))
    }

    /// Single outlet carrying the full flow at the flashed state.
    fn single_outlet(&self, inlets: &[MaterialStream]) -> Result<Vec<MaterialStream>, UnitError> {
        let feed = inlets.first().ok_or(UnitError::WrongInletCount {
            expected: 1,
            got: inlets.len(),
        })?;
        let result = self.reflash(feed)?;
        let flow = feed
            .total_flow()
            .map_err(|e| UnitError::MissingStreamData(e.to_string()))?;
        let beta = result.vapor_fraction;
        let phase = phase_for(beta);
        let outlet =
            MaterialStream::new(0, "outlet")
                .with_state(result.temperature, result.pressure)
                .map_err(|e| UnitError::MissingStreamData(e.to_string()))?
                .with_flow(FlowRate::Molar(flow))
                .map_err(|e| UnitError::MissingStreamData(e.to_string()))?
                .with_composition(feed.composition().cloned().ok_or_else(|| {
                    UnitError::MissingStreamData("inlet has no composition".into())
                })?)
                .with_phase(phase);
        Ok(vec![outlet])
    }
}

impl UnitBehavior for FlashUnit {
    fn kind(&self) -> &'static str {
        self.kind
    }

    fn num_inlets(&self) -> usize {
        1
    }

    fn num_outlets(&self) -> usize {
        self.outlets
    }

    fn solve(&self, inlets: &[MaterialStream]) -> Result<Vec<MaterialStream>, UnitError> {
        if inlets.len() != 1 {
            return Err(UnitError::WrongInletCount {
                expected: 1,
                got: inlets.len(),
            });
        }
        if self.outlets == 1 {
            return self.single_outlet(inlets);
        }
        // Two outlets: vapor (β) and liquid (1 − β).
        let feed = &inlets[0];
        let result = self.reflash(feed)?;
        let flow = feed
            .total_flow()
            .map_err(|e| UnitError::MissingStreamData(e.to_string()))?;
        let beta = result.vapor_fraction.clamp(0.0, 1.0);
        let mk = |name: &str,
                  fraction: f64,
                  composition: &Composition,
                  phase: PhaseState|
         -> Result<MaterialStream, UnitError> {
            Ok(MaterialStream::new(0, name)
                .with_state(result.temperature, result.pressure)
                .map_err(|e| UnitError::MissingStreamData(e.to_string()))?
                .with_flow(FlowRate::Molar(fraction * flow))
                .map_err(|e| UnitError::MissingStreamData(e.to_string()))?
                .with_composition(composition.clone())
                .with_phase(phase))
        };
        Ok(vec![
            mk("vapor", beta, &result.vapor_composition, PhaseState::Vapor)?,
            mk(
                "liquid",
                1.0 - beta,
                &result.liquid_composition,
                PhaseState::Liquid,
            )?,
        ])
    }
}

/// Phase classification from a converged vapor fraction.
fn phase_for(beta: f64) -> PhaseState {
    if beta <= 1e-9 {
        PhaseState::Liquid
    } else if beta >= 1.0 - 1e-9 {
        PhaseState::Vapor
    } else {
        PhaseState::TwoPhase {
            vapor_fraction: beta,
        }
    }
}

/// Runs a TOML config document and returns the solver outcome plus the
/// component list the package was built from.
///
/// # Errors
/// [`CliError`] at whichever stage fails: config parsing, database
/// lookup, stream/unit construction, or the solve itself.
pub fn run_config_str(config_text: &str) -> Result<(FlowsheetResult, Vec<String>), CliError> {
    let config: Config =
        toml::from_str(config_text).map_err(|e| CliError::Config(e.to_string()))?;
    run_config(&config)
}

/// Runs a parsed config. See [`run_config_str`].
///
/// # Errors
/// See [`run_config_str`].
pub fn run_config(config: &Config) -> Result<(FlowsheetResult, Vec<String>), CliError> {
    let mut fs = Flowsheet::new(config.flowsheet.id, config.flowsheet.name.clone());

    // Thermodynamics: package from the built-in database.
    let mut flash_solver: Option<FlashSolver> = None;
    let mut component_names: Vec<String> = Vec::new();
    if let Some(thermo) = &config.thermo {
        let db = ChemicalDatabase::builtin();
        let names = &thermo.components;
        if names.is_empty() {
            return Err(CliError::Invalid(
                "[thermo] components must list at least one component".into(),
            ));
        }
        let components: Vec<Component> = db
            .components_for(&names.iter().map(String::as_str).collect::<Vec<_>>())
            .map_err(|e: CoreError| CliError::Unknown(e.to_string()))?;
        let package =
            build_package(&components, thermo.eos).map_err(|e| CliError::Thermo(e.to_string()))?;
        flash_solver = Some(FlashSolver::new(package));
        component_names = names.clone();
    }

    // Streams.
    for spec in &config.streams {
        let composition = Composition::from_mole_fractions(&spec.mole_fractions)
            .map_err(|e| CliError::Invalid(format!("stream {}: {e}", spec.id)))?;
        let stream = MaterialStream::new(
            spec.id,
            spec.name.clone().unwrap_or_else(|| format!("S{}", spec.id)),
        )
        .with_state(spec.temperature_k, spec.pressure_pa)
        .map_err(|e| CliError::Invalid(format!("stream {}: {e}", spec.id)))?
        .with_flow(FlowRate::Molar(spec.molar_flow_mol_s))
        .map_err(|e| CliError::Invalid(format!("stream {}: {e}", spec.id)))?
        .with_composition(composition);
        fs.add_stream(stream);
    }

    // Units: specs plus behaviors.
    let mut registry = UnitRegistry::new();
    for spec in &config.units {
        let unit_id = UnitId(spec.id);
        match spec.kind {
            UnitKind::Mixer => {
                let inlets = spec.inlets.unwrap_or(2).max(2);
                fs.add_unit(tpt_proc_core::UnitOperation::Mixer {
                    id: unit_id,
                    inlets,
                });
                registry.register(unit_id, Arc::new(tpt_proc_units::Mixer));
            }
            UnitKind::Splitter => {
                let ratios = spec.ratios.clone().ok_or_else(|| {
                    CliError::Invalid(format!("splitter {}: `ratios` is required", spec.id))
                })?;
                if ratios.len() < 2 {
                    return Err(CliError::Invalid(format!(
                        "splitter {}: at least two ratios are required",
                        spec.id
                    )));
                }
                fs.add_unit(tpt_proc_core::UnitOperation::Splitter {
                    id: unit_id,
                    split_ratios: ratios.clone(),
                });
                registry.register(unit_id, Arc::new(Splitter::new(ratios)));
            }
            UnitKind::Flash => {
                let solver = require_flash(&flash_solver)?;
                fs.add_unit(tpt_proc_core::UnitOperation::Flash { id: unit_id });
                registry.register(
                    unit_id,
                    Arc::new(FlashUnit {
                        solver,
                        kind: "flash",
                        temperature: spec.temperature_k,
                        pressure: spec.pressure_pa,
                        pressure_rise: None,
                        outlets: 2,
                    }),
                );
            }
            UnitKind::Heater | UnitKind::Cooler => {
                let solver = require_flash(&flash_solver)?;
                let target = spec.temperature_k.ok_or_else(|| {
                    CliError::Invalid(format!(
                        "{} {}: `temperature_k` (outlet target) is required",
                        if spec.kind == UnitKind::Heater {
                            "heater"
                        } else {
                            "cooler"
                        },
                        spec.id
                    ))
                })?;
                let kind = if spec.kind == UnitKind::Heater {
                    "heater"
                } else {
                    "cooler"
                };
                fs.add_unit(tpt_proc_core::UnitOperation::HeatExchanger {
                    id: unit_id,
                    config: tpt_proc_core::HeatExchangerConfig {
                        area: 0.0,
                        overall_u: 0.0,
                        fouling: 0.0,
                        duty: None,
                    },
                });
                registry.register(
                    unit_id,
                    Arc::new(FlashUnit {
                        solver,
                        kind,
                        temperature: Some(target),
                        pressure: None,
                        pressure_rise: None,
                        outlets: 1,
                    }),
                );
            }
            UnitKind::Valve => {
                let solver = require_flash(&flash_solver)?;
                let outlet = spec.pressure_pa.ok_or_else(|| {
                    CliError::Invalid(format!(
                        "valve {}: `pressure_pa` (outlet) is required",
                        spec.id
                    ))
                })?;
                fs.add_unit(tpt_proc_core::UnitOperation::Valve {
                    id: unit_id,
                    config: tpt_proc_core::ValveConfig {
                        cv: 0.0,
                        opening: 1.0,
                        characteristic: tpt_proc_core::units::ValveCharacteristic::Linear,
                    },
                });
                registry.register(
                    unit_id,
                    Arc::new(FlashUnit {
                        solver,
                        kind: "valve",
                        temperature: None,
                        pressure: Some(outlet),
                        pressure_rise: None,
                        outlets: 1,
                    }),
                );
            }
            UnitKind::Pump => {
                let solver = require_flash(&flash_solver)?;
                let rise = spec.pressure_rise_pa.ok_or_else(|| {
                    CliError::Invalid(format!("pump {}: `pressure_rise_pa` is required", spec.id))
                })?;
                fs.add_unit(tpt_proc_core::UnitOperation::Pump {
                    id: unit_id,
                    config: tpt_proc_core::PumpConfig {
                        curve: [0.0; 3],
                        speed_rpm: 0.0,
                        efficiency: 1.0,
                        pressure_rise: Some(rise),
                    },
                });
                registry.register(
                    unit_id,
                    Arc::new(FlashUnit {
                        solver,
                        kind: "pump",
                        temperature: None,
                        pressure: None,
                        pressure_rise: Some(rise),
                        outlets: 1,
                    }),
                );
            }
        }
    }

    // Connections. Streams referenced by connections but not declared in
    // [[streams]] are registered as zero-flow placeholders seeded with the
    // first declared stream's state — units may execute *before* their
    // tear-stream upstream on the first pass, and behaviors like the mixer
    // require finite T/P on every inlet. The solver replaces placeholder
    // state as soon as the producing unit runs. A boundary feed
    // (from_unit absent), however, must be fully declared — nothing ever
    // writes it.
    let template = config
        .streams
        .first()
        .map(|s| (s.temperature_k, s.pressure_pa, s.mole_fractions.clone()));
    for spec in &config.connections {
        let stream = StreamId(spec.stream);
        if fs.stream(stream).is_some() {
            continue;
        }
        if spec.from_unit.is_none() {
            return Err(CliError::Invalid(format!(
                "boundary feed stream {} must be declared in [[streams]] with full state",
                spec.stream
            )));
        }
        let mut placeholder = MaterialStream::new(spec.stream, format!("S{}", spec.stream));
        if let Some((t, p, fractions)) = &template {
            if let Ok(composition) = Composition::from_mole_fractions(fractions) {
                placeholder = placeholder
                    .with_state(*t, *p)
                    .map_err(|e| CliError::Invalid(e.to_string()))?
                    .with_flow(FlowRate::Molar(0.0))
                    .map_err(|e| CliError::Invalid(e.to_string()))?
                    .with_composition(composition);
            }
        }
        fs.add_stream(placeholder);
    }
    for spec in &config.connections {
        let stream = StreamId(spec.stream);
        match (spec.from_unit, spec.to_unit) {
            (Some(from), Some(to)) => fs
                .connect(
                    UnitId(from),
                    PortId(spec.from_port),
                    UnitId(to),
                    PortId(spec.to_port),
                    stream,
                )
                .map_err(|e| CliError::Invalid(e.to_string()))?,
            (None, Some(to)) => fs
                .feed(UnitId(to), PortId(spec.to_port), stream)
                .map_err(|e| CliError::Invalid(e.to_string()))?,
            (Some(from), None) => fs
                .withdraw(UnitId(from), PortId(spec.from_port), stream)
                .map_err(|e| CliError::Invalid(e.to_string()))?,
            (None, None) => {
                return Err(CliError::Invalid(format!(
                    "connection on stream {} has neither endpoint",
                    spec.stream
                )))
            }
        }
    }

    let solver = FlowsheetSolver::new(fs, registry);
    let result = solver.run(1e-8, 200)?;
    Ok((result, component_names))
}

fn require_flash(flash_solver: &Option<FlashSolver>) -> Result<FlashSolver, CliError> {
    flash_solver.clone().ok_or_else(|| {
        CliError::Invalid(
            "this unit kind needs thermodynamics; add a [thermo] section with components".into(),
        )
    })
}

fn build_package(
    components: &[Component],
    eos: EosChoice,
) -> Result<PropertyPackage, tpt_proc_thermo_core::ThermoError> {
    let components = components.to_vec();
    let package = PropertyPackage::new(components.clone())?;
    match eos {
        EosChoice::Ideal => Ok(package),
        EosChoice::PengRobinson => {
            let cubic = CubicEos::peng_robinson(components);
            Ok(package.with_eos(Arc::new(cubic)))
        }
        EosChoice::Srk => {
            let cubic = CubicEos::srk(components);
            Ok(package.with_eos(Arc::new(cubic)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEMO: &str = r#"
[flowsheet]
id = 1
name = "recycle demo"

[thermo]
components = ["water", "methanol"]
eos = "peng-robinson"

[[streams]]
id = 1
name = "fresh feed"
temperature_k = 350.0
pressure_pa = 101325.0
molar_flow_mol_s = 100.0
mole_fractions = [0.5, 0.5]

[[units]]
kind = "mixer"
id = 10
inlets = 2

[[units]]
kind = "heater"
id = 11
temperature_k = 360.0

[[units]]
kind = "splitter"
id = 12
ratios = [0.8, 0.2]

[[connections]]
stream = 1
to_unit = 10
to_port = 0

[[connections]]
stream = 21
from_unit = 10
from_port = 0
to_unit = 11
to_port = 0

[[connections]]
stream = 22
from_unit = 11
from_port = 0
to_unit = 12
to_port = 0

[[connections]]
stream = 23
from_unit = 12
from_port = 0

[[connections]]
stream = 24
from_unit = 12
from_port = 1
to_unit = 10
to_port = 1
"#;

    #[test]
    fn demo_config_runs_and_converges_the_recycle() {
        let (result, components) = run_config_str(DEMO).unwrap();
        assert_eq!(components, vec!["water", "methanol"]);
        // Feed → mixer → heater → splitter with a 20 % recycle: the loop
        // converges to M = 100 / (1 − 0.2) = 125 mol/s through the mixer.
        assert!(result.converged, "residual {}", result.tear_residual);
        assert_eq!(result.tear_values.len(), 1);
        let product = &result.streams[&StreamId(23)];
        let recycle = &result.streams[&StreamId(24)];
        assert!((product.total_flow().unwrap() - 100.0).abs() < 1e-2);
        assert!((recycle.total_flow().unwrap() - 25.0).abs() < 1e-2);
        assert!((product.temperature() - 360.0).abs() < 1e-6);
        assert!(product.phase().is_some());
    }

    #[test]
    fn flash_unit_splits_vapor_and_liquid() {
        let config_text = r#"
[flowsheet]
id = 1
name = "flash demo"

[thermo]
components = ["water", "methanol"]

[[streams]]
id = 1
name = "feed"
temperature_k = 360.0
pressure_pa = 101325.0
molar_flow_mol_s = 100.0
mole_fractions = [0.5, 0.5]

[[units]]
kind = "flash"
id = 10
temperature_k = 350.0
pressure_pa = 101325.0

[[connections]]
stream = 1
to_unit = 10
to_port = 0

[[connections]]
stream = 21
from_unit = 10
from_port = 0

[[connections]]
stream = 22
from_unit = 10
from_port = 1
"#;
        let (result, _) = run_config_str(config_text).unwrap();
        assert!(result.converged);
        let vapor = &result.streams[&StreamId(21)];
        let liquid = &result.streams[&StreamId(22)];
        let fv = vapor.total_flow().unwrap();
        let fl = liquid.total_flow().unwrap();
        assert!(
            (fv + fl - 100.0).abs() < 1e-6,
            "vapor {fv} + liquid {fl} != 100"
        );
        assert_eq!(vapor.phase(), Some(PhaseState::Vapor));
        assert_eq!(liquid.phase(), Some(PhaseState::Liquid));
        // Methanol is the more volatile component here: the vapor is never
        // leaner in methanol than the liquid (equality when β = 0).
        let y_meoh = vapor.composition().unwrap().get(1).unwrap();
        let x_meoh = liquid.composition().unwrap().get(1).unwrap();
        assert!(y_meoh >= x_meoh - 1e-9, "y {y_meoh} vs x {x_meoh}");
    }

    #[test]
    fn missing_thermo_is_a_config_error() {
        let config_text = r#"
[flowsheet]
id = 1
name = "no thermo"

[[units]]
kind = "flash"
id = 10
"#;
        assert!(matches!(
            run_config_str(config_text),
            Err(CliError::Invalid(_))
        ));
    }

    #[test]
    fn unknown_component_is_an_error() {
        let config_text = r#"
[flowsheet]
id = 1
name = "bad component"

[thermo]
components = ["unobtainium"]

[[streams]]
id = 1
temperature_k = 300.0
pressure_pa = 101325.0
molar_flow_mol_s = 1.0
mole_fractions = [1.0]
"#;
        assert!(matches!(
            run_config_str(config_text),
            Err(CliError::Unknown(_))
        ));
    }
}
