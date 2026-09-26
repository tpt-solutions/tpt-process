# tpt-proc-refining

> Refining stream characterization: API gravity, Watson K, crude assay cuts.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-refining.svg)](https://crates.io/crates/tpt-proc-refining)
[![Documentation](https://docs.rs/tpt-proc-refining/badge.svg)](https://docs.rs/tpt-proc-refining)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The refining-characterization crate of the
[tpt-process](../../../README.md) process-engineering suite: petroleum stream
gravity conversions, Watson characterization factors, boiling-point cut
blending, and a simplified crude-assay structure.

## Overview

Refining work starts from a handful of classical characterizing numbers, and
this crate implements them exactly as the industry defines them.

**Gravity.** [`api_to_specific_gravity`] and
[`specific_gravity_to_api`] convert between API gravity and specific
gravity (60 °F basis) via the standard relation
sg = 141.5/(API + 131.5) — water is 10° API, a 40° API light stream is
sg ≈ 0.8251.

**Watson K.** [`watson_k_factor`] computes the UOP characterization factor
K = (Tb_R)^(1/3)/sg from the mean average boiling point (MeABP) in degrees
Rankine; the docs note the Kelvin form K = 1.216·Tb_K^(1/3)/sg. K classifies
the stream's hydrogen content: paraffinic ≈ 12.5+, naphthenic 11–12,
aromatic ≈ 10.5.

**Crude assay.** A [`CrudeAssay`] is a named list of [`Cut`]s, each with an
initial/final boiling point (°F) and a mass fraction. [`CrudeAssay::total_yield`]
checks normalization, [`CrudeAssay::weighted_api`] mass-weights per-cut API
gravities (missing entries default to 35° API), and
[`CrudeAssay::yield_below`] blends the yield of every cut whose final
boiling point is at or below a ceiling — the gasoline- or naphtha-pool
yield, for example. [`light_crude_assay`] provides a reference
light-crude shape (seven cuts, butanes-plus through vacuum resid, ~40° API
class).

## Features

- [`api_to_specific_gravity`] / [`specific_gravity_to_api`] — standard
  API ↔ specific gravity conversions (60 °F basis).
- [`watson_k_factor`] — UOP characterization factor from MeABP (°R) and
  specific gravity; returns NaN for non-physical inputs.
- [`Cut`] — a boiling-point cut: name, initial/final boiling point (°F),
  mass fraction.
- [`CrudeAssay`] — a named cut list with `CrudeAssay::new`,
  `total_yield`, `weighted_api`, and `yield_below`.
- [`CrudeAssay::yield_below`] — product-pool yield blending by final
  boiling point.
- [`light_crude_assay`] — an illustrative reference light-crude assay
  (cuts sum to 1.0).
- Pure functions and plain data structures; `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
tpt-proc-refining = "0.1"
```

## Usage

```rust
use tpt_proc_refining::{api_to_specific_gravity, watson_k_factor};

// A 40° API light stream.
let sg = api_to_specific_gravity(40.0);
assert!((sg - 0.8251).abs() < 1e-3);

// Watson characterization factor from MeABP (avg boiling point).
let k = watson_k_factor(650.0, sg); // °R basis
assert!((10.0..=13.0).contains(&k));
```

## API tour

| Item | Purpose |
|---|---|
| [`api_to_specific_gravity`] | API gravity → specific gravity (141.5/(API + 131.5)). |
| [`specific_gravity_to_api`] | Specific gravity → API gravity. |
| [`watson_k_factor`] | Watson/UOP K from MeABP (°R) and sg. |
| [`Cut`] | One boiling-range cut with a mass fraction. |
| [`CrudeAssay`] | Named cut list; `total_yield`, `weighted_api`, `yield_below`. |
| [`light_crude_assay`] | Reference light-crude assay shape (~40° API class). |

## Scope and limitations

- Simplified characterization: no ASTM D86/TBP distillation-curve fitting,
  no pseudocomponent generation, and no property prediction (viscosity,
  flash point, cetane/octane) from the assay.
- Cuts carry **mass** fractions, while published assays are usually reported
  in volume percent — convert before comparing.
- [`light_crude_assay`] is explicitly illustrative, not a real crude's
  certified assay data.
- [`CrudeAssay::weighted_api`] pairs gravities by index and silently
  defaults missing entries to 35° API.
- Mixed legacy units by design (API convention): °F boiling points, °R/K
  MeABP, 60 °F gravities — no SI-only interface.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — the stream and unit types of
  the suite (declared companion dependency).
- [`tpt-proc-distillation`](../../separations/tpt-proc-distillation) — the
  suite's column models; assay cuts define the feeds such columns separate.
- No suite crate consumes `tpt-proc-refining` yet.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
