# tpt-proc-fired-equipment

> Fired equipment: fuel combustion stoichiometry and furnace efficiency by the stack-loss method.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-fired-equipment.svg)](https://crates.io/crates/tpt-proc-fired-equipment)
[![Documentation](https://docs.rs/tpt-proc-fired-equipment/badge.svg)](https://docs.rs/tpt-proc-fired-equipment)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-fired-equipment` models furnaces and fired heaters at pre-design
fidelity: fuel combustion stoichiometry with excess air, combustion-air and
fuel flows per fired duty, stack-loss efficiency, and absorbed process duty.
Part of the [tpt-process](../../../README.md) process-engineering suite.

## Overview

Combustion is built from first principles per fuel. The `Fuel` enum
(`Methane`, `Propane`, `NaturalGas` — treated as methane, `Hydrogen`)
carries the standard higher heating values (HHV, J/mol), stoichiometric
oxygen demand, and CO₂ yield. Air is 21 % O₂ by volume, so `air_per_mol_fuel`
is the oxygen demand times (1 + excess air) times the N₂/O₂ volume ratio
79/21 — 9.52 mol air per mol methane at stoichiometric, 11.43 at 20 %
excess.

`Combustion` turns that stoichiometry into plant numbers: `fuel_flow` and
`air_flow` in mol/J for a fired duty (both exactly linear in duty),
`carbon_dioxide_per_mol_fuel` for dry flue-gas CO₂ accounting, and
`furnace_efficiency` by the **stack-loss (segmental) method** — the flue
gas (air + products, mean molar cp 33 J/(mol·K)) raised from 298.15 K
ambient to the stack temperature, with losses divided by the fired HHV.
`absorbed_duty` closes the loop: Q_process = η·Q_fired. Test expectations
bracket the classic figure of merit: a methane heater at 160 °C stack and
20 % excess air lands in the η ≈ 0.85–0.95 band.

## Features

- `Fuel` — `Methane`, `Propane`, `NaturalGas`, `Hydrogen` with
  `heating_value_per_mol`, `oxygen_per_mol`, `carbon_dioxide_per_mol`.
- `Combustion::new` — a combustion model for a fuel and excess-air
  fraction (negative excess clamped to 0).
- `Combustion::air_per_mol_fuel` — mol air per mol fuel including excess.
- `Combustion::fuel_flow` / `Combustion::air_flow` — consumption per fired
  duty, mol/J.
- `Combustion::furnace_efficiency` — stack-loss efficiency at a given
  stack temperature, clamped to [0, 1].
- `Combustion::absorbed_duty` — process duty η·Q_fired.
- `Combustion::carbon_dioxide_per_mol_fuel` — flue-gas CO₂ per mol fuel.

## Installation

```toml
[dependencies]
tpt-proc-fired-equipment = "0.1"
```

## Usage

```rust
use tpt_proc_fired_equipment::{Combustion, Fuel};

// Methane fired with 20% excess air.
let combustion = Combustion::new(Fuel::Methane, 0.20);
let air = combustion.air_flow(1.0e6); // per 1 MW fired
assert!(air > 0.0);

let furnace = combustion.furnace_efficiency(433.15); // stack at 160 °C
assert!(furnace > 0.80 && furnace < 1.0);
```

## API tour

| Item | Purpose |
|---|---|
| `Fuel` | Supported fuels with HHV, O₂ demand, CO₂ yield |
| `Combustion` | Fuel + excess air; all furnace calculations |
| `Combustion::air_per_mol_fuel` | Stoichiometric + excess air demand |
| `Combustion::fuel_flow` / `air_flow` | Consumption per fired duty |
| `Combustion::furnace_efficiency` | Stack-loss (segmental) efficiency |
| `Combustion::absorbed_duty` | Duty actually delivered to the process |

## Scope and limitations

- **Single-fuel, pre-design models** — no fuel mixes, no liquid/solid fuels,
  no fuel-property input; `NaturalGas` is hard-wired to methane
  stoichiometry.
- **Higher heating value basis** — efficiencies are HHV-based; no LHV
  reporting.
- **Simplified stack loss** — a fixed mean flue-gas cp of 33 J/(mol·K) and
  fixed 298.15 K ambient; no radiation, casing, or blowdown losses, no
  acid-dew-point or recovery (economizer/air-preheater) modeling.
- **Complete combustion assumed** — no CO/soot/unburnt-carbon losses, no
  excess-air optimization or flue-gas recirculation.
- The furnace is treated as a black box between fuel and stack: no
  geometry, tubeskin temperatures, or firebox heat-flux profiles.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types.
- [`tpt-proc-heat-transfer`](../tpt-proc-heat-transfer) — the heat-transfer
  fundamentals layer of the same domain.
- [`tpt-proc-heat-network`](../tpt-proc-heat-network) — the utility targets
  a fired heater would supply.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
