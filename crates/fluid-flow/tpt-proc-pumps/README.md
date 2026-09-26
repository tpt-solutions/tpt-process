# tpt-proc-pumps

> Centrifugal pump modeling: quadratic curves, operating points, affinity laws, and NPSH.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-pumps.svg)](https://crates.io/crates/tpt-proc-pumps)
[![Documentation](https://docs.rs/tpt-proc-pumps/badge.svg)](https://docs.rs/tpt-proc-pumps)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-pumps` models centrifugal pumps — head/flow curves, the operating
point against a system curve, speed scaling by the affinity laws, shaft
power, and NPSH — as part of the [tpt-process](../../../README.md)
process-engineering suite. Heads are in m of fluid, flows in m³/s, power in W.

## Overview

The crate keeps the classic textbook pump model: a quadratic pump curve
`H(Q) = h₀ + a·Q + b·Q²` (`PumpCurve`), a quadratic system curve
`H_sys(Q) = H_static + k·Q²` (`SystemCurve`), and the pump as a machine at a
fixed shaft speed with a hydraulic-efficiency model (`Pump`). The efficiency
model is a quadratic roll-off from the peak (best-efficiency-point) value,
clamped to a small positive floor so power never divides by zero.

The operating point is where the two curves meet. `Pump::operating_point`
finds it by bisection on the residual `H_pump − H_sys` (which decreases in Q
for normal drooping curves) and returns `None` when the pump can never
overcome the static head — the physical "no flow" answer rather than a
converged nonsense number.

Speed changes go through the affinity laws: `PumpCurve::affinity_scaled`
takes a speed ratio N₂/N₁ and scales head with ratio² and flow with ratio.

## Features

- `PumpCurve` — quadratic head/flow curve with `head_at_flow`.
- `PumpCurve::affinity_scaled` — affinity-law scaling for a speed ratio.
- `SystemCurve` — static head plus friction term, `head_at_flow`.
- `Pump` — curve + shaft speed (`speed_rpm`) + efficiency model
  (`bep_flow`, `peak_efficiency`).
- `Pump::operating_point` — bisection intersection of pump and system
  curves; `None` when the pump cannot overcome static head.
- `Pump::power_at_flow` — shaft power `P = ρgQH/η` in W.
- `Pump::efficiency_at_flow` — hydraulic efficiency clamped to (0, 1].
- `Pump::npsh_available` — NPSH in m from surface pressure, vapor
  pressure, static suction head, and suction losses.

## Installation

```toml
[dependencies]
tpt-proc-pumps = "0.1"
```

## Usage

```rust
use tpt_proc_pumps::{Pump, PumpCurve, SystemCurve};

// H = 30 − 8000·Q² at 1750 rpm, BEP at 0.03 m³/s, 72 % peak efficiency.
let pump = Pump {
    curve: PumpCurve { shutoff_head: 30.0, a: 0.0, b: -8000.0 },
    speed_rpm: 1750.0,
    bep_flow: 0.03,
    peak_efficiency: 0.72,
};

// 10 m of static lift plus friction: find the duty point.
let system = SystemCurve { static_head: 10.0, k: 8000.0 };
let (flow, head) = pump.operating_point(&system, 0.2).unwrap();

let power = pump.power_at_flow(flow, 998.0);          // W, water
let npsh_a = Pump::npsh_available(101_325.0, 2340.0, 2.0, 0.5, 998.0);

// Half speed: Q halves, shutoff head quarters.
let slow = pump.curve.affinity_scaled(0.5);
```

## API tour

| Item | Purpose |
|---|---|
| `PumpCurve` | Quadratic H(Q) pump curve; `head_at_flow`, `affinity_scaled` |
| `SystemCurve` | Quadratic system requirement H_static + k·Q² |
| `Pump` | Pump at fixed speed with efficiency model |
| `Pump::operating_point` | Duty point (Q, H) via bisection |
| `Pump::power_at_flow` | Shaft power from hydraulic power and efficiency |
| `Pump::npsh_available` | NPSH(a) from suction-side conditions |

## Scope and limitations

- **Quadratic curves only** — no multi-segment or tabulated manufacturer
  curves, no drooping/instable curve handling beyond what the quadratic form
  allows.
- **Simple efficiency model** — a quadratic roll-off from the BEP, not a
  measured efficiency map; no motor/drive efficiency.
- **No cavitation limit checking** — the crate computes NPSH(a) but has no
  NPSH(r) data and does not flag damage limits.
- **No series/parallel combinations, no VFD curves beyond a single ratio,
  no specific-speed selection**; SI units throughout (m, m³/s, W, Pa).

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types.
- [`tpt-proc-fluid`](../tpt-proc-fluid) — the pipe hydraulics whose pressure
  drops become the k in your system curve.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
