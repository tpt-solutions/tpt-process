# tpt-proc-compressors

> Gas compression duty: isentropic and polytropic head, shaft work, and discharge temperature.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-compressors.svg)](https://crates.io/crates/tpt-proc-compressors)
[![Documentation](https://docs.rs/tpt-proc-compressors/badge.svg)](https://docs.rs/tpt-proc-compressors)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-compressors` calculates compression work, head, and discharge
temperature for centrifugal and reciprocating machines — part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

The crate implements the standard pre-design method for compressors: an
ideal-gas treatment with real efficiency factors. A `CompressorService`
captures the service conditions — suction/discharge pressures (absolute,
Pa), suction temperature (K), heat-capacity ratio κ, isentropic efficiency,
and a suction compressibility factor Z — and `evaluate` returns a
`CompressionResult` with:

- the **isentropic head** `Z·(k/(k−1))·R_s·T₁·(r^((k−1)/k) − 1)` in J/kg,
- the **actual (shaft) specific work** `head/η_is`,
- the **actual discharge temperature** from `T₂ = T₁ + W/(cp·Z)` with
  `cp = k/(k−1)·R_s`, and
- the **pressure ratio** r.

For multistage centrifugal machines the crate also provides the polytropic
head via `polytropic_head`, converting polytropic efficiency to the exponent
n through `(n−1)/n = (k−1)/(k·η_p)`. Shaft power for a given mass flow is a
one-liner with `power`.

## Features

- `CompressorService` — one struct describing the compression service.
- `CompressorService::evaluate` — isentropic head, actual work, discharge
  temperature, and pressure ratio in one call.
- `CompressorService::polytropic_head` — polytropic head from a polytropic
  efficiency (the centrifugal-machine convention).
- `CompressorService::power` — shaft power in W for a mass flow in kg/s.
- `CompressionResult` — the four-result struct with `head_isentropic`,
  `work_actual`, `discharge_temperature`, `pressure_ratio`.
- Compressibility factor Z on the service for near-ideal correction.

## Installation

```toml
[dependencies]
tpt-proc-compressors = "0.1"
```

## Usage

```rust
use tpt_proc_compressors::CompressorService;

// Air compressed 1 bar → 4 bar from 300 K at 75 % isentropic efficiency.
let service = CompressorService {
    suction_pressure: 101_325.0,
    discharge_pressure: 405_300.0,
    suction_temperature: 300.0,
    kappa: 1.4,
    isentropic_efficiency: 0.75,
    compressibility: 1.0,
};
let r_s = 8.314462618 / 0.02897; // specific gas constant of air, J/(kg·K)

let result = service.evaluate(r_s);
println!("head: {} J/kg, discharge T: {} K",
    result.head_isentropic, result.discharge_temperature);

// Multistage view: polytropic head at 72 % polytropic efficiency.
let head_poly = service.polytropic_head(r_s, 0.72);

// Shaft power for 2 kg/s.
let shaft_power = service.power(2.0, r_s); // W
```

## API tour

| Item | Purpose |
|---|---|
| `CompressorService` | Service definition: pressures, temperatures, κ, η, Z |
| `CompressorService::evaluate` | Head, work, discharge T, pressure ratio |
| `CompressorService::polytropic_head` | Polytropic head from η_p |
| `CompressorService::power` | Shaft power for a mass flow |
| `CompressionResult` | Head / work / discharge temperature / ratio |

## Scope and limitations

- **Ideal-gas model** — real-gas behavior enters only through a single
  suction compressibility factor Z; no EOS integration, no real-fluid
  enthalpy balances.
- **Pre-design accuracy** — meant for estimates and feasibility work, not
  manufacturer performance prediction; no impeller/geometry modeling.
- **Single stage** — `evaluate` treats the service as one compression step;
  staging, intercooling, and surge/choke maps are out of scope.
- κ is assumed constant across the compression; SI units throughout
  (Pa, K, J/kg, W).

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types.
- [`tpt-proc-fluid`](../tpt-proc-fluid) — sibling fluid-flow crate providing
  the pipe hydraulics around the machine.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
