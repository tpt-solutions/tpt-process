# tpt-proc-thermo-core

> Thermodynamic property framework: components, ideal-gas heat capacities, and the property-package contract.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-thermo-core.svg)](https://crates.io/crates/tpt-proc-thermo-core)
[![Documentation](https://docs.rs/tpt-proc-thermo-core/badge.svg)](https://docs.rs/tpt-proc-thermo-core)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Defines the pieces every thermodynamic model in the suite shares. Part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

Rather than letting each model invent its own component type and calling
convention, `tpt-proc-thermo-core` fixes the shared vocabulary:

- [`Component`] — pure-component constants: molecular weight, critical
  temperature/pressure/volume, Pitzer acentric factor, normal boiling point,
  and an ideal-gas heat-capacity correlation — with `validate`,
  `reduced_temperature`, `reduced_pressure`, and
  `critical_compressibility` helpers.
- [`CpCorrelation`] — ideal-gas heat capacity with analytic enthalpy and
  entropy integrals. The DIPPR-127 form `Cp/R = A + B·T + C·T² + D·T³ +
  E/T²` is supported alongside a plain constant, following the DIPPR-style
  correlations used across the suite's data (Smith–Van Ness–Abbott Appendix
  C / NIST tabulations in the database crate).
- The [`EquationOfStateModel`] and [`ActivityCoefficientModel`] traits —
  the contracts `tpt-proc-thermo-eos` and `tpt-proc-thermo-activity`
  implement.
- [`PropertyPackage`] — the concrete package tying components + EOS +
  activity model together and implementing
  `tpt_proc_core::PropertyPackage`, so the flash solver and flowsheet engine
  can consume it.

Keeping the traits here (rather than in the implementing crates) lets
[`PropertyPackage`] compose models without dependency cycles, and lets the
flash solver accept any package built from any model pair.

All quantities are SI: T in K, P in Pa, molar amounts in mol, energies in J.
The universal gas constant [`R`] is exported with the CODATA 2018 value,
8.314 462 618 J/(mol·K).

## Features

- [`Component::new`] with full critical-state constants and
  `Component::validate` for physically consistent entries.
- [`CpCorrelation::dippr127`] and [`CpCorrelation::constant`], with
  `cp_ig`, and the analytic `enthalpy_ig` / `entropy_ig` integrals.
- [`PropertyPackage::new`] over an ordered component list, plus the
  `with_eos`, `with_activity`, and `with_reference_temperature` builders.
- Package-level property queries: `cp_ig_mixture`, `vapor_pressure`
  (EOS saturation pressure when available, otherwise the Trouton–Clausius
  correlation anchored at the normal boiling point),
  `activity_coefficients` (γ = 1 when no model is attached), and
  `rackett_liquid_density` (Rackett/Spencer–Danner corresponding states).
- [`EquationOfStateModel`] — `ln_fugacity_coefficients`, `compressibility`,
  `enthalpy_departure`, `entropy_departure`, `pure_vapor_pressure`.
- [`ActivityCoefficientModel`] — `ln_gamma` for liquid excess-Gibbs models.
- [`PhaseSelection`] (`Vapor`, `Liquid`, `Stable`) for three-root cubic
  states, the [`Phase`] and [`PropertyMethods`] enums, and the [`ThermoError`]
  taxonomy (`StateOutOfDomain`, `Numeric`, `Unsupported`, …).

## Installation

```toml
[dependencies]
tpt-proc-thermo-core = "0.1"
```

## Usage

```rust
use tpt_proc_core::Composition;
use tpt_proc_thermo_core::{Component, CpCorrelation, PropertyPackage};

// Water with a DIPPR-127 ideal-gas Cp correlation.
let water = Component::new(
    "water", "7732-18-5", 18.015e-3,
    647.096, 22.064e6, 55.9e-6, 0.344, 373.124,
    CpCorrelation::dippr127(3.470, 1.450e-3, 0.0, 0.0, 0.121e5),
);

let package = PropertyPackage::new(vec![water]).unwrap();
let feed = Composition::from_mole_fractions(&[1.0]).unwrap();
let cp = package.cp_ig_mixture(&feed, 400.0);
assert!(cp > 25.0 && cp < 45.0); // J/(mol·K), ideal gas at 400 K
```

## API tour

| Item | Purpose |
|---|---|
| `Component` | Pure-component constants (critical state, ω, Tb, ideal-gas Cp) |
| `CpCorrelation` | DIPPR-127 or constant ideal-gas Cp with analytic integrals |
| `PropertyPackage` | Components + optional EOS + optional activity model, as one queryable package |
| `EquationOfStateModel` (trait) | Fugacity, compressibility, and departure-function contract |
| `ActivityCoefficientModel` (trait) | Liquid γ-model contract |
| `PhaseSelection` / `Phase` / `PropertyMethods` | Phase-root selection and method tagging |
| `ThermoError`, `Result` | Domain, numeric, and unsupported-model errors |
| `R` | Universal gas constant (CODATA 2018), J/(mol·K) |

## Scope and limitations

- A framework, not a model set: the EOS and activity implementations live in
  `tpt-proc-thermo-eos` and `tpt-proc-thermo-activity`.
- With no EOS attached, vapor pressures fall back to the Trouton–Clausius
  correlation and liquid density to Rackett with linear mixing of critical
  properties — both are estimate-grade approximations.
- Ideal-gas heat capacities only: no liquid/solid Cp correlations, no
  transport properties (viscosity, thermal conductivity), no latent-heat
  tabulations.
- No serialization support (no `serde` derive); component data enters via
  `Component::new` or the database crate.
- SI units only; the package cannot translate unit systems.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — upstream: `Composition` and
  the `tpt_proc_core::PropertyPackage` trait implemented here.
- [`tpt-proc-thermo-eos`](../tpt-proc-thermo-eos) — implements
  `EquationOfStateModel` (Peng-Robinson, SRK).
- [`tpt-proc-thermo-activity`](../tpt-proc-thermo-activity) — implements
  `ActivityCoefficientModel` (Wilson, NRTL, UNIQUAC, UNIFAC).
- [`tpt-proc-thermo-phase`](../tpt-proc-thermo-phase) — consumes packages
  from this crate for flash calculations.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
