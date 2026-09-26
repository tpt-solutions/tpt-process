# tpt-proc-heat-transfer

> Heat-transfer fundamentals: convection correlations, Nusselt numbers, and overall coefficients.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-heat-transfer.svg)](https://crates.io/crates/tpt-proc-heat-transfer)
[![Documentation](https://docs.rs/tpt-proc-heat-transfer/badge.svg)](https://docs.rs/tpt-proc-heat-transfer)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-heat-transfer` is the fundamentals layer of the heat-transfer
domain in the [tpt-process](../../../README.md) process-engineering suite: the
dimensionless convection correlations, the property-group conversions
(Prandtl, Reynolds), and the series-resistance bookkeeping that every
exchanger rating and furnace model builds on. All correlations use SI;
Nusselt, Reynolds, and Prandtl numbers are dimensionless.

## Overview

The `convection` module collects the standard turbulent and laminar
correlations. For turbulent internal flow there is Dittus-Boelter
(`Nu = 0.023·Re⁰·⁸·Prⁿ`, n = 0.4 heating / 0.3 cooling, valid roughly
10⁴ < Re < 1.2·10⁵ and 0.7 < Pr < 120) and Sieder-Tate with the viscosity
correction `(μ/μ_w)⁰·¹⁴` for viscous liquids and large temperature
differences. For fully developed laminar internal flow the exact constant-T
and constant-q asymptotes (Nu = 3.66 and 4.36) are provided, and
`internal_flow` selects between them by Reynolds number. Flat-plate
correlations cover external laminar and turbulent flow.

`WallResistance` accumulates a series resistance stack — conductive layers
(L/k), convective films (1/h), and fouling factors — and converts it to an
overall coefficient `U = 1/(R·A)`. `conduction_flux` closes the 1-D steady
conduction picture with `q = ΔT/R`.

## Features

- `convection::dittus_boelter` — turbulent internal flow with heating/
  cooling Pr exponents.
- `convection::sieder_tate` — viscous-liquid variant with the
  (μ/μ_w)⁰·¹⁴ correction.
- `convection::laminar_internal_constant_t` / `..._constant_q` — the
  3.66 and 4.36 fully developed laminar asymptotes.
- `convection::internal_flow` — Reynolds-based selector across the above.
- `convection::flat_plate_laminar` / `convection::flat_plate_turbulent` —
  external flat-plate correlations.
- `convection::prandtl`, `convection::reynolds_internal` — dimensionless
  groups from SI transport properties.
- `convection::heat_transfer_coefficient` — h = Nu·k/L in W/(m²·K).
- `WallResistance` — builder for series R: `.conductive()`, `.convective()`,
  `.fouling()`, then `overall_u`.
- `conduction_flux` — 1-D steady flux through a resistance, W/m².
- `FlowRegime` — laminar/transition/turbulent plus heating/cooling flags.

## Installation

```toml
[dependencies]
tpt-proc-heat-transfer = "0.1"
```

## Usage

```rust
use tpt_proc_heat_transfer::{convection, FlowRegime};

// Dittus-Boelter for cooling water inside a tube (Pr > 0.7).
let nu = convection::dittus_boelter(10_000.0, 6.0, FlowRegime::Heating);
assert!(nu > 0.0);
assert!((convection::laminar_internal_constant_t() - 3.66).abs() < 1e-12);
```

A composite wall with films, a conductive layer, and fouling:

```rust
use tpt_proc_heat_transfer::WallResistance;

let wall = WallResistance::new()
    .convective(1000.0)          // hot-side film, 1/h
    .conductive(0.002, 16.0)     // 2 mm steel wall, L/k
    .fouling(0.0002)             // fouling factor, m²·K/W
    .convective(500.0);          // cold-side film
let u = wall.overall_u(1.0);     // W/(m²·K) per unit area
```

## API tour

| Item | Purpose |
|---|---|
| `convection::dittus_boelter` | Turbulent internal Nu, heating/cooling |
| `convection::sieder_tate` | Turbulent internal Nu with viscosity ratio |
| `convection::internal_flow` | Regime-selecting Nu for tubes |
| `convection::flat_plate_*` | External flat-plate Nu (laminar, turbulent) |
| `convection::prandtl` / `reynolds_internal` | Dimensionless groups |
| `convection::heat_transfer_coefficient` | Nu → h conversion |
| `WallResistance` | Series R stack → overall U |
| `FlowRegime` | Regime/heating-cooling selector enum |
| `conduction_flux` | ΔT/R steady flux |

## Scope and limitations

- **Correlation accuracy** — each correlation is valid only inside its
  documented range (e.g. Dittus-Boelter 10⁴ < Re < 1.2·10⁵,
  0.7 < Pr < 120); nothing enforces those limits.
- **The transition band is not resolved** — `internal_flow` returns the
  Dittus-Boelter value between Re 2300 and 4000 with no guarantee; there is
  no Gnielinski implementation.
- **1-D, steady-state only** — no transient conduction, no radiation, no
  2-D effects, no fin calculations.
- SI units throughout; dimensionless groups must be fed consistent
  property evaluations (film-temperature property selection is the
  caller's responsibility).

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types.
- [`tpt-proc-heat-exchangers`](../tpt-proc-heat-exchangers) — uses
  `WallResistance` for fouling-inclusive overall coefficients.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
