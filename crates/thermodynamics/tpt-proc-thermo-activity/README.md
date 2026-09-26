# tpt-proc-thermo-activity

> Activity coefficient models for the liquid phase: Wilson, NRTL, UNIQUAC, and UNIFAC.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-thermo-activity.svg)](https://crates.io/crates/tpt-proc-thermo-activity)
[![Documentation](https://docs.rs/tpt-proc-thermo-activity/badge.svg)](https://docs.rs/tpt-proc-thermo-activity)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Local-composition and group-contribution excess-Gibbs models that drive the
modified-Raoult K-value path `Kᵢ = γᵢ·P_sat,ᵢ/P`. Part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

All models implement
[`ActivityCoefficientModel`](https://docs.rs/tpt-proc-thermo-core) and can be
attached to a `PropertyPackage` (via `with_activity`) so the flash solver and
distillation columns see liquid non-ideality:

- [`Nrtl`] — electrolyte-free NRTL with temperature-independent τ and
  non-randomness parameters α (Renon & Prausnitz non-random two-liquid model).
- [`Wilson`] — Wilson's local-composition model (1964); miscible systems only.
- [`Uniquac`] — UNIQUAC combinatorial + residual terms (Abrams & Prausnitz,
  1975).
- [`Unifac`] — UNIFAC group-contribution estimation (Fredenslund, Jones &
  Prausnitz, 1975): the UNIQUAC combinatorial term over molecular r/q built
  from subgroup parameters, plus a main-group residual term.

Pair parameters are stored by component *index* in the package's component
list, using the shared [`PairMap`] type. A helper
[`excess_gibbs_over_rt`] computes `Σ xᵢ ln γᵢ` from any model's output —
useful for mixing-rule coupling (RFC 0001) and as a test invariant.

## Features

- [`Nrtl::from_params`] with [`NrtlParams::binary`]`(alpha, tau_01,
  tau_10)` and [`NrtlParams::set`] for arbitrary pairs.
- [`Wilson::from_params`] with [`WilsonParams::binary`].
- [`Uniquac::from_params`] with [`UniquacParams::binary`].
- [`Unifac::new`] over a [`UnifacTable`] and per-component [`GroupCount`]s;
  [`UnifacTable::builtin_subset`] ships the classic groups for
  hydrocarbon/aromatic/alcohol/water systems (main groups CH2, ACH, OH,
  CH3OH, H2O with the standard 1975 interaction values), and
  `Unifac::with_table` accepts any fuller table — the math is
  table-agnostic.
- [`GroupCount::from_pairs`] — subgroup id → count map per component.
- [`excess_gibbs_over_rt`] — G^E/RT from ln γ, shared invariant across
  models.
- Composition validation (finite, non-negative, sums to 1) on every
  evaluation, returning [`ThermoError`](https://docs.rs/tpt-proc-thermo-core)
  rather than panicking.
- All models are `Send + Sync` and `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
tpt-proc-thermo-activity = "0.1"
```

## Usage

```rust
use tpt_proc_thermo_activity::{Nrtl, NrtlParams};
use tpt_proc_thermo_core::ActivityCoefficientModel;

// Ethanol(0)/water(1), illustrative NRTL parameters (J/mol).
let nrtl = Nrtl::from_params(2, NrtlParams::binary(0.30, 728.0, 385.0));
let gamma = nrtl.gamma(&[0.5, 0.5], 351.45).unwrap();

// Non-ideal but finite; both gammas are plausible magnitudes.
assert!(gamma[0] > 1.0 && gamma[0] < 3.0);
assert!(gamma[1] > 0.8 && gamma[1] < 3.0);
```

## API tour

| Item | Purpose |
|---|---|
| `Nrtl` / `NrtlParams` | Non-random two-liquid model with per-pair α and τ |
| `Wilson` / `WilsonParams` | Wilson local-composition model (miscible systems) |
| `Uniquac` / `UniquacParams` | UNIQUAC combinatorial + residual model |
| `Unifac` / `UnifacTable` / `GroupCount` | Group-contribution γ from subgroup counts and interaction tables |
| `PairMap` | `BTreeMap<(usize, usize), f64>` of index-keyed pair parameters |
| `excess_gibbs_over_rt` | Σ xᵢ ln γᵢ helper shared by all models |

## Scope and limitations

- Parameters are **not** bundled: only the small built-in UNIFAC subgroup
  table exists, and it is a *demonstration subset* (CH2, ACH, OH, CH3OH,
  H2O). Production use requires loading published parameter tables via
  `Unifac::with_table` / the params constructors.
- NRTL τ and α are treated as temperature-independent in this
  implementation; temperature-dependent τ correlations are not provided.
- Wilson is valid for miscible systems only (it cannot represent liquid-liquid
  splitting).
- No electrolyte models (eNRTL, Pitzer), no modified-Dortmund UNIFAC
  variant, and no liquid-liquid equilibrium solving here — this crate
  computes γ only.
- Binary data must be entered by component index; there is no name-keyed
  parameter store (see `tpt-proc-thermo-database` for name-keyed data).

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — upstream: `Composition`
  conventions.
- [`tpt-proc-thermo-core`](../tpt-proc-thermo-core) — upstream: the
  `ActivityCoefficientModel` trait and `ThermoError`.
- [`tpt-proc-thermo-phase`](../tpt-proc-thermo-phase) — downstream: uses
  attached activity models in the modified-Raoult K-value path.
- [`tpt-proc-distillation`](../../separations/tpt-proc-distillation) —
  downstream consumer of activity-corrected K-values.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
