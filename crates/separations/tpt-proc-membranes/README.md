# tpt-proc-membranes

> Solution-diffusion membrane transport for gas separations and reverse osmosis.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-membranes.svg)](https://crates.io/crates/tpt-proc-membranes)
[![Documentation](https://docs.rs/tpt-proc-membranes/badge.svg)](https://docs.rs/tpt-proc-membranes)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-membranes` computes steady permeate flux, permeance, selectivity,
osmotic pressure, and rejection for solution-diffusion membrane elements, and
is part of the [tpt-process](../../../README.md) process-engineering suite.

## Overview

Gas transport through a dense selective layer follows the solution-diffusion
law: the flux of a component is `J = P·Δp/ℓ`, the product of its
permeability `P` and its partial-pressure drop across a layer of thickness
`ℓ`. The crate works in the units membrane engineers actually use —
permeability in Barrer (`1 Barrer = 3.348e-16 mol·m/(m²·s·Pa)`) and
permeance in GPU (`1 GPU = 3.348e-10 mol/(m²·s·Pa)`) — and converts to SI
internally. Ideal selectivity is the simple permeability ratio of the fast
to the slow gas.

For liquid separations the crate implements reverse-osmosis water flux
`J_w = A·(ΔP − Δπ)`, where the net driving force is the applied pressure
difference minus the osmotic pressure difference, both clamped at zero so a
sub-osmotic ΔP yields no flux. Osmotic pressure itself comes from the
van 't Hoff relation `π = i·c·R·T`, and salt rejection is reported as
`R = 1 − c_p/c_f` from feed and permeate concentrations.

## Features

- `Membrane::new(thickness, permeability_barrer)` defines a selective layer
  (thickness in m, permeability in Barrer).
- `Membrane::gas_flux` returns the steady permeate flux in mol/(m²·s) from
  feed- and permeate-side partial pressures; the driving force is clamped at
  zero, so reverse or null pressure drops give exactly zero flux.
- `Membrane::permeance_gpu` reports `P/ℓ` in GPU, consistent with the same
  Barrer conversion used by `gas_flux`.
- `Membrane::selectivity` gives the ideal selectivity `α = P_fast/P_slow`.
- `Membrane::liquid_flux` gives RO solvent flux in m/s for a water
  permeability `A` in m/(s·Pa).
- `Membrane::osmotic_pressure` evaluates `π = i·c·R·T` for a solution with
  concentration in mol/m³ (seawater-strength checks included in tests).
- `Membrane::rejection` returns `R = 1 − c_p/c_f` clamped to `[0, 1]`.

## Installation

```toml
[dependencies]
tpt-proc-membranes = "0.1"
```

## Usage

```rust
use tpt_proc_membranes::Membrane;

// CO2-selective polymer membrane.
let membrane = Membrane::new(2.0e-6, 30.0); // 2 µm, 30 Barrer CO2
let flux = membrane.gas_flux(1.5e6, 0.2e6, 30.0e-3);
assert!(flux > 0.0);
```

## API tour

| Item | Purpose |
|---|---|
| `Membrane` | Solution-diffusion element: selective-layer thickness + permeability |
| `Membrane::new` | Construct from thickness (m) and Barrer permeability |
| `Membrane::gas_flux` | `J = P·Δp/ℓ` in mol/(m²·s) |
| `Membrane::permeance_gpu` | Permeance `P/ℓ` in GPU |
| `Membrane::selectivity` | Ideal selectivity from two permeabilities |
| `Membrane::liquid_flux` | RO flux `J_w = A·(ΔP − Δπ)` in m/s |
| `Membrane::osmotic_pressure` | van 't Hoff osmotic pressure, Pa |
| `Membrane::rejection` | Salt rejection `R = 1 − c_p/c_f` |

## Scope and limitations

- Single-component fluxes: no multicomponent coupling between permeating
  species, and no module-level staging, stage cut, or membrane-area sizing —
  this is an element-level transport model.
- Permeability is a constant per call; pressure, temperature, and
  composition dependence, concentration polarization, and fouling are not
  modeled.
- The van 't Hoff relation is the dilute-solution limit of osmotic pressure;
  brines far from ideality will deviate.
- No module hydraulics — spiral-wound or hollow-fiber pressure drops,
  feed-spacer effects, and recovery-dependent driving force are out of scope.

## Related crates

- Depends on [`tpt-proc-core`](../../core/tpt-proc-core) for core stream
  types.
- Sits in the separations domain alongside `tpt-proc-distillation`,
  `tpt-proc-absorption`, `tpt-proc-extraction`, and
  `tpt-proc-crystallization`.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
