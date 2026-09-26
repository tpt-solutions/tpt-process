# tpt-proc-crystallization

> Cooling-crystallization yield and MSMPR population-balance estimates.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-crystallization.svg)](https://crates.io/crates/tpt-proc-crystallization)
[![Documentation](https://docs.rs/tpt-proc-crystallization/badge.svg)](https://docs.rs/tpt-proc-crystallization)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-crystallization` estimates how much solute crystallizes on cooling
and characterizes a well-mixed (MSMPR) crystallizer with a compact
population-balance kernel, and is part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

The yield side is a solubility-curve material balance. With concentrations
expressed on a solvent basis (kg solute per kg free solvent), cooling a
solution from `c_initial` to `c_final` crystallizes
`solvent_kg·(c_initial − c_final)`, since the solvent inventory is unchanged.
`cooling_yield` normalizes this to a fraction of the initial solution mass
(`Y = (c_i − c_f)/(1 + c_i)` per kg of solution), and `crystal_mass_kg`
returns the absolute crystal crop; heating (or no supersaturation) yields
exactly zero.

The crystallizer side applies the mixed-suspension, mixed-product-removal
(MSMPR) population-balance results of Randolph and Larson. Nucleation is the
empirical power law `B⁰ = k_B·G^i·M_T^j` in growth rate `G` and magma
density `M_T` (fixed exponents `i = 2, j = 1` for scale estimates), and the
dominant crystal size comes from the moment relations of an MSMPR
crystallizer as `L_D = 3.67·G·τ` — e.g. `G = 1e-8 m/s` at a one-hour
residence gives roughly 132 µm crystals.

## Features

- `cooling_yield(initial_solution_kg, c_initial, c_final)` returns the
  crystal yield as a fraction of the initial solution, clamped to `[0, 1]`.
- `crystal_mass_kg(solvent_kg, c_initial, c_final)` gives the absolute crop
  from the solvent-basis solubility change.
- `Crystallizer::new(volume_m3)` defines a well-mixed crystallizer by its
  working suspension volume.
- `Crystallizer::msmpr_nucleation(growth_m_s, kb, magma_density_kg_m3)`
  evaluates the `B⁰ = k_B·G²·M_T` power law (volume-corrected).
- `Crystallizer::msmpr_dominant_size(growth_m_s, residence_time_s)`
  evaluates `L_D = 3.67·G·τ` from the MSMPR moments.
- Direction-safe: `cooling_yield` and `crystal_mass_kg` return zero when the
  "final" concentration exceeds the initial (heating crystallizes nothing).

## Installation

```toml
[dependencies]
tpt-proc-crystallization = "0.1"
```

## Usage

```rust
use tpt_proc_crystallization::{cooling_yield, Crystallizer};

// KNO3 in water: solubility ~38.3 wt% at 80 °C vs ~13.3 wt% at 20 °C.
let yield_frac = cooling_yield(1000.0, 0.51, 0.15);
assert!((0.2..0.3).contains(&yield_frac));

let c = Crystallizer::new(2.0); // m³ suspension
assert!(c.msmpr_nucleation(1.0, 1.0e8, 2.0) > 0.0);
```

## API tour

| Item | Purpose |
|---|---|
| `cooling_yield` | Crystal fraction recovered per kg of initial solution |
| `crystal_mass_kg` | Absolute crystal mass from the solvent-basis balance |
| `Crystallizer` | Well-mixed crystallizer with a working volume in m³ |
| `Crystallizer::msmpr_nucleation` | Power-law nucleation rate `B⁰` in crystals/(m³·s) |
| `Crystallizer::msmpr_dominant_size` | Dominant crystal size `L_D = 3.67·G·τ` |

## Scope and limitations

- Solubility data are supplied by the caller (kg solute per kg free solvent);
  there is no built-in solubility database or phase diagram.
- No crystal size distribution is produced — only the dominant size
  `L_D = 3.67·G·τ` from the moment relations.
- The nucleation law uses the fixed exponent set `i = 2, j = 1` as a scale
  estimate; system-specific exponents and kinetic constants are not
  configurable.
- Evaporative crystallization is not covered (solvent lost = 0), and there
  are no fines destruction, classifier, or seeding models; the energy
  balance (cooling duty, heat of crystallization) is out of scope.

## Related crates

- Depends on [`tpt-proc-core`](../../core/tpt-proc-core) for core stream
  types.
- Sits in the separations domain alongside `tpt-proc-distillation`,
  `tpt-proc-absorption`, `tpt-proc-extraction`, and `tpt-proc-membranes`.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
