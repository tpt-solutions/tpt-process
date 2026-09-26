# tpt-proc-absorption

> Kremser design of staged gas absorbers and strippers with ideal stages.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-absorption.svg)](https://crates.io/crates/tpt-proc-absorption)
[![Documentation](https://docs.rs/tpt-proc-absorption/badge.svg)](https://docs.rs/tpt-proc-absorption)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-absorption` sizes multistage countercurrent gas absorbers and
strippers with the classical Kremser analytical solution, and is part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

For dilute solutes with a linear equilibrium curve `y = m·x` and constant
gas/liquid flows, the absorption factor `A = L/(m·G)` is constant and the
staged-contact problem has a closed form. After `n` ideal stages the fraction
of entering solute remaining in the gas is

```text
frac = (A − 1) / (A^{n+1} − 1)          for A ≠ 1
frac = 1 / (n + 1)                      for A = 1
```

which is the Kremser equation. Deep absorption (`A > 1`) drives the fraction
toward zero; `A < 1` leaves most of the solute in the gas regardless of stage
count. The crate wraps this relation in an `Absorber` type that returns the
exiting solute fraction and the overall solute recovery for both absorption
(factor `A`) and its symmetric stripping case (stripping factor `S = 1/A`),
so a rich-liquid regeneration column is modeled with the same arithmetic.

## Features

- `Absorber::new(num_stages)` builds an ideal-stage countercurrent column.
- `Absorber::absorb(entering_solute, absorption_factor)` returns an
  `AbsorptionResult` with the exiting gas solute fraction, `solute_recovery`,
  and the raw Kremser `fraction_remaining`.
- `Absorber::strip(entering_solute, stripping_factor)` treats regeneration
  with the symmetric Kremser form, mirroring `absorb` exactly.
- `kremser_fraction_remaining(stages, absorption_factor)` exposes the
  closed-form solution directly for use in other models.
- The unit absorption-factor limit `A = 1` is handled analytically as
  `1/(n + 1)` instead of the degenerate `0/0`.
- No-panic robustness: non-finite or non-positive absorption factors and zero
  stage counts return a fraction of 1.0 (zero recovery) rather than `NaN`.

## Installation

```toml
[dependencies]
tpt-proc-absorption = "0.1"
```

## Usage

```rust
use tpt_proc_absorption::{Absorber, kremser_fraction_remaining};

// 5 stages, absorption factor A = 1.5: fraction of solute remaining.
let frac = kremser_fraction_remaining(5, 1.5);
assert!(frac < 0.1); // deep absorption

let absorber = Absorber::new(5);
let result = absorber.absorb(0.05, 1.5);
assert!((result.solute_recovery - (1.0 - frac)).abs() < 1e-9);
```

## API tour

| Item | Purpose |
|---|---|
| `Absorber` | Countercurrent absorber/stripper with `num_stages` ideal stages |
| `Absorber::absorb` | Kremser absorption from a lean gas at factor `A = L/(m·G)` |
| `Absorber::strip` | Kremser stripping from a rich liquid at factor `S` |
| `AbsorptionResult` | Exiting solute fraction, solute recovery, fraction remaining |
| `kremser_fraction_remaining` | Closed-form fraction of solute left in the gas |

## Scope and limitations

- Constant absorption factor only: linear equilibrium `y = m·x` and constant
  gas/liquid flows; curved isotherms and concentrated solutes are out of scope.
- All stages are ideal equilibrium contacts — no tray or packing efficiency,
  no HTU-NTU or rate-based mass transfer.
- `absorb` assumes the entering solvent is solute-free (lean); a nonzero
  entering-liquid solute load shifts the operating line and is not modeled.
- Single dilute solute on a mole-fraction basis; no heat of absorption, no
  temperature profile, and no multicomponent interactions.

## Related crates

- Depends on [`tpt-proc-core`](../../core/tpt-proc-core) for core stream
  types.
- Sits in the separations domain alongside `tpt-proc-distillation`,
  `tpt-proc-extraction`, `tpt-proc-membranes`, and `tpt-proc-crystallization`.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
