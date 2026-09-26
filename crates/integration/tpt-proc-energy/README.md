# tpt-proc-energy

> Pinch utility targets from `tpt-process`, registered as loads and CHP capacity in `tpt-energy` power-flow models.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A workspace-internal integration bridge of the [tpt-process](../../../README.md)
process-engineering suite. It connects the process side (pinch analysis in
`tpt-proc-heat-network`) to the energy side (electrical system planning in
`tpt-energy` via `tpt-nrg-core`). **This crate is not published to crates.io**: it
depends on git dependencies pinned to substrate revisions and is consumed as a git or
path dependency inside the TPT workspace.

## Overview

The process side computes *thermal* quantities: minimum heating and cooling duties in
watts ([`tpt_proc_heat_network::PinchAnalysis`] → `UtilityTargets`). The energy side
plans *electrical* systems in MW with buses, generators, and loads
(`tpt_nrg_core::EnergySystem`). [`UtilityBridge`] is the deliberate, documented
translation between the two:

- **Cooling duty → chiller electricity.** A cooling duty `Q_cold` becomes a chiller
  load of `Q_cold / (COP × 10⁶)` MW, with COP an explicit assumption
  ([`UtilityBridge::chiller_cop`], default 4.0).
- **Heating duty → CHP or electric heating.** When enabled
  ([`UtilityBridge::chp_electrical_efficiency`] > 0, default 0.40), the heating duty
  is served by a CHP unit registered as a thermal `Generator` sized to its electrical
  output, `Q_heat × η_elec / 10⁶` MW. Otherwise the duty becomes an electric-heating
  load scaled by [`UtilityBridge::electric_heating_fraction`] — 0 by default, i.e. no
  silent electrification of heat.
- **First-law bookkeeping.** [`UtilityBridge::apply`] registers the base process
  electrical load, the chiller load, and the CHP generator (or electric-heating load)
  on a chosen [`tpt_nrg_core::Bus`], returning an [`AppliedUtilities`] record of
  everything it registered. Zero duties register nothing beyond the process load.

The translation conventions follow standard utility-planning practice — chiller
electricity is duty divided by COP, CHP is credited at its electrical output — and
the assumptions are data on the bridge struct, not hidden constants, so studies can
sweep them.

## Features

- [`UtilityBridge`] — configurable translation assumptions: `chiller_cop`, `chp_electrical_efficiency`, `electric_heating_fraction`.
- [`UtilityBridge::new`] / `Default` — explicit or typical-default assumptions (COP 4.0, η_elec 0.40, no electric heating).
- [`UtilityBridge::apply`] — registers process, chiller, and heating loads plus optional CHP `Generator` on an `EnergySystem` bus; returns `Result<AppliedUtilities, tpt_nrg_core::CoreError>`.
- [`AppliedUtilities`] — what was registered: `chiller_load_mw`, `heating_load_mw`, `chp_capacity_mw`, `process_load_mw`.
- [`UtilityBridge::chiller_load_mw`] — standalone duty/COP → MW conversion.
- [`UtilityBridge::chp_capacity_mw`] — standalone duty × η_elec → MW conversion.
- [`utility_bus_type`] — the `BusType::Pq` needed for a pure-load utility bus.
- `#![forbid(unsafe_code)]`.

## Installation

Not on crates.io. Consume it as a git dependency of the workspace:

```toml
[dependencies]
tpt-proc-energy = { git = "https://github.com/tpt-solutions/tpt-process" }
```

Its git dependencies (`tpt-nrg-core` from `tpt-solutions/tpt-energy`) are resolved
through the workspace `[patch]`/dependency table, pinned to substrate revisions, so a
pinned revision of this repository builds reproducibly.

## Usage

```rust
use tpt_nrg_core::{Bus, BusType, EnergySystem};
use tpt_proc_energy::UtilityBridge;
use tpt_proc_heat_network::{PinchAnalysis, ProcessStream};

let pinch = PinchAnalysis::new(
    vec![ProcessStream::hot(433.15, 323.15, 10.0)],
    vec![ProcessStream::cold(318.15, 428.15, 10.0)],
    10.0,
);
let targets = pinch.minimum_utilities();

let mut system = EnergySystem::new("plant", "Process plant", 100.0, 60.0);
system.add_bus(Bus::new(1, "Utility bus", BusType::Pq)).unwrap();

let bridge = UtilityBridge::default();
let applied = bridge
    .apply(&mut system, 1, &targets, 2.0e6) // 2 MW of process power demand
    .unwrap();
assert!(applied.chiller_load_mw > 0.0);
```

## API tour

| Item | Purpose |
|---|---|
| [`UtilityBridge`] | The translation assumptions (COP, CHP efficiency, electric-heating fraction). |
| `UtilityBridge::apply` | Applies pinch targets to an `EnergySystem` at a bus. |
| [`AppliedUtilities`] | Report of the loads/generator the bridge registered, in MW. |
| `UtilityBridge::chiller_load_mw` | Cooling duty → chiller electrical load (MW). |
| `UtilityBridge::chp_capacity_mw` | Heating duty → CHP electrical capacity (MW). |
| [`utility_bus_type`] | Convenience `BusType::Pq` for a utility load bus. |

## Scope and limitations

- One-directional: pinch targets → energy-system elements. There is no reverse path
  (energy prices or dispatch results back into process decisions) and no co-optimization.
- The CHP is registered as a thermal `Generator` sized at its electrical output only;
  no fuel scheduling, emissions, steam-turbine extraction detail, or part-load curves.
- Chiller and heating conversions are steady-state scalars; no seasonal variation,
  storage, or time-series profiles.
- Only element *registration* is performed — no power-flow solving, which remains
  `tpt-energy`'s job.

## Related crates

- [`tpt-proc-heat-network`](../../heat-transfer/tpt-proc-heat-network) — upstream: produces the `UtilityTargets` (pinch minimum utilities) the bridge consumes.
- [`tpt-nrg-core`](https://github.com/tpt-solutions/tpt-energy) — downstream substrate: `EnergySystem`, `Bus`, `Load`, `Generator` (git dependency, pinned revision).
- [`tpt-proc-construction`](../tpt-proc-construction) — sibling integration bridge, PFD → construction models.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
