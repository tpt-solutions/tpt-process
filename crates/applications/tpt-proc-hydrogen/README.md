# tpt-proc-hydrogen

> Hydrogen production energy models: electrolysis, SMR, ATR, water-gas shift.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-hydrogen.svg)](https://crates.io/crates/tpt-proc-hydrogen)
[![Documentation](https://docs.rs/tpt-proc-hydrogen/badge.svg)](https://docs.rs/tpt-proc-hydrogen)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The hydrogen-production crate of the [tpt-process](../../../README.md)
process-engineering suite: pre-design energy accounting for water
electrolysis, steam methane reforming (SMR), autothermal reforming (ATR),
and water-gas shift — explicitly not a detailed reactor model.

## Overview

Comparing hydrogen routes needs the same few numbers for every technology:
specific energy consumption, feed demand, production rate per unit of input
power or fuel, and CO₂ intensity. This crate provides them as small,
transparent models with the constants in the open.

**Electrolysis.** An [`Electrolyzer`] is typed by [`ElectrolyzerType`] —
`Alkaline`, `Pem`, or `SolidOxide` — each with a typical stack efficiency
relative to the thermodynamic ideal (0.67, 0.65, 0.90). Specific energy
consumption is SEC = LHV(H₂)/η with LHV = 120 MJ/kg and 1 kWh = 3.6 MJ
(≈ 51 kWh/kg for PEM); [`Electrolyzer::production_rate`] converts input
power to kg/s of H₂ and [`Electrolyzer::water_consumption`] applies the
8.936 kg water/kg H₂ stoichiometry plus a purification margin.

**Methane routes.** [`Smr`] carries a reformer thermal efficiency
(fuel → H₂ LHV, typically 0.74–0.86): [`Smr::fuel_per_kg_h2`] gives the
natural-gas demand, [`Smr::co2_per_kg_h2`] the unabated emissions
(5.5 kg/kg stoichiometric process CO₂ plus 0.055 kg CO₂/MJ on the fired
fuel share — ~9–10 kg CO₂ per kg H₂ industrially), and
[`Smr::production_rate`] converts fuel energy rate to H₂ rate.
[`Atr`] adds oxygen (near-thermoneutral): [`Atr::oxygen_per_kg_h2`] and
[`Atr::production_rate`] with a typical overall efficiency of 0.78–0.88.

**Shift.** [`water_gas_shift_conversion`] gives the per-stage CO conversion
fraction, falling with temperature above ~500 K and clamped to [0.55, 0.97].

Physical constants ([`HYDROGEN_LHV_MJ_KG`], [`HYDROGEN_HHV_MJ_KG`]) are
exported; the crate is `#![forbid(unsafe_code)]`.

## Features

- [`ElectrolyzerType`] — `Alkaline`, `Pem`, `SolidOxide` technologies with
  documented typical consumption ranges.
- [`Electrolyzer`] — efficiency-typed electrolysis plant with
  `Electrolyzer::new` selecting the technology's typical efficiency.
- [`Electrolyzer::specific_energy_consumption`] — kWh/kg H₂ on the LHV
  basis.
- [`Electrolyzer::production_rate`] — kg H₂/s for an input power in W.
- [`Electrolyzer::water_consumption`] — feedwater demand with a
  purification margin.
- [`Smr`] — `fuel_per_kg_h2`, `co2_per_kg_h2` (unabated), and
  `production_rate` from fuel energy input.
- [`Atr`] — `oxygen_per_kg_h2` and `production_rate`.
- [`water_gas_shift_conversion`] — temperature-dependent CO conversion per
  shift stage.
- [`HYDROGEN_LHV_MJ_KG`] / [`HYDROGEN_HHV_MJ_KG`] — 120.0 / 141.8 MJ/kg.

## Installation

```toml
[dependencies]
tpt-proc-hydrogen = "0.1"
```

## Usage

```rust
use tpt_proc_hydrogen::{Electrolyzer, ElectrolyzerType};

let pem = Electrolyzer::new(ElectrolyzerType::Pem);
let sec = pem.specific_energy_consumption(); // kWh/kg H2
assert!((47.0..=60.0).contains(&sec));

// A 100 MW plant produces roughly 2 t/h.
let rate = pem.production_rate(100.0e6); // kg/s
assert!(rate > 0.5 && rate < 0.7);
```

## API tour

| Item | Purpose |
|---|---|
| [`ElectrolyzerType`] | Alkaline / PEM / solid-oxide technology selector. |
| [`Electrolyzer`] | SEC (kWh/kg), production rate (kg/s per W), water demand. |
| [`Smr`] | Fuel per kg H₂, unabated CO₂ per kg H₂, production rate. |
| [`Atr`] | Oxygen per kg H₂, production rate. |
| [`water_gas_shift_conversion`] | Per-stage WGS CO conversion vs temperature. |
| [`HYDROGEN_LHV_MJ_KG`] / [`HYDROGEN_HHV_MJ_KG`] | H₂ heating values, MJ/kg. |

## Scope and limitations

- **Pre-design energy accounting only** (as the crate docs state): constant
  efficiencies and stoichiometric ratios, not reactor models — no kinetics,
  heat-exchanger networks, equilibrium composition, or equipment sizing.
- Emissions are unabated: no carbon capture, and the SMR CO₂ split
  (5.5 kg/kg process + 0.055 kg/MJ fired) is a fixed empirical
  parameterization.
- [`water_gas_shift_conversion`] is an empirical clamp, not an equilibrium
  calculation; two-stage HTS/LTS sequencing is up to the caller.
- No compression, storage, purification, or delivery; no cost or LCOH
  computation.
- Electrolyzer efficiencies are fixed typical values in `new`; override by
  constructing the struct directly if your vendor data differs.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — the stream and unit types of
  the suite (declared companion dependency).
- [`tpt-proc-economics`](../../data/tpt-proc-economics) — levelized-cost
  machinery for comparing these routes on LCOH (not wired together yet).
- No suite crate consumes `tpt-proc-hydrogen` yet.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
