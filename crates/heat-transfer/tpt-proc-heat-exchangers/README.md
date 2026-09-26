# tpt-proc-heat-exchangers

> Heat-exchanger rating and sizing: LMTD with correction factors and the NTU-effectiveness method.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-heat-exchangers.svg)](https://crates.io/crates/tpt-proc-heat-exchangers)
[![Documentation](https://docs.rs/tpt-proc-heat-exchangers/badge.svg)](https://docs.rs/tpt-proc-heat-exchangers)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-heat-exchangers` rates and sizes heat exchangers — duty, outlet
temperatures, effectiveness, and required area — following Incropera's
*Fundamentals of Heat and Mass Transfer*. Part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

Two complementary classical methods are implemented on one `HeatExchanger`
type (area, clean overall coefficient, flow arrangement):

- **LMTD method** — `lmtd` computes the log-mean temperature difference
  (identical terminal differences collapse to their common value, avoiding
  0/0), `correction_factor` returns the F factor for the arrangement
  (1.0 for counter-current and co-current as paired by the caller; a Bowman
  chart fit for `ShellAndTube1ShellPass`, i.e. 1 shell pass / 2+ tube
  passes), and `duty_from_lmtd` gives `Q = U·A·LMTD·F`. The inverse —
  `required_area` — sizes the surface for a specified duty.
- **ε-NTU method** — `effectiveness` evaluates ε from `NTU = UA/C_min` and
  the capacity ratio `Cr` for each arrangement, and `rate` finds duty and
  both outlet temperatures from inlet states. Energy balance closes
  exactly: the hot side loses what the cold side gains, and duty never
  exceeds the thermodynamic maximum.

`overall_from_resistances` builds a fouling-inclusive U from the series
resistance stack of `tpt-proc-heat-transfer::WallResistance`. Test
tolerances are absolute against hand-computed values.

## Features

- `HeatExchanger` — area (m²), overall U (W/(m²·K)), and flow arrangement.
- `HeatExchanger::lmtd` — log-mean temperature difference in K.
- `HeatExchanger::correction_factor` — F for counter-current, co-current,
  or 1-shell-pass shell-and-tube (Bowman correlation).
- `HeatExchanger::duty_from_lmtd` — Q = U·A·LMTD·F.
- `HeatExchanger::required_area` — sizing inverse of the LMTD method.
- `HeatExchanger::effectiveness` — ε from the NTU method per arrangement.
- `HeatExchanger::rate` — full rating: duty plus hot/cold outlets.
- `HeatExchanger::overall_from_resistances` / `ua` — fouling-aware U and
  the UA product.
- `FlowConfiguration` — `CounterCurrent`, `CoCurrent`,
  `ShellAndTube1ShellPass`.
- `RatingResult` — duty, both outlets, effectiveness.

## Installation

```toml
[dependencies]
tpt-proc-heat-exchangers = "0.1"
```

## Usage

```rust
use tpt_proc_heat_exchangers::{FlowConfiguration, HeatExchanger};

let hx = HeatExchanger::new(25.0, 900.0, FlowConfiguration::CounterCurrent);

// LMTD of counter-current 100→30 / 20→50 °C service.
let lmtd = hx.lmtd(373.15, 303.15, 293.15, 323.15);
let expected = 40.0 / 5.0f64.ln(); // K, terminal differences 50 & 10
assert!((lmtd - expected).abs() < 1e-6);

// Rating: duty and outlet temperatures from UA and inlet states.
let rated = hx.rate(2.0e3, 4.0e3, 3.0e3, 330.0, 290.0);
assert!(rated.duty > 0.0);
assert!(rated.hot_outlet < 330.0 && rated.cold_outlet > 290.0);
```

## API tour

| Item | Purpose |
|---|---|
| `HeatExchanger` | The exchanger: A, U, arrangement |
| `HeatExchanger::lmtd` | Log-mean temperature difference |
| `HeatExchanger::correction_factor` | F factor per flow configuration |
| `HeatExchanger::duty_from_lmtd` | Duty via the LMTD method |
| `HeatExchanger::required_area` | Area for a specified duty |
| `HeatExchanger::effectiveness` | ε-NTU effectiveness |
| `HeatExchanger::rate` | Rating: duty + outlet temperatures |
| `FlowConfiguration` | Counter-current / co-current / 1-shell-pass |
| `RatingResult` | Duty, outlets, effectiveness |

## Scope and limitations

- **Constant capacity rates** — the ε-NTU method assumes cp independent of
  temperature; no phase change (condensers/reboilers), no variable-property
  integration.
- **Three flow arrangements only** — no multi-shell-pass (F ≈ 0.8 style)
  charts beyond the 1-shell-pass Bowman fit, no cross-flow or plate
  arrangements, no TEMA geometry or vibration/pressure-drop detail.
- **Fixed UA** — the rating treats U as a clean, uniform coefficient;
  the `duty_cap` parameter of `rate` is currently ignored by the model.
- Core (tube-side/shell-side) pressure drop is not modeled; temperatures
  in K, duties in W, SI throughout.

## Related crates

- [`tpt-proc-heat-transfer`](../tpt-proc-heat-transfer) — film coefficients
  and `WallResistance` fouling stacks feed `overall_from_resistances`.
- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types.
- [`tpt-proc-wasm`](../../core/tpt-proc-wasm) — exposes exchanger rating to
  WebAssembly.
- `examples/heat-exchanger-rating` and `benches/heat-exchanger-rating` in
  the repository root exercise this crate.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
