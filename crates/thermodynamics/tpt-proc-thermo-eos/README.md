# tpt-proc-thermo-eos

> Cubic equations of state: Peng-Robinson and Soave-Redlich-Kwong with van der Waals mixing.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-thermo-eos.svg)](https://crates.io/crates/tpt-proc-thermo-eos)
[![Documentation](https://docs.rs/tpt-proc-thermo-eos/badge.svg)](https://docs.rs/tpt-proc-thermo-eos)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The workhorse volumetric and fugacity models of the suite, implemented as one
generalized cubic. Part of the [tpt-process](../../../README.md)
process-engineering suite.

## Overview

Both flavors are implemented as the generalized 2-parameter cubic

```text
P = R·T / (v − b) − a / (v² + u·b·v + w·b²)
```

with (u, w) = (2, −1) for Peng-Robinson (1976) and (u, w) = (1, 0) for
Soave-Redlich-Kwong (1972). The attraction parameter uses the standard alpha
correlation `αᵢ(T) = [1 + mᵢ(1 − √Tr)]²` with the flavor-specific `m(ω)`,
and mixing is the classical van der Waals one-fluid rule with a symmetric
binary interaction matrix `kᵢⱼ` (see RFC 0001 in the repository's `rfcs/`
directory).

The solver-facing surface follows the
[`EquationOfStateModel`](https://docs.rs/tpt-proc-thermo-core) trait:
log fugacity coefficients per component, compressibility with explicit
multi-root handling, enthalpy and entropy departure functions from analytic
`d(a_mix)/dT` derivatives, and pure-component vapor pressure by fugacity
equality. Phase selection among multiple real compressibility roots uses
[`PhaseSelection::Vapor`], [`PhaseSelection::Liquid`], or
[`PhaseSelection::Stable`] (minimum mixture Gibbs energy via `ln φ`).

## Features

- [`CubicEos::peng_robinson`] and [`CubicEos::srk`] constructors over a
  `tpt_proc_thermo_core::Component` list; [`CubicFlavor`] names the member.
- [`MixingRule`] — `VanDerWaals` (implemented); `HuronVidal` and
  `WongSandler` can be set but are rejected at evaluation time until the
  activity-crate integration lands.
- [`CubicEos::with_binary_interaction`] / [`CubicEos::binary_interaction`] —
  symmetric kᵢⱼ management (diagonal implied zero).
- [`CubicEos::compressibility_roots`] — all positive real Z roots, ascending.
- [`CubicEos::select_root`] — vapor/liquid/stable root selection with the
  minimum-Gibbs tie-break.
- [`CubicEos::ln_phi_at_root`] and [`CubicEos::ln_phi_mixture`] — component
  and mixture fugacity coefficients at a specific root.
- `enthalpy_departure` / `entropy_departure` (via `EquationOfStateModel`)
  from analytic `da_mix_dt` derivatives; [`CubicEos::a_mix`],
  [`CubicEos::b_mix`], and [`CubicEos::pressure`] are public for custom
  solvers.
- `pure_vapor_pressure` — saturation pressure below Tc by fugacity equality.

## Installation

```toml
[dependencies]
tpt-proc-thermo-eos = "0.1"
```

## Usage

```rust
use tpt_proc_thermo_core::{Component, CpCorrelation, EquationOfStateModel};
use tpt_proc_thermo_eos::CubicEos;
use tpt_proc_core::Composition;

// Pure water saturation pressure at 100 °C (PR EOS).
let water = Component::new(
    "water", "7732-18-5", 18.015e-3,
    647.096, 22.064e6, 55.9e-6, 0.344, 373.124,
    CpCorrelation::constant(34.0),
);
let eos = CubicEos::peng_robinson(vec![water]);
let p_sat = eos.pure_vapor_pressure(0, 373.15).unwrap();

// Within ~3% of 1 atm for PR water.
assert!((p_sat - 101_325.0).abs() < 101_325.0 * 0.05);
let _ = Composition::from_mole_fractions(&[1.0]).unwrap();
```

## API tour

| Item | Purpose |
|---|---|
| `CubicEos` | A cubic EOS over a fixed component list (PR or SRK) |
| `CubicEos::peng_robinson` / `srk` | Flavor constructors |
| `CubicEos::with_binary_interaction` | Symmetric kᵢⱼ pair parameters |
| `CubicEos::compressibility_roots` / `select_root` | Multi-root Z handling and phase selection |
| `CubicEos::ln_phi_at_root` / `ln_phi_mixture` | Component/mixture fugacity coefficients |
| `CubicFlavor` | `PengRobinson` or `SoaveRedlichKwong` |
| `MixingRule` | Van der Waals (active); Huron-Vidal / Wong-Sandler (reserved) |

## Scope and limitations

- Only the classical van der Waals one-fluid mixing rule evaluates;
  Huron-Vidal and Wong-Sandler mixing are placeholders pending
  `tpt-proc-thermo-activity` integration (RFC 0001).
- `pure_vapor_pressure` is meaningful only below the critical temperature;
  at/above Tc (or below roughly 0.4·Tc, where the cubic saturation branch
  loses accuracy) it reports `ThermoError::StateOutOfDomain`.
- No volume translation, no Peneloux correction, no density accuracy
  guarantee beyond the raw cubic.
- No transport properties or derivative wealth (heat capacities at
  constant P/V, speed of sound) are exposed.
- Default `kᵢⱼ = 0` for unlisted pairs — supply literature values (see
  `tpt-proc-thermo-database`) for polar systems.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — upstream: `Composition`.
- [`tpt-proc-thermo-core`](../tpt-proc-thermo-core) — upstream: `Component`
  and the `EquationOfStateModel` trait implemented here.
- [`tpt-proc-thermo-database`](../tpt-proc-thermo-database) — ships built-in
  Peng-Robinson kᵢⱼ values for common pairs.
- [`tpt-proc-thermo-phase`](../tpt-proc-thermo-phase) — consumes this crate's
  fugacity coefficients in the flash solver.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
