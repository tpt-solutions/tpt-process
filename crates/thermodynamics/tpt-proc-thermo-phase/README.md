# tpt-proc-thermo-phase

> Phase equilibrium and flash calculations: PT/PH flash, Rachford-Rice, bubble and dew point.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-thermo-phase.svg)](https://crates.io/crates/tpt-proc-thermo-phase)
[![Documentation](https://docs.rs/tpt-proc-thermo-phase/badge.svg)](https://docs.rs/tpt-proc-thermo-phase)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The two-phase VLE workhorse of the suite (design per RFC 0002 in the
repository's `rfcs/` directory). Part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

[`FlashSolver`] implements the equilibrium calculations every flowsheet
depends on:

- **PT flash** — successive substitution on K-values seeded with the Wilson
  correlation `Kᵢ = (Pcᵢ/P)·exp(5.373·(1+ωᵢ)(1 − Tcᵢ/T))`, with a bracketed
  Rachford-Rice inner solve `g(β) = Σ zᵢ(Kᵢ−1)/(1+β(Kᵢ−1)) = 0` by bisection
  (strictly monotone on its bracket, so no divergence and no tuning). When an
  EOS is attached to the package, K-values come from phase-specific fugacity
  coefficients; otherwise the modified-Raoult path `Kᵢ = γᵢ·P_sat,ᵢ/P` is
  used with the attached activity model (or ideal γ = 1).
- **PH flash** — pressure fixed, enthalpy (J/mol of feed) specified; the
  equilibrium temperature is found by outer bisection on the PT-flash
  enthalpy.
- **Bubble and dew point** — `bubble_point_t` and `dew_point_t` solve
  Σ zᵢKᵢ(T) = 1 / Σ zᵢ/Kᵢ(T) = 1 by monotone bisection at fixed pressure.

Design commitments, enforced in code:

- **Never panics and never lies**: non-convergence is an `Err`
  (`FlashError::NotConverged`), and the converged iteration count is part of
  every result.
- **Mass balance is exact**: `z = (1−β)·x + β·y` holds to machine precision
  by construction.
- **Degenerate single-component two-phase states return a guarded result**
  (`degenerate: true`, β ≈ 0.5) instead of diverging.

## Features

- [`FlashSolver::new`] — flash over any `tpt_proc_thermo_core::PropertyPackage`.
- [`FlashSolver::peng_robinson`] — convenience constructor from a component
  list and mixing rule, returning `Result<Self, FlashError>` (component
  validation failures surface as `FlashError::Thermo`).
- [`FlashSolver::pt_flash`] — fixed T,P two-phase flash returning a
  [`FlashResult`].
- [`FlashSolver::ph_flash`] — fixed P, enthalpy-specified flash.
- [`FlashSolver::bubble_point_t`] / [`FlashSolver::dew_point_t`] —
  bubble/dew temperatures at fixed pressure.
- [`FlashResult`] — temperature, pressure, vapor fraction β, liquid and
  vapor compositions, the converged K-values, iteration count, and the
  `degenerate` flag.
- [`FlashError`] — `NotConverged`, `InvalidInput`, and `Thermo` variants;
  no panics anywhere.
- A [`prelude`](https://docs.rs/tpt-proc-thermo-phase) module re-exporting
  `FlashSolver`, `FlashResult`, `FlashError`, and `Composition` so flash work
  needs only this crate.

## Installation

```toml
[dependencies]
tpt-proc-thermo-phase = "0.1"
```

## Usage

```rust
use tpt_proc_core::Composition;
use tpt_proc_thermo_database::ChemicalDatabase;
use tpt_proc_thermo_eos::CubicEos;
use tpt_proc_thermo_phase::FlashSolver;
use tpt_proc_thermo_core::PropertyPackage;

let db = ChemicalDatabase::builtin();
let components = db.components_for(&["water", "methanol"]).unwrap();
let eos = CubicEos::peng_robinson(components.clone());

let package = tpt_proc_thermo_core::PropertyPackage::new(components)
    .unwrap()
    .with_eos(std::sync::Arc::new(eos));
let flash = FlashSolver::new(package);

let feed = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
let result = flash.pt_flash(&feed, 350.0, 101_325.0).unwrap();
println!("β = {}", result.vapor_fraction);
assert!((0.0..=1.0).contains(&result.vapor_fraction));
```

## API tour

| Item | Purpose |
|---|---|
| `FlashSolver` | A configured flash calculator over one property package |
| `FlashSolver::pt_flash` | Two-phase equilibrium at fixed T and P |
| `FlashSolver::ph_flash` | Equilibrium at fixed P and specified enthalpy |
| `FlashSolver::bubble_point_t` / `dew_point_t` | Saturation temperatures at fixed P |
| `FlashSolver::peng_robinson` | PR convenience constructor; `Result<Self, FlashError>` |
| `FlashResult` | β, phase compositions, K-values, iterations, degenerate flag |
| `FlashError` | Not-converged / invalid-input / thermo failures |
| `prelude` | Re-exports for one-crate flash usage |

## Scope and limitations

- **Two-phase vapor–liquid only**: no liquid–liquid equilibrium, no
  three-phase flash, no solid/precipitate phases or hydrates.
- The Rachford-Rice loop is bracketed bisection — bulletproof but
  computationally heavier per iteration than Newton methods; near-critical
  mixtures rely on K-value damping after 20 iterations.
- Bubble/dew residuals use the Wilson-correlation K estimate inside the
  bisection (the fast path), so those temperatures inherit Wilson's accuracy
  rather than the full EOS iteration.
- PH flash brackets the enthalpy by expanding temperature between (roughly)
  1 K and a growing upper bound; enthalpies outside the bracketable range
  return `FlashError::InvalidInput`.
- Single-phase results (subcooled/superheated) are reported with β = 0 or 1
  and K = 1; enthalpy-based phase-stability tests (tangent-plane) are not
  performed.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — upstream: `Composition`,
  `PhaseState`.
- [`tpt-proc-thermo-core`](../tpt-proc-thermo-core) — upstream:
  `PropertyPackage`, `PhaseSelection`, `ThermoError`.
- [`tpt-proc-thermo-eos`](../tpt-proc-thermo-eos) — upstream: the
  `CubicEos` fugacity path and `MixingRule`.
- [`tpt-proc-thermo-database`](../tpt-proc-thermo-database) — dev-dependency:
  supplies components for the examples and tests.
- [`tpt-proc-wasm`](../../core/tpt-proc-wasm) and
  [`tpt-proc-flowsheet`](../../simulation/tpt-proc-flowsheet) — downstream
  consumers of the flash solver.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
