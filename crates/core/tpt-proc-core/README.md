# tpt-proc-core

> Core domain types for chemical process engineering: material streams, compositions, unit operations, and flowsheets.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-core.svg)](https://crates.io/crates/tpt-proc-core)
[![Documentation](https://docs.rs/tpt-proc-core/badge.svg)](https://docs.rs/tpt-proc-core)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-core` defines the data model every other `tpt-proc-*` crate builds on.
It is the foundation crate of the [tpt-process](../../../README.md)
process-engineering suite.

## Overview

Every calculation in the suite — flash separations, distillation columns,
flowsheet convergence — operates on the types defined here. A
[`MaterialStream`](https://docs.rs/tpt-proc-core/latest/tpt_proc_core/stream/struct.MaterialStream.html)
carries a thermodynamic state (temperature, pressure), a [`FlowRate`], a
[`Composition`], and a [`PhaseState`], assembled through a chainable builder.
[`Flowsheet`] and [`Connection`] wire streams and unit operations into a
process flow diagram with boundary feeds and product withdrawals.

The crate also hosts the [`PropertyPackage`] trait — the single contract
thermodynamic packages implement so the flash solver and flowsheet engine can
consume any model interchangeably. Concrete implementations live in the
`tpt-proc-thermo-*` crates; this crate keeps the interface dependency-free.

Three conventions govern the whole suite and originate here:

- **Units are SI** unless a name states otherwise: temperature in K, pressure
  in Pa, energy in J, amount in mol, length in m, time in s.
- **Determinism:** compositions are index-aligned vectors ordered by the
  owning property package's component list; maps are ordered (`BTreeMap`), so
  identical inputs give identical results.
- **No panics:** library code returns `Result`; construction of physically
  invalid data (e.g. a negative mole fraction) is rejected at the boundary.

## Features

- [`MaterialStream`] builder — `new`, `with_state`, `with_flow`,
  `with_composition`, `with_phase`, `with_properties`, plus `total_flow`
  accessors.
- [`Composition`] with validated mole, mass, and volumetric bases
  (`CompositionBasis`): `from_mole_fractions`, `from_amounts`, `pure`,
  `normalized`, `get`, `sum`.
- [`FlowRate`] in mass (kg/s), molar (mol/s), or volumetric (m³/s) form;
  [`PhaseState`] covering `Liquid`, `Vapor`, `TwoPhase`, `Supercritical`, and
  `Solid`.
- [`Flowsheet`] with `add_unit`, `add_stream`, `connect`, `feed`, `withdraw`,
  `validate`, and `inlet_streams`/`outlet_streams` boundary analysis.
- Strongly-typed identifiers — `StreamId`, `UnitId`, `PortId`, `ComponentId`,
  `FlowsheetId` — so streams and units cannot be mixed up by accident.
- The [`PropertyPackage`] trait: `fugacity_coefficients`, `enthalpy`,
  `entropy`, `density`, `k_values`, and molecular-weight helpers.
- The [`UnitOperation`] enum with typed configuration structs — `Mixer`,
  `HeatExchangerConfig`, `PumpConfig`, `CompressorConfig`, `ValveConfig`,
  `ReactorConfig`, `ColumnConfig`, `AbsorberConfig`, `CrystallizerConfig` —
  each reporting `num_inlets`/`num_outlets`.
- A total error taxonomy in [`CoreError`] (invalid compositions, missing
  references, malformed flowsheets) with `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
tpt-proc-core = "0.1"
```

## Usage

```rust
use tpt_proc_core::{Composition, FlowRate, MaterialStream, PhaseState};

# fn main() -> Result<(), tpt_proc_core::CoreError> {
let composition = Composition::from_mole_fractions(&[0.7, 0.3])?;
let stream = MaterialStream::new(1, "feed")
    .with_state(350.0, 101_325.0)?
    .with_flow(FlowRate::Molar(100.0))?
    .with_composition(composition)
    .with_phase(PhaseState::Liquid);

assert!((stream.total_flow()? - 100.0).abs() < 1e-12);
# Ok(())
# }
```

## API tour

| Item | Purpose |
|---|---|
| `MaterialStream` | A named, identified stream with state, flow, composition, phase |
| `Composition` / `CompositionBasis` | Validated fraction vectors on the mole/mass/volume basis |
| `FlowRate` / `PhaseState` | Stream flow basis and phase classification |
| `Flowsheet` / `Connection` | The PFD container and its unit-to-unit and boundary edges |
| `PropertyPackage` (trait) | The thermodynamic contract implemented by the `tpt-proc-thermo-*` crates |
| `UnitOperation` + config structs | Typed unit-operation specs with port counts |
| `StreamId`, `UnitId`, `PortId`, … | Newtype ids preventing stream/unit mix-ups |
| `CoreError`, `Result` | The crate-wide error type; nothing panics |

## Scope and limitations

- SI units only; there is no unit-conversion layer. Mixed-unit inputs are a
  caller error.
- The crate defines data and contracts, not physics: it computes no
  thermodynamic properties itself — attach a package from the
  `tpt-proc-thermo-*` crates.
- Compositions are index-aligned with the owning component list; the crate
  does not re-sort or re-key fractions for you.
- Serialization is opt-in via the `serde` feature (covers streams, compositions, unit operations, connections, and flowsheets; the property package is skipped); persistence and
  exchange formats live outside this crate.
- `Flowsheet` is a structural model — it does not execute units or converge
  recycles (see `tpt-proc-topology` and `tpt-proc-flowsheet`).

## Related crates

- [`tpt-proc-thermo-core`](../../thermodynamics/tpt-proc-thermo-core) —
  implements the `PropertyPackage` trait defined here.
- [`tpt-proc-topology`](../tpt-proc-topology) — graph analysis over
  `Flowsheet` connection structures.
- [`tpt-proc-units`](../tpt-proc-units) — the `UnitBehavior` contract and
  reference units operating on `MaterialStream`s.
- [`tpt-proc-thermo-phase`](../../thermodynamics/tpt-proc-thermo-phase) —
  flash calculations consuming this crate's `Composition` and property
  contracts.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
