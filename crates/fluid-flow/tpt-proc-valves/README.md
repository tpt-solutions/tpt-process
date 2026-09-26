# tpt-proc-valves

> Control-valve sizing: Cv/Kv conversions, liquid and gas flow, inherent characteristics.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-valves.svg)](https://crates.io/crates/tpt-proc-valves)
[![Documentation](https://docs.rs/tpt-proc-valves/badge.svg)](https://docs.rs/tpt-proc-valves)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-valves` sizes control valves and computes installed flow through
them — part of the [tpt-process](../../../README.md) process-engineering suite.
It speaks the flow-coefficient dialect of the valve industry: Cv in US units
(gpm water at 1 psi ΔP), Kv in metric (m³/h water at 1 bar ΔP), pressures in
Pa inside the API.

## Overview

A `ControlValve` carries a full-open `cv_max` and an inherent characteristic
(`ValveCharacteristic`): `Linear` (Cv ∝ opening), `EqualPercentage`
(Cv = Cv_max·R^(x−1) with rangeability R = 50), or `QuickOpening`
(Cv/Cv_max = 1 − (1−x)², high gain near closed). `cv_at` gives the effective
coefficient at any opening x ∈ [0, 1].

Liquid sizing uses the ISA-style simplified equation with a choking
(recovery) factor: `Q = Cv·√(ΔP_psi/(SG·(1 − F_ch)))`, returned in m³/s.
The inverse problem — what Cv does this service need? — is the associated
function `required_cv_liquid`. Subcritical gas sizing uses the classic Cv
equation `Q_scfh = 1360·Cv·√((ΔP·P₂)/(SG·T))` (P in psia, T in °R),
converted to kg/s; reverse ΔP returns zero flow rather than NaN.

`cv_to_kv` / `kv_to_cv` convert between the two coefficient conventions
(Kv = Cv·0.865).

## Features

- `ControlValve` — a valve defined by `cv_max` and its inherent
  characteristic; `ControlValve::new` constructor.
- `ControlValve::cv_at` — effective Cv at an opening x ∈ [0, 1].
- `ValveCharacteristic` — `Linear`, `EqualPercentage` (R = 50),
  `QuickOpening` inherent characteristics.
- `ControlValve::liquid_flow_m3s` — liquid flow with choking factor for
  near-cavitation service.
- `ControlValve::required_cv_liquid` — Cv required for a liquid duty.
- `ControlValve::gas_flow_kgs` — subcritical gas flow in kg/s.
- `cv_to_kv` / `kv_to_cv` — coefficient conversions (0.865 factor).

## Installation

```toml
[dependencies]
tpt-proc-valves = "0.1"
```

## Usage

```rust
use tpt_proc_valves::{ControlValve, ValveCharacteristic};

// 50 Cv globe valve with an equal-percentage trim.
let valve = ControlValve::new(50.0, ValveCharacteristic::EqualPercentage);

// Effective Cv at 60 % opening.
let cv_open = valve.cv_at(0.6);

// Liquid flow at 100 kPa drop, water (SG 1.0), no choking.
let q_m3s = valve.liquid_flow_m3s(0.6, 100.0e3, 1.0, 0.0);

// Required Cv to pass 10 gpm-equivalent water at 1 psi drop → 10.
let cv_needed = ControlValve::required_cv_liquid(10.0 * 6.30902e-5, 6894.76, 1.0);
assert!((cv_needed - 10.0).abs() < 1e-9);

// Metric coefficient for the datasheet.
let kv = tpt_proc_valves::cv_to_kv(50.0); // 43.25
```

## API tour

| Item | Purpose |
|---|---|
| `ControlValve` | Valve with `cv_max` + characteristic |
| `ValveCharacteristic` | Linear / equal-percentage / quick-opening trim |
| `ControlValve::cv_at` | Installed Cv at a given opening |
| `ControlValve::liquid_flow_m3s` | Liquid capacity with choking factor |
| `ControlValve::required_cv_liquid` | Inverse liquid sizing (Cv from duty) |
| `ControlValve::gas_flow_kgs` | Subcritical gas capacity, kg/s |
| `cv_to_kv`, `kv_to_cv` | Cv ↔ Kv conversions |

## Scope and limitations

- **Simplified sizing equations** — the ISA simplified liquid equation with
  a single choking factor, and the classic subcritical gas equation; no
  full ISA/IEC 60534 expansion-factor, piping-geometry-factor, or
  cavitation-index formalism.
- **Choked gas flow not modeled** — `gas_flow_kgs` is subcritical only and
  returns 0 for reverse ΔP; it does not predict critical-flow choking.
- **Inherent characteristics only** — no installed-characteristic
  distortion from process gain, no actuator dynamics, no noise or erosion
  estimates.
- Single equal-percentage rangeability (R = 50) is hard-coded.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types.
- [`tpt-proc-fluid`](../tpt-proc-fluid) — pipe hydraulics for the pressure
  drops feeding the valve.
- [`tpt-proc-control`](../../dynamics/tpt-proc-control) — PID control and
  tuning downstream of the valve model.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
