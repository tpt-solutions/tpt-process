# tpt-proc-mass-balance

> Mass balance accounting and weighted least-squares data reconciliation.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-mass-balance.svg)](https://crates.io/crates/tpt-proc-mass-balance)
[![Documentation](https://docs.rs/tpt-proc-mass-balance/badge.svg)](https://docs.rs/tpt-proc-mass-balance)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The mass-balance bookkeeping crate of the [tpt-process](../../../README.md)
process-engineering suite: closure reports for balance nodes and a weighted
data-reconciliation adjustment that makes noisy plant measurements close the
balance exactly.

## Overview

Plant flow meters never tie out: a splitter with 100 kg/s measured in will
report 48 + 55 kg/s out. This crate provides the two standard responses.

First, accounting. [`overall_balance`] totals a node's inflows and outflows
and returns a [`BalanceReport`] with the residual and the relative closure
error; [`component_balance`] does the same per component across many nodes
(pass it a map of component name to `(inflows, outflows)` in kg/s of that
component).

Second, reconciliation. [`reconcile_flows`] performs the classic weighted
least-squares (data reconciliation) adjustment for one node: measurements
`(name, value, sigma)` are moved minimally in a χ² sense, weighted by
1/σ², so that the signed linear constraints `Σ signᵢ·xᵢ = 0` close exactly.
Precisely measured streams absorb less of the imbalance than loosely measured
ones — the closed-form projection moves each value by
`Δxᵢ = −signᵢ·(imbalance)/(ωᵢ·Σⱼ 1/ωⱼ)` and returns the reconciled values in
a [`Reconciliation`] with the post-adjustment residual.

Degenerate inputs (zero total weight, zero σ clamped to a large weight) never
panic: they return the raw measurements with the un-closed residual folded
into the report.

## Features

- [`overall_balance`] — one-node inflow/outflow totals with residual and
  relative closure error.
- [`BalanceReport`] — `inflow`, `outflow`, `residual`, `closure_error`
  (all kg/s except the dimensionless closure error).
- [`component_balance`] — per-component balance reports over multiple nodes
  from a `BTreeMap<String, (Vec<f64>, Vec<f64>)>`.
- [`reconcile_flows`] — closed-form weighted least-squares reconciliation of
  one node against signed linear constraints.
- [`Reconciliation`] — reconciled values by stream name plus post-fit
  residual and the total statistical weight used.
- Uncertainty-aware: measurements carry a standard deviation σ; σ ≤ 0 is
  treated as a very-large weight (near-exact measurement).
- Deterministic ordering (values keyed in a `BTreeMap`) and pure functions —
  no builders, no state.
- `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
tpt-proc-mass-balance = "0.1"
```

## Usage

```rust
use tpt_proc_mass_balance::{reconcile_flows, BalanceReport};

// A splitter node: measured 100 in, 48 + 55 out (103) — over-closed.
let report = reconcile_flows(&[("feed", 100.0, 2.0), ("p1", 48.0, 2.0), ("p2", 55.0, 2.0)],
                             &[("feed", 1.0), ("p1", -1.0), ("p2", -1.0)]);
assert!(report.residual.abs() < 1e-9);
```

Each constraint is `(stream name, sign)` and each measurement is
`(name, value, standard deviation)`.

## API tour

| Item | Purpose |
|---|---|
| [`overall_balance`] | Total a node's inflows/outflows; report residual and closure error. |
| [`BalanceReport`] | `inflow`, `outflow`, `residual`, `closure_error` for one node/component. |
| [`component_balance`] | One [`BalanceReport`] per component across multiple nodes. |
| [`reconcile_flows`] | Weighted least-squares closure of one node's measurements. |
| [`Reconciliation`] | Reconciled values, post-fit residual, total weight. |

## Scope and limitations

- Single-node reconciliation: the constraint set is one linear equation
  `Σ signᵢ·xᵢ = 0`; multi-node networks with shared measurements and full
  covariance matrices are out of scope.
- Flows only: no composition, temperature, or enthalpy reconciliation.
- All constraint streams must be measured (or default to 0.0 with weight 1.0);
  unmeasured-variable estimation is not provided.
- Units are whatever you pass in — the report fields are documented as kg/s
  but no conversion or validation is performed.
- The adjustment is linear (flow-weighted); no gross-error detection
  (outlier/leak tests) is implemented.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — the stream and unit types of
  the suite (declared companion dependency).
- [`tpt-proc-flowsheet`](../tpt-proc-flowsheet) — computes the converged
  flows this crate can audit and reconcile against plant data.
- No suite crate consumes `tpt-proc-mass-balance` yet.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
