# tpt-proc-distillation

> Binary distillation column design: Fenske-Underwood-Gilliland shortcuts, McCabe-Thiele stepping, and constant-molal-overflow stage-to-stage simulation.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-distillation.svg)](https://crates.io/crates/tpt-proc-distillation)
[![Documentation](https://docs.rs/tpt-proc-distillation/badge.svg)](https://docs.rs/tpt-proc-distillation)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-distillation` covers the everyday distillation design toolbox — shortcut
stage and reflux estimates, graphical McCabe-Thiele stepping, and an
equilibrium-stage column simulation — and is part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

The crate implements the three workhorse methods of binary column design:

- **FUG shortcut** — Fenske minimum stages at total reflux, Underwood minimum
  reflux (binary, saturated-liquid feed, solved by bisection on θ), and the
  Gilliland correlation for actual stages, using the Seader-Henley exponential
  fit `Y = 1 − exp[(1+54.4X)/(11+117.2X)·(X−1)/√X]` with
  `X = (R − R_min)/(R + 1)`.
- **McCabe-Thiele** — stage-to-stage stepping between the equilibrium curve
  `y = αx/(1+(α−1)x)` and the rectifying/stripping operating lines for a
  saturated-liquid feed, returning the total stage count, feed stage position,
  and the minimum reflux from the pinch at the q-line intersection.
- **MESH simulation** — a constant-molal-overflow equilibrium-stage column
  solved by the Lewis-Sorel method: guess the top-liquid composition, step
  down the column through the section operating lines, and bisect on the
  guess until the stepped bottoms matches the overall material balance. The
  balance then closes exactly by construction.

A small thermo bridge (`benzene_toluene_package`,
`relative_volatility_from_package`, `package_k_values`) connects the column
methods to `tpt-proc-thermo-core` property packages, so constant-α inputs can
be generated from real Antoine-type vapor pressures instead of guessed.

## Features

- Full FUG shortcut via `fug_shortcut`, returning `FugResult` with
  `min_stages`, `min_reflux`, and `actual_stages` from a `FugSpec` of
  relative volatility, feed composition, key recoveries, and reflux ratio.
- Standalone `fenske_min_stages`, `underwood_min_reflux`, and
  `gilliland_stages` functions, with hard `Err` returns when `R ≤ R_min` or
  the Gilliland parameter `X` leaves `[0.01, 1]`.
- `mccabe_thiele_stages` produces `MccabeResult` with fractional
  equilibrium `stages` (reboiler included), 1-based `feed_stage`, and the
  pinch-based `min_reflux`.
- `simulate` runs an MESH column from a `ColumnSpec` (stages, feed stage,
  reflux ratio, distillate cut) to a `MeshResult` with distillate/bottoms
  compositions, the full top-to-bottom liquid profile, a `converged` flag,
  and the bisection iteration count.
- `benzene_toluene_package` builds a ready-to-use
  `tpt_proc_thermo_core::PropertyPackage` for column work.
- `relative_volatility_from_package` derives the constant α for the shortcut
  and stepping methods from pure-component vapor pressures (Raoult K-ratio).
- `package_k_values` fetches K-values from a package at `T` and `P`.
- Validation-first design: infeasible specifications (disordered
  compositions, sub-minimum reflux, bad stage counts) return descriptive
  errors rather than silent nonsense.

## Installation

```toml
[dependencies]
tpt-proc-distillation = "0.1"
```

## Usage

```rust
use tpt_proc_distillation::{FugSpec, fug_shortcut};

// Benzene/toluene at ~1.1 atm with α ≈ 2.4, recover 95% LK, 95% HK.
let spec = FugSpec {
    relative_volatility: 2.4,
    feed_mole_fraction: 0.5,
    light_key_recovery: 0.95,
    heavy_key_recovery: 0.05,
    actual_reflux_ratio: 1.5,
};
let result = fug_shortcut(&spec).unwrap();
assert!(result.min_stages > 3.0);
assert!(result.min_reflux < spec.actual_reflux_ratio);
assert!(result.actual_stages > result.min_stages);
```

## API tour

| Item | Purpose |
|---|---|
| `FugSpec` | Shortcut inputs: constant α, feed composition, key recoveries, reflux ratio |
| `FugResult` | `min_stages` (Fenske), `min_reflux` (Underwood), `actual_stages` (Gilliland) |
| `fenske_min_stages` | Minimum stages at total reflux from the key-recovery ratio |
| `underwood_min_reflux` | Binary Underwood R_min at saturated-liquid feed (bisected θ) |
| `gilliland_stages` | Seader-Henley exponential Gilliland correlation |
| `fug_shortcut` | Composes the three into one `Result<FugResult, String>` |
| `mccabe_thiele_stages` | Graphical stepping; returns `MccabeResult` |
| `MccabeResult` | Fractional stages, 1-based feed stage, pinch `min_reflux` |
| `ColumnSpec` / `MeshResult` | MESH column specification and converged profile |
| `simulate` | Lewis-Sorel stage-to-stage MESH simulation |
| `benzene_toluene_package` | Convenience `PropertyPackage` for benzene/toluene |
| `relative_volatility_from_package` / `package_k_values` | Thermo bridges to the property package |

## Scope and limitations

- Binary (two-key) basis with constant relative volatility; the FUG and
  McCabe-Thiele methods assume a saturated-liquid feed (q = 1).
- No energy balances: constant molal overflow throughout; no condenser/reboiler
  duties, no pressure profile.
- No column hydraulics — tray efficiency, flooding, weeping, and diameter
  sizing are out of scope; all stages are ideal (total condenser assumed).
- The Gilliland fit is valid for `0.01 ≤ X ≤ 1`, and the Underwood
  implementation is the binary form only; multicomponent key systems are not
  handled.
- K-values come from Raoult-law pure vapor pressures via the property package;
  activity-coefficient models for non-ideal VLE are not wired in here.

## Related crates

- Depends on [`tpt-proc-core`](../../core/tpt-proc-core) for stream
  compositions and [`tpt-proc-thermo-core`](../../thermodynamics/tpt-proc-thermo-core)
  / [`tpt-proc-thermo-database`](../../thermodynamics/tpt-proc-thermo-database)
  for components and property packages.
- Exercised by the `binary-distillation` worked example and the
  `distillation-mesh` benchmark in the workspace.
- Sits in the separations domain alongside `tpt-proc-absorption`,
  `tpt-proc-extraction`, `tpt-proc-membranes`, and `tpt-proc-crystallization`.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
