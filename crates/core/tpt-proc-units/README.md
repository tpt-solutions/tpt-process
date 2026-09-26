# tpt-proc-units

> Unit-operation scaffolding: the behavior contract plus mixer and splitter reference models.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-units.svg)](https://crates.io/crates/tpt-proc-units)
[![Documentation](https://docs.rs/tpt-proc-units/badge.svg)](https://docs.rs/tpt-proc-units)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Defines *how a unit behaves* — a pure mapping from inlet streams to outlet
streams — plus the two units whose physics need no thermodynamics. Part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

Richer units (heat exchangers, pumps, reactors, …) implement the same
[`UnitBehavior`] contract in their own `tpt-proc-*` crates so the flowsheet
solver can execute any mixture of them through a [`UnitRegistry`]. This crate
fixes that contract and ships the two reference implementations whose
physics are pure bookkeeping: [`Mixer`] and [`Splitter`].

The unit contract is deliberately stateless: `solve` is a pure function of
the inlets and the unit's own parameters. State lives in the flowsheet's
streams, which keeps sequential-modular execution trivially reproducible.

[`Mixer`] blends any number of inlets into one outlet with an exact mass
balance, a flow-weighted outlet temperature, the minimum inlet pressure, and
flow-weighted composition. [`Splitter`] divides one inlet into any number of
outlets; specified split ratios are normalized to sum to one, and each outlet
inherits the inlet composition, temperature, and pressure.

## Features

- [`UnitBehavior`] trait — `kind`, `num_inlets`, `num_outlets`, and the pure
  `solve(&[MaterialStream]) -> Result<Vec<MaterialStream>, UnitError>`
  mapping; `Send + Sync` for parallel flowsheet execution.
- [`Mixer`] — N-inlet, 1-outlet blending with exact flow conservation and
  flow-weighted outlet temperature and composition.
- [`Splitter::new`] — 1-inlet, N-outlet division with ratio normalization
  (`vec![0.7, 0.4, 0.1]` behaves exactly like weights).
- [`UnitRegistry`] — ordered (`BTreeMap`) mapping of `UnitId` to
  `Arc<dyn UnitBehavior>`, with `register`, `get`, and `execute` dispatch.
- [`UnitError`] taxonomy — `WrongInletCount`, `MissingStreamData`,
  `UnknownUnit`, `Unsupported`, `NotConverged` — plus automatic conversion
  from `tpt_proc_core::CoreError`.
- `#![forbid(unsafe_code)]`; no hidden mutable state anywhere.

## Installation

```toml
[dependencies]
tpt-proc-units = "0.1"
```

## Usage

```rust
use tpt_proc_core::{Composition, FlowRate, MaterialStream};
use tpt_proc_units::{Mixer, UnitBehavior};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mixer = Mixer;
let a = MaterialStream::new(1, "a")
    .with_state(300.0, 101_325.0)?
    .with_flow(FlowRate::Molar(60.0))?
    .with_composition(Composition::from_mole_fractions(&[1.0, 0.0])?);
let b = MaterialStream::new(2, "b")
    .with_state(350.0, 101_325.0)?
    .with_flow(FlowRate::Molar(40.0))?
    .with_composition(Composition::from_mole_fractions(&[0.0, 1.0])?);

let outlets = mixer.solve(&[a, b])?;
assert_eq!(outlets.len(), 1);
// 60 + 40 mol/s conserved on the molar basis.
assert!((outlets[0].total_flow()? - 100.0).abs() < 1e-9);
# Ok(())
# }
```

## API tour

| Item | Purpose |
|---|---|
| `UnitBehavior` (trait) | The stateless inlet-to-outlet contract every unit implements |
| `Mixer` | Reference N-to-1 blending unit (no thermodynamics needed) |
| `Splitter` | Reference 1-to-N division unit with normalized ratios |
| `UnitRegistry` | Maps `UnitId` → behavior; dispatch point for the flowsheet solver |
| `UnitError` | Wrong inlet counts, missing stream data, unknown units, non-convergence |

## Scope and limitations

- Only `Mixer` and `Splitter` are included here; equipment with real physics
  (heat exchangers, pumps, reactors) lives in its own `tpt-proc-*` crates and
  plugs in through the same `UnitBehavior` trait.
- `Mixer` uses flow-weighted temperature and minimum inlet pressure — an
  adiabatic, pressure-driven idealization, not an enthalpy balance with heat
  of mixing.
- `Splitter` scales every extensive property by the split ratio; it cannot
  separate (for that, use the flash or separations crates).
- No unit has tuning parameters beyond the splitter ratios; the crate does
  not model hydraulics, holdup, or dynamics.

## Related crates

- [`tpt-proc-core`](../tpt-proc-core) — upstream: `MaterialStream`, `UnitId`,
  and `CoreError` come from here.
- [`tpt-proc-topology`](../tpt-proc-topology) — computes the order in which a
  solver executes registered units.
- [`tpt-proc-flowsheet`](../../simulation/tpt-proc-flowsheet) — downstream
  consumer: drives `UnitRegistry::execute` during sequential-modular passes.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
