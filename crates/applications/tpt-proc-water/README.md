# tpt-proc-water

> Water treatment train simulation: coagulation to reverse osmosis and disinfection.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-water.svg)](https://crates.io/crates/tpt-proc-water)
[![Documentation](https://docs.rs/tpt-proc-water/badge.svg)](https://docs.rs/tpt-proc-water)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The water-treatment crate of the [tpt-process](../../../README.md)
process-engineering suite: compose a treatment train as a list of processes
and simulate a [`WaterQuality`] stream through it, from raw water to
disinfected product.

## Overview

A treatment train is modeled the way the suite models everything else: as a
sequence of transformations on a process stream. Here the stream is a
[`WaterQuality`] state carrying four lumped indicators — turbidity (NTU),
total dissolved solids (mg/L), hardness as CaCO₃ (mg/L), and coliform count
(CFU/mL).

Each treatment step is one variant of the [`Process`] enum, and the free
function [`apply`] transforms a quality state through a single step:

- `Coagulation` — sweep-floc removal scaling with coagulant dose (up to ~70%
  of turbidity at typical doses, with partial coliform removal).
- `Flocculation` — gentle additional turbidity removal that saturates with
  contact time.
- `Sedimentation` / `Filtration` — fractional removal of remaining turbidity
  (filtration also removes ~90% of its fraction from coliform).
- `IonExchange` — softening against a finite exchange capacity (mg/L as
  CaCO₃); TDS drops as Na swaps for Ca/Mg.
- `ReverseOsmosis` — desalination by salt rejection; permeate TDS is
  feed·(1 − rejection).
- `Disinfection` — CT (concentration × time) concept: log removal =
  k·CT, applied as a power of ten to the coliform count.

A [`Train`] holds the ordered process list; [`Train::simulate`] folds the
steps over the raw water in declaration order, so a conventional
coagulation → flocculation → sedimentation → filtration → disinfection train
can be checked against drinking-water quality targets in a handful of lines.

## Features

- [`WaterQuality`] — the four-indicator process state (turbidity NTU, TDS
  mg/L, hardness mg/L as CaCO₃, coliform CFU/mL).
- [`Process`] — the seven-step treatment enum: `Coagulation`,
  `Flocculation`, `Sedimentation`, `Filtration`, `IonExchange`,
  `ReverseOsmosis`, `Disinfection`.
- [`Train`] — an ordered process list with `Train::new`,
  `Train::processes`, and `Train::simulate`.
- [`apply`] — apply a single [`Process`] to a [`WaterQuality`] state
  (public, so steps can be studied in isolation).
- Dose-, capacity-, and CT-based parameterizations (mg/L coagulant dose,
  mg/L exchange capacity, mg·min/L CT values).
- Log-removal disinfection arithmetic (10⁻ᵏ·ᶜᵗ scaling of coliform).
- An empty [`Train`] is an exact no-op; quality states are `Copy` value
  types.
- `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
tpt-proc-water = "0.1"
```

## Usage

```rust
use tpt_proc_water::{Process, Train, WaterQuality};

let raw = WaterQuality::new(25.0, 800.0, 250.0, 10_000.0);
let train = Train::new(vec![
    Process::Coagulation { dose_mg_l: 30.0 },
    Process::Filtration { removal_fraction: 0.95 },
    Process::ReverseOsmosis { rejection: 0.98, recovery: 0.75 },
    Process::Disinfection { ct_value: 40.0, removal_rate: 4.0 },
]);
let out = train.simulate(&raw);
assert!(out.turbidity_ntu < 1.0);
assert!(out.tds_mg_l < 800.0 * 0.05);
assert!(out.coliform_cfu_ml < 1.0);
```

## API tour

| Item | Purpose |
|---|---|
| [`WaterQuality`] | Turbidity, TDS, hardness, coliform state of the water. |
| [`Process`] | One treatment step (coagulation … disinfection). |
| [`Train`] | Ordered treatment train; `simulate` folds steps over raw water. |
| [`apply`] | Transform a quality state through a single step. |

## Scope and limitations

- Four lumped indicators only: no pH, alkalinity, ions, disinfection
  by-products, metals, or organics; coliform stands in for microbiology.
- Removal models are empirical lumped fractions and saturating dose curves —
  not jar-test kinetics, coagulation chemistry, or filter hydraulics.
- No hydraulics or sizing: no tank volumes, weir rates, filter areas,
  membrane area/flux, headloss, or pumping energy.
- `ReverseOsmosis` recovery is accepted but does not yet affect permeate
  quality or reject flow; only the rejection fraction acts on the stream.
- No cost estimation, chemical consumption, or sludge production.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — the stream and unit types of
  the suite (declared companion dependency).
- [`example-water-treatment-plant`](../../../examples/water-treatment-plant)
  — the worked example binary built on this crate.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
