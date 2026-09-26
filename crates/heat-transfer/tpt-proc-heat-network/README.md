# tpt-proc-heat-network

> Pinch analysis: minimum-utility targets, composite curves, grand composite, and HEN synthesis.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-heat-network.svg)](https://crates.io/crates/tpt-proc-heat-network)
[![Documentation](https://docs.rs/tpt-proc-heat-network/badge.svg)](https://docs.rs/tpt-proc-heat-network)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-heat-network` performs pinch analysis and heat-exchanger-network
(HEN) synthesis: how much heating and cooling a process really needs, where
the pinch sits, and a minimum-utility network of matches that achieves the
targets. Part of the [tpt-process](../../../README.md) process-engineering
suite; the network synthesis follows the pinch-design method as specified in
[RFC 0004](../../../rfcs/0004-pinch-analysis.md).

## Overview

A problem is a list of hot streams, cold streams (each a `ProcessStream`
with supply/target temperatures and a constant heat-capacity flow), and a
ΔT_min. `PinchAnalysis` runs the **problem table algorithm**: temperatures
are shifted (hot −ΔT_min/2, cold +ΔT_min/2), a heat cascade is built over
the shifted intervals, and the most negative flux fixes the minimum hot
utility before the cascade is re-run. `minimum_utilities` reports
`min_heating_duty`, `min_cooling_duty`, and the pinch temperature pair —
and correctly reports **threshold problems** (cascades that never dip
negative) as having no pinch.

`composite_curves` returns the hot and cold composite polylines in
(duty, temperature) coordinates, and `grand_composite_curve` returns the
(shifted temperature, heat flux) profile for utility-placement decisions.

`network_synthesis` then generates a feasible **minimum-utility network**:
within each shifted-temperature interval, hot capacity is matched against
cold capacity in deterministic stream order and per-interval surpluses
become heaters/coolers. First law holds by construction and every match
respects ΔT_min (all contact happens inside a shifted interval). The tests
verify the first-law identity `Q_H,min − Q_C,min = ΣQ_cold − ΣQ_hot` and
that synthesized utility placement matches the targets.

## Features

- `PinchAnalysis::new` — problem definition over hot/cold stream lists and
  ΔT_min (invalid inputs degrade to all-zero targets, never panic).
- `PinchAnalysis::minimum_utilities` — problem table algorithm: minimum
  hot/cold utility and the pinch pair via `UtilityTargets`.
- `PinchAnalysis::composite_curves` — hot and cold composite curves.
- `PinchAnalysis::grand_composite_curve` — grand composite profile.
- `PinchAnalysis::pinch_temperature` — the unshifted pinch pair.
- `PinchAnalysis::network_synthesis` — minimum-utility HEN synthesis with
  `ExchangerMatch`es plus heaters/coolers, flagged above/below pinch.
- `ProcessStream::hot` / `ProcessStream::cold` / `ProcessStream::duty` —
  stream construction and duty.

## Installation

```toml
[dependencies]
tpt-proc-heat-network = "0.1"
```

## Usage

```rust
use tpt_proc_heat_network::{PinchAnalysis, ProcessStream};

// Classic two-hot / two-cold problem, ΔT_min = 10 K.
let pinch = PinchAnalysis::new(
    vec![
        ProcessStream::hot(433.15, 318.15, 15.0),   // kW/K
        ProcessStream::hot(493.15, 333.15, 25.0),
    ],
    vec![
        ProcessStream::cold(318.15, 448.15, 10.0),
        ProcessStream::cold(333.15, 408.15, 20.0),
    ],
    10.0,
);
let targets = pinch.minimum_utilities();

// First-law identity: Q_H,min − Q_C,min = ΣQ_cold − ΣQ_hot.
let q_hot_total: f64 = (433.15 - 318.15) * 15.0 + (493.15 - 333.15) * 25.0;
let q_cold_total: f64 = (448.15 - 318.15) * 10.0 + (408.15 - 333.15) * 20.0;
assert!((targets.min_heating_duty - targets.min_cooling_duty
    - (q_cold_total - q_hot_total)).abs() < 1e-6);
// Hot duty exceeds cold duty with ample overlap: a threshold problem
// needs no hot utility and has no pinch.
assert_eq!(targets.min_heating_duty, 0.0);
assert!(targets.pinch_hot_temp.is_none());
```

## API tour

| Item | Purpose |
|---|---|
| `PinchAnalysis` | The problem: hot streams, cold streams, ΔT_min |
| `ProcessStream` | Constant-cp stream segment; `hot`/`cold` constructors |
| `UtilityTargets` | Minimum utilities + pinch temperatures |
| `CompositeCurves` | Hot/cold composite polylines (duty, T) |
| `GrandCompositeCurve` | Shifted-T heat cascade profile |
| `HeatExchangerNetwork` | Synthesized matches + heaters/coolers |
| `ExchangerMatch` | One hot/cold match with duty and pinch side |

## Scope and limitations

- **Constant heat-capacity flow** — streams are linear in (duty, T); no
  phase change, latent segments, or variable-cp stream segmentation.
- **Constant ΔT_min** — a single global approach temperature; no
  stream-specific ΔT contributions (no ΔT_cont tables).
- **Feasible, not optimal, networks** — `network_synthesis` produces a
  minimum-utility network honoring the ΔT_min and cp rules; unit count is
  not minimized (no MER-network stream splitting, no Hohmann/cost targeting,
  no minimum-number-of-units optimization) per RFC 0004.
- Utility placement is reported as aggregate per-interval duties, not
  utility types or temperature levels.
- SI-consistent units: K, W (or kW/K for cp with duties in kW).

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types.
- [`tpt-proc-energy`](../../integration/tpt-proc-energy) — converts these
  pinch targets into utility loads on a `tpt-energy` system model.
- [`examples/pinch-analysis`](../../../examples/pinch-analysis) in the
  repository root runs this crate end to end.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
