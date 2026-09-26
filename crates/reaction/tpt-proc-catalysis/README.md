# tpt-proc-catalysis

> Porous-catalyst analysis: Thiele modulus, internal effectiveness factor, and deactivation models.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-catalysis.svg)](https://crates.io/crates/tpt-proc-catalysis)
[![Documentation](https://docs.rs/tpt-proc-catalysis/badge.svg)](https://docs.rs/tpt-proc-catalysis)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-catalysis` quantifies diffusion-reaction inside porous catalyst
pellets and the slow loss of catalyst activity over time, and is part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

The internal effectiveness factor η is the ratio of the actual reaction rate
in a pellet to the rate that would occur if the whole internal surface were
exposed to the external concentration. The crate implements the classical
Thiele-modulus analysis for a first-order reaction:

- `thiele_modulus` defines the modulus `φ = R·√(k/D_eff)` from pellet radius,
  rate constant, and effective diffusivity.
- `effectiveness_factor` evaluates the geometry-dependent closed forms —
  `η = tanh(φ)/φ` for an infinite slab and
  `η = (3/φ²)·(φ·coth φ − 1)` for a sphere — with the cylinder handled by a
  blend of the two. All geometries honor the large-φ asymptote
  `η → n/φ`, where the shape factor `n` is 1, 2, 3 for slab, cylinder, and
  sphere (`PelletGeometry::shape_factor`). Non-first-order reactions use the
  generalized-modulus approximation on the first-order curve.

The deactivation side models the activity `a(t) ∈ (0, 1]` by which rates must
be multiplied as the catalyst ages, with three empirical laws: exponential
`exp(−k_d·t)`, linear-in-time decay clamped at zero, and the slowest-decaying
power law `(1 + k_d·t)^(−1/2)`. Degenerate inputs never panic: a
non-positive Thiele modulus returns η = 1 (no internal limitation), and a
zero decay constant returns fresh-catalyst activity of 1.

## Features

- `PelletGeometry` enum (`Slab`, `Cylinder`, `Sphere`) with
  `shape_factor()` returning the asymptotic shape factor n = 1/2/3.
- `thiele_modulus(pellet_radius_m, rate_constant, effective_diffusivity_m2_s)`
  computes `φ = R·√(k/D_eff)` (infinite for non-positive diffusivity).
- `effectiveness_factor(thiele_modulus, geometry, order)` returns η ∈ (0, 1]
  with exact slab and sphere closed forms and a small-φ expansion
  (`1 − φ²/15` for spheres).
- Non-first-order reactions are approximated through the generalized modulus
  via `η₁^(2−order)` on the first-order curve.
- `DeactivationModel` enum: `Exponential`, `Linear`, `PowerLaw`.
- `deactivation_factor(k_d, time_s, model)` returns catalyst activity
  clamped to `[0, 1]`, hitting exactly zero for the linear model at
  `t = 1/k_d`.

## Installation

```toml
[dependencies]
tpt-proc-catalysis = "0.1"
```

## Usage

```rust
use tpt_proc_catalysis::{
    deactivation_factor, effectiveness_factor, DeactivationModel, PelletGeometry,
};

// First-order reaction in a sphere at Thiele modulus 3.
let eta = effectiveness_factor(3.0, PelletGeometry::Sphere, 1.0);
assert!((0.6..0.75).contains(&eta)); // diffusion-limited but active

// Exponential deactivation after 1 year of a 5%/month decay.
let activity = deactivation_factor(0.05, 12.0, DeactivationModel::Exponential);
assert!(activity < 0.6);
```

## API tour

| Item | Purpose |
|---|---|
| `PelletGeometry` | Pellet shape: slab, cylinder, or sphere |
| `PelletGeometry::shape_factor` | Asymptotic shape factor n for η → n/φ |
| `thiele_modulus` | Diffusion-reaction modulus φ = R·√(k/D_eff) |
| `effectiveness_factor` | Internal effectiveness factor η for a pellet geometry |
| `DeactivationModel` | Activity decay law: exponential, linear, or power law |
| `deactivation_factor` | Catalyst activity a(t) ∈ (0, 1] |

## Scope and limitations

- Exact closed forms exist for the first-order reaction only; non-first-order
  orders are handled with a generalized-modulus approximation, and the
  cylinder value is a slab/sphere blend rather than the exact Bessel-function
  ratio.
- Internal (intrapellet) diffusion only — there is no external film
  resistance, overall effectiveness factor, or Weisz-Prater criterion check.
- The pellet is isothermal: non-isothermal effectiveness (exothermic hot-spot
  inside the pellet) is not modeled.
- Single reaction on a single pellet; `deactivation_factor` is prescribed
  kinetics with a user-supplied decay constant and is not coupled to any
  reactor balance.

## Related crates

- Depends on [`tpt-proc-core`](../../core/tpt-proc-core) for core types.
- Companion reaction-engineering crates in the suite:
  [`tpt-proc-reaction`](../tpt-proc-reaction) (rate laws) and
  [`tpt-proc-reactors`](../tpt-proc-reactors) (CSTR/PFR models); the
  effectiveness factor from this crate would be applied to scale their
  intrinsic rates.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
