# tpt-proc-pharma

> Pharmaceutical batch process models: cycle accounting, dissolution, shelf life.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-pharma.svg)](https://crates.io/crates/tpt-proc-pharma)
[![Documentation](https://docs.rs/tpt-proc-pharma/badge.svg)](https://docs.rs/tpt-proc-pharma)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The pharmaceutical-manufacturing crate of the
[tpt-process](../../../README.md) process-engineering suite: batch cycle time
accounting, Noyes–Whitney dissolution kinetics, and Arrhenius degradation
with USP-limit shelf-life estimation.

## Overview

Three small models cover the batch-oriented questions pharma process work
asks first.

**Cycle time.** [`BatchCycle`] is a named-step schedule: `add_step` chains
`(name, duration)` entries (charge, react, discharge, …), `total_duration`
sums them, and `utilization` reports total duration against a plant time
budget (e.g. a 48 h campaign window).

**Dissolution.** [`dissolution_fraction`] integrates the Noyes–Whitney
first-order model, f = 1 − exp(−k·t), for a rate constant k in h⁻¹ — the
shape behind a typical dissolution curve (e.g. k = 0.35 h⁻¹ reaches ~95% in
8.5 h).

**Stability.** [`potency_after_storage`] applies Arrhenius temperature
dependence to a reference first-order degradation constant
(`k_ref_per_day` at `reference_k`, activation energy in J/mol) and returns
the remaining potency fraction after `days` at `temperature_k`.
[`shelf_life_days`] inverts the reference-conditions decay to the USP
t₉₅ limit — the storage time until potency drops to 95%.

All functions are pure, `f64`-based, and clamp negative time/rate inputs;
the crate is `#![forbid(unsafe_code)]`.

## Features

- [`BatchCycle`] — named batch-step schedule (`steps` is public:
  `Vec<(String, f64)>`, durations in h).
- [`BatchCycle::add_step`] — builder-style chaining (`&mut Self` returns).
- [`BatchCycle::total_duration`] / [`BatchCycle::utilization`] — cycle time
  and plant-budget utilization.
- [`dissolution_fraction`] — first-order (integrated Noyes–Whitney)
  dissolved fraction, f = 1 − exp(−k·t).
- [`potency_after_storage`] — Arrhenius-adjusted first-order degradation;
  potency fraction after storage at an arbitrary temperature.
- [`shelf_life_days`] — t₉₅ shelf life (days) at reference conditions per
  the USP 95%-potency limit.
- Negative inputs clamped (no NaNs from bad data); stable molecule with
  k → 0 reports an infinite shelf life.
- `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
tpt-proc-pharma = "0.1"
```

## Usage

```rust
use tpt_proc_pharma::{BatchCycle, dissolution_fraction};

// Tablet dissolving with k = 0.35 h⁻¹: ~95% in 8.5 h.
let f = dissolution_fraction(0.35, 8.5);
assert!(f > 0.9 && f < 1.0);

let mut cycle = BatchCycle::new(48.0);
let total = cycle
    .add_step("charge", 2.0)
    .add_step("react", 24.0)
    .add_step("discharge", 4.0)
    .total_duration();
assert!((total - 30.0).abs() < 1e-12);
```

## API tour

| Item | Purpose |
|---|---|
| [`BatchCycle`] | Named-step batch schedule; `total_duration`, `utilization`. |
| `add_step` | Append a `(name, hours)` step, chainable. |
| [`dissolution_fraction`] | Dissolved fraction by first-order Noyes–Whitney kinetics. |
| [`potency_after_storage`] | Potency fraction after storage (Arrhenius + 1st order). |
| [`shelf_life_days`] | Days until potency hits the USP 95% limit at reference conditions. |

## Scope and limitations

- First-order kinetics only: dissolution is a single exponential (no
  Weibull/Hixson–Crowell, no matrix diffusion), and stability is a single
  Arrhenius pathway (no humidity, light, or autocatalysis).
- [`shelf_life_days`] evaluates reference conditions only — use
  [`potency_after_storage`] for accelerated-temperature scenarios.
- [`BatchCycle`] is time accounting only: no GMP logic, cleaning
  validation, changeover rules, equipment sharing, or scheduling of
  parallel batches.
- No formulation, unit-operation, or bioreactor models; doses and
  concentrations are not modeled.
- Hours and days as stated per function; SI (kelvin, J/mol) for the
  Arrhenius parameters.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — the stream and unit types of
  the suite (declared companion dependency).
- [`tpt-proc-reaction`](../../reaction/tpt-proc-reaction) — the suite's
  general kinetics crate (Arrhenius, Michaelis-Menten, LHHW); this crate
  keeps its own small stability model for standalone use.
- `tpt-medical` (in the counterpart ecosystem repository) — the planned
  consumer of these batch models, per the root README.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
