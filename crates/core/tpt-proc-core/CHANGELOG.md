# Changelog

All notable changes to `tpt-proc-core` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- `MaterialStream` builder: `new`, `with_state`, `with_flow`, `with_composition`, `with_phase`, `with_properties`, `with_id`, plus `total_flow` and property accessors.
- `Composition` with validated mole, mass, and volumetric bases (`CompositionBasis`): `from_mole_fractions`, `from_parts`, `from_amounts`, `pure`, `normalized`, `get`, `as_slice`, `sum`.
- `FlowRate` (mass kg/s, molar mol/s, volumetric m³/s) and `PhaseState` (`Liquid`, `Vapor`, `TwoPhase`, `Supercritical`, `Solid`).
- `Flowsheet` and `Connection` with `add_unit`, `add_stream`, `connect`, `feed`, `withdraw`, `validate`, and `inlet_streams`/`outlet_streams` boundary analysis.
- The `PropertyPackage` trait — the thermodynamic contract (`fugacity_coefficients`, `enthalpy`, `entropy`, `density`, `k_values`) implemented by the `tpt-proc-thermo-*` crates.
- `UnitOperation` enum with typed configuration structs: `HeatExchangerConfig`, `PumpConfig`, `CompressorConfig`, `ValveConfig`, `ReactorConfig`, `ColumnConfig`, `AbsorberConfig`, `CrystallizerConfig`.
- Newtype identifiers `StreamId`, `UnitId`, `PortId`, `ComponentId`, and `FlowsheetId`.
- The `CoreError`/`Result` error taxonomy; construction of invalid data returns errors, and the crate carries `#![forbid(unsafe_code)]`.

### Added
- `serde` feature: `Serialize`/`Deserialize` for `MaterialStream`, `Composition`, `FlowRate`, `PhaseState`, `StreamProperties` (unset NaN fields map to JSON `null`), `UnitOperation` and its config types, `Connection`, `Flowsheet` (the property package is skipped), and the strongly typed ids; JSON round-trip tests included.
