# tpt-proc-wasm

> WebAssembly bindings for the tpt-process engineering suite.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-wasm.svg)](https://crates.io/crates/tpt-proc-wasm)
[![Documentation](https://docs.rs/tpt-proc-wasm/badge.svg)](https://docs.rs/tpt-proc-wasm)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The JS boundary of the tpt-process stack: browser-based PFD tools, educational
simulators, and edge controllers consume the solver crates through the
`#[wasm_bindgen]` façades defined here. This crate is part of the
[tpt-process](../../../README.md) process-engineering suite.

**Note:** `tpt-proc-wasm` is *not* published to crates.io. It is primarily
consumed as a `wasm-bindgen` facade: build it for `wasm32` targets and load
the generated bindings from JavaScript, or depend on it as a plain Rust crate
(it builds and tests on native targets too, where the bindings become ordinary
Rust types).

## Overview

The crate wraps the thermodynamic stack — the built-in chemical database, the
Peng-Robinson cubic EOS, and the flash solver — behind a small, JS-friendly
surface:

- [`WasmFlashCalculator`] resolves a whitespace-separated component list
  (e.g. `"water methanol"`) against `ChemicalDatabase::builtin()`, attaches a
  Peng-Robinson EOS, and exposes `pt_flash` (returning a
  [`WasmFlashResult`] with vapor fraction, phase compositions, iteration
  count, and a `degenerate` flag for pure-component saturation points) and
  `bubble_point`.
- [`WasmHeatExchanger`] rates a counter-current exchanger via the
  epsilon-NTU method from `tpt-proc-heat-exchangers`, returning duty and
  outlet temperatures in a [`WasmRatingResult`].
- [`lookup_component`] fetches constants for one chemical (by name or CAS
  number) as a [`WasmComponent`].
- Failures cross the boundary as a [`WasmError`] carrying a class tag
  (`"validation"`, `"solver"`, `"lookup"`) and a human-readable message —
  never a panic.

## Features

- [`WasmFlashCalculator::new`] — constructor resolving component names
  against the built-in database and wiring the Peng-Robinson EOS.
- [`WasmFlashCalculator::pt_flash`] — validated PT flash; composition length
  is checked against the component list.
- [`WasmFlashResult`] getters — `vapor_fraction`, `temperature`, `pressure`,
  `iterations`, `degenerate`, `liquid_composition`, `vapor_composition`.
- [`WasmFlashCalculator::bubble_point`] — bubble-point temperature in K at a
  fixed pressure.
- [`WasmHeatExchanger::rate`] — epsilon-NTU rating (duty, hot/cold outlets,
  effectiveness) with input validation.
- [`WasmComponent`] and [`lookup_component`] — critical constants and
  acentric factor by name or CAS lookup.
- `WasmError` — structured error kind/message pairs across the JS boundary.
- Ships as `crate-type = ["cdylib", "rlib"]`, so the same code serves wasm
  artifacts and native tests.

## Installation

Not on crates.io — build from the workspace or a git dependency:

```toml
[dependencies]
tpt-proc-wasm = { git = "https://github.com/tpt-solutions/tpt-process" }
```

## Usage

Rust side (identical on native and wasm32):

```rust
use tpt_proc_wasm::{WasmFlashCalculator, WasmHeatExchanger};

// PT flash of a 50/50 water/methanol feed at 350 K, 1 atm.
let calc = WasmFlashCalculator::new("water methanol").unwrap();
let flash = calc.pt_flash(&[0.5, 0.5], 350.0, 101_325.0).unwrap();
println!("vapor fraction = {}", flash.vapor_fraction());

// Bubble point of pure benzene at 1 atm.
let benzene = WasmFlashCalculator::new("benzene toluene").unwrap();
let t_bubble = benzene.bubble_point(&[1.0, 0.0], 101_325.0).unwrap();
println!("T_bubble = {t_bubble} K");

// Epsilon-NTU rating of a 25 m2, U = 900 W/(m2*K) counter-current exchanger.
let hx = WasmHeatExchanger::new(25.0, 900.0).unwrap();
let rated = hx.rate(2.0e3, 4.0e3, 330.0, 290.0).unwrap();
println!("duty = {} W", rated.duty());
```

Build for the browser:

```sh
cargo build -p tpt-proc-wasm --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/tpt_proc_wasm.wasm --out-dir pkg
```

From JavaScript the façades behave like plain objects:
`new WasmFlashCalculator("water methanol")`, then
`calc.pt_flash([0.5, 0.5], 350.0, 101325.0)` returns an object with the
getters above.

## API tour

| Item | Purpose |
|---|---|
| `WasmFlashCalculator` | Flash + bubble-point façade over the built-in database and Peng-Robinson EOS |
| `WasmFlashResult` | Vapor fraction, phase compositions, iterations, degenerate flag |
| `WasmHeatExchanger` / `WasmRatingResult` | Epsilon-NTU rating façade and its result |
| `WasmComponent` / `lookup_component` | Single-component constants by name or CAS |
| `WasmError` | Class-tagged (`validation`/`solver`/`lookup`) error crossing the JS boundary |

## Scope and limitations

- A thin facade, not a new solver: only PT flash, bubble-point temperature,
  and heat-exchanger rating are exposed so far. PH flash, dew points, and the
  activity-model crates have no bindings yet.
- Components must exist in the built-in database; there is no way to register
  custom components through the facade.
- Only the Peng-Robinson EOS is wired; other EOS flavors or activity models
  are not reachable from JavaScript.
- Error reporting is string-based (`WasmError`); numeric results are `f64`
  and arrays of `f64` — no rich result objects beyond those listed here.

## Related crates

- [`tpt-proc-core`](../tpt-proc-core) — `Composition` validation at the boundary.
- [`tpt-proc-thermo-core`](../../thermodynamics/tpt-proc-thermo-core) —
  `PropertyPackage` construction behind the flash façade.
- [`tpt-proc-thermo-database`](../../thermodynamics/tpt-proc-thermo-database) —
  the built-in component lookup.
- [`tpt-proc-thermo-eos`](../../thermodynamics/tpt-proc-thermo-eos) and
  [`tpt-proc-thermo-phase`](../../thermodynamics/tpt-proc-thermo-phase) — the
  EOS and flash solver the calculator wraps.
- [`tpt-proc-heat-exchangers`](../../heat-transfer/tpt-proc-heat-exchangers) —
  the epsilon-NTU rating behind `WasmHeatExchanger`.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
