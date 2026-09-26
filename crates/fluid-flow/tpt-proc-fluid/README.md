# tpt-proc-fluid

> Incompressible pipe-flow hydraulics: Reynolds number, Darcy friction factor, and Darcy-Weisbach pressure drop.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-fluid.svg)](https://crates.io/crates/tpt-proc-fluid)
[![Documentation](https://docs.rs/tpt-proc-fluid/badge.svg)](https://docs.rs/tpt-proc-fluid)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-fluid` is the hydraulic foundation of the fluid-flow domain in the
[tpt-process](../../../README.md) process-engineering suite: single-phase,
incompressible flow in circular pipes with the classic Darcy-Weisbach
formulation, in pure Rust with no dependencies beyond `tpt-proc-core`.

## Overview

Every steady-state hydraulics calculation in the suite bottoms out here. The
crate models a circular pipe (`PipeFlow`) carrying an incompressible fluid
(`FluidProperties`: density and dynamic viscosity) and answers the two
questions every engineer asks of a line: what is the regime (`reynolds_number`)
and what does it cost (`pressure_drop`, `minor_losses`,
`total_pressure_drop`).

The Darcy friction factor follows the standard Moody-chart formulation. In the
laminar regime (Re < 2300) it is exactly `f = 64/Re`, matching
Hagen-Poiseuille. In the turbulent regime the crate solves the implicit
Colebrook-White equation

```text
1/√f = −2·log10( ε/(3.7·D) + 2.51/(Re·√f) )
```

by Newton iteration, seeded with the explicit Swamee-Jain approximation —
convergence to machine precision in roughly five iterations. Both the exact
solver (`friction_factor`) and the explicit approximation (`swamee_jain`) are
exposed as free functions.

Note that the factor returned is the **Darcy** factor (`f = 64/Re` laminar),
not the Fanning factor; divide by 4 to convert.

## Features

- `PipeFlow` — a circular pipe (diameter, length, roughness) with a fluid;
  builder-style `with_fluid` (defaults to water at 20 °C).
- `reynolds_number` — regime classification from `Re = ρvD/μ`.
- `friction_factor` — Darcy factor: laminar `64/Re`, turbulent
  Colebrook-White by Newton iteration.
- `swamee_jain` — the explicit Swamee-Jain friction-factor approximation.
- `pressure_drop` / `pressure_drop_at_flow` — Darcy-Weisbach
  `ΔP = f·(L/D)·ρv²/2` by velocity or volumetric flow.
- `minor_losses` / `total_pressure_drop` — ΣK loss terms on top of friction.
- `area`, `velocity`, `flow_rate` — conversions between velocity and
  volumetric flow through the pipe cross-section.
- Free function `friction_factor(reynolds, relative_roughness)` usable
  standalone, without building a pipe.

## Installation

```toml
[dependencies]
tpt-proc-fluid = "0.1"
```

## Usage

```rust
use tpt_proc_fluid::{FluidProperties, PipeFlow};

// Water in a 100 m commercial-steel pipe, 0.05 m diameter.
let pipe = PipeFlow::new(0.05, 100.0, 4.6e-5)
    .with_fluid(FluidProperties { density: 998.0, viscosity: 1.0e-3 });

let v = 1.0; // m/s
assert!(pipe.reynolds_number(v) > 4000.0); // turbulent
let dp = pipe.pressure_drop(v);
assert!(dp > 0.0);
```

## API tour

| Item | Purpose |
|---|---|
| `FluidProperties` | Density and dynamic viscosity of the fluid |
| `PipeFlow` | Circular pipe + fluid; all flow calculations hang off it |
| `PipeFlow::friction_factor` | Darcy friction factor at a mean velocity |
| `PipeFlow::pressure_drop` | Frictional pressure drop (Darcy-Weisbach), Pa |
| `PipeFlow::minor_losses` | Fitting/valve losses from a total K factor, Pa |
| `friction_factor(re, eps/D)` | Standalone Colebrook-White / laminar solver |
| `swamee_jain(re, eps/D)` | Explicit friction-factor approximation |

## Scope and limitations

- **Incompressible single-phase flow only** — no compressible gas flow, no
  two-phase flow; density and viscosity are constants.
- **Circular pipes only** — no non-circular ducts or annuli.
- **No elevation term** — `pressure_drop` covers friction plus minor losses;
  static (geodetic) head is the caller's problem.
- **No fittings database** — minor losses take a user-supplied ΣK; the crate
  does not tabulate K values.
- **Sharp laminar/turbulent switch** at Re = 2300 (no transition-band
  interpolation); SI units throughout; Darcy (not Fanning) factor.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types.
- [`tpt-proc-pumps`](../tpt-proc-pumps) — pump curves and operating points
  build on this hydraulics layer.
- [`tpt-proc-compressors`](../tpt-proc-compressors) and
  [`tpt-proc-valves`](../tpt-proc-valves) — consume the same flow
  vocabulary for gas compression and Cv/Kv sizing.
- [`tpt-proc-network`](../tpt-proc-network) — solves whole pipe networks
  whose pipe resistances derive from these friction factors.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
