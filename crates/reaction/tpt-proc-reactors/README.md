# tpt-proc-reactors

> Steady-state CSTR and PFR reactor simulation over `tpt-proc-reaction` kinetics, with adiabatic energy balance.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-reactors.svg)](https://crates.io/crates/tpt-proc-reactors)
[![Documentation](https://docs.rs/tpt-proc-reactors/badge.svg)](https://docs.rs/tpt-proc-reactors)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-reactors` turns `tpt-proc-reaction` kinetics into reactor outlet
predictions — a fixed-point CSTR solve and an RK4-integrated PFR — and is
part of the [tpt-process](../../../README.md) process-engineering suite.

## Overview

A `ReactorSim` is constructed over a set of `tpt_proc_reaction::Reaction`
objects (parallel reaction sets are supported: every solver sums the
contributions of all reactions). Both models share one interface: a
specification (reacting volume), feed composition and volumetric flow, a feed
temperature, and an `adiabatic` switch, and both return a `ReactorResult`
with outlet concentrations, per-reactant conversion, outlet temperature, and
heat duty.

The **CSTR** solves the design equation `c_out = c_in + τ·r(c_out)` by
under-relaxed (ω = 0.5) fixed-point iteration, which damps the oscillation
the undamped iteration shows for `τ·k ≳ 1`. The **PFR** integrates
`dc/dt = r(c, T)` through the residence time with fourth-order Runge-Kutta
(2000 fixed steps), co-integrating temperature when adiabatic. Both are
validated against their analytic limits in tests: `X = kτ/(1+kτ)` for a
first-order CSTR and `X = 1 − e^(−kτ)` for a first-order PFR, with the PFR
ahead of the CSTR for positive-order kinetics.

The adiabatic energy balance uses the heat release
`Σ(−ΔH_R)·r` from `ReactorSim::heat_release` and a fixed volumetric heat
capacity (4.0e6 J/(m³·K), water-like) to compute the outlet temperature rise.

## Features

- `ReactorSim::new(reactions)` builds a simulator over a `Vec` of
  `tpt_proc_reaction::Reaction` — reaction networks, not just single steps.
- `ReactorSim::cstr(&CstrSpec, feed_c, feed_rate_m3s, temperature,
  adiabatic)` solves the steady-state CSTR by damped fixed-point iteration.
- `ReactorSim::pfr(&PfrSpec, feed_c, feed_rate_m3s, temperature, adiabatic)`
  integrates the plug-flow balances with RK4.
- `ReactorResult` carries outlet `concentrations` (mol/m³), `conversion`
  per reactant (components with negative stoichiometry, clamped to `[0, 1]`),
  outlet `temperature`, and `heat_duty`.
- `ReactorSim::heat_release(c, t)` reports the instantaneous heat release
  rate in W from Σ(−ΔH_R)·r over the reaction set.
- Conversion is derived generically from the reaction set's stoichiometry —
  any reactant consumed anywhere in the set is tracked automatically.
- Solver accuracy is pinned by unit tests against the analytic CSTR/PFR
  first-order limits.

## Installation

```toml
[dependencies]
tpt-proc-reactors = "0.1"
```

## Usage

```rust
use std::collections::BTreeMap;
use tpt_proc_reaction::{RateLaw, Reaction};
use tpt_proc_reactors::{CstrSpec, PfrSpec, ReactorSim};

// First-order A → B, k = 0.5 s⁻¹ at feed temperature.
let mut stoich = BTreeMap::new();
stoich.insert(0, -1.0);
let mut orders = BTreeMap::new();
orders.insert(0, 1.0);
let reaction = Reaction::new(
    "A → B",
    stoich,
    RateLaw::Arrhenius {
        pre_exponential: 0.5,
        activation_energy: 0.0,
        orders,
    },
    -60.0e3,
);

// CSTR of 2 m³ at 1 m³/s feed → residence time 2 s → X = kτ/(1+kτ).
let cstr = ReactorSim::new(vec![reaction.clone()]);
let out = cstr.cstr(&CstrSpec { volume: 2.0 }, &[10.0], 1.0, 350.0, false);
assert!((out.conversion[0] - 0.5).abs() < 1e-6);

// PFR of 2 m³ at 1 m³/s → X = 1 − e^(−kτ).
let pfr = ReactorSim::new(vec![reaction]);
let out = pfr.pfr(&PfrSpec { volume: 2.0 }, &[10.0], 1.0, 350.0, false);
assert!((out.conversion[0] - (1.0 - (-1.0_f64).exp())).abs() < 1e-4);
```

## API tour

| Item | Purpose |
|---|---|
| `ReactorSim` | Simulator over a set of `tpt_proc_reaction::Reaction` objects |
| `ReactorSim::cstr` | Steady-state CSTR by under-relaxed fixed-point iteration |
| `ReactorSim::pfr` | Steady-state PFR by RK4 integration over residence time |
| `ReactorSim::heat_release` | Instantaneous heat release Σ(−ΔH_R)·r in W |
| `CstrSpec` / `PfrSpec` | Reacting volume, m³ |
| `ReactorResult` | Outlet concentrations, conversions, temperature, heat duty |

## Scope and limitations

- The exported models are steady-state **CSTR and PFR only**. No batch,
  packed-bed, or fluidized-bed reactor model exists in the crate yet, and no
  catalyst weighting of kinetics is applied — rate laws are used as given on
  a per-feed-volume basis.
- Constant volumetric flow is assumed (liquid-like); gas-phase expansion and
  mole change effects on residence time are not modeled.
- Energy options are isothermal or adiabatic only; the adiabatic balance uses
  a fixed water-like volumetric heat capacity of 4.0e6 J/(m³·K), and there is
  no jacket/coil duty specification or heat-transfer coefficient.
- No pressure drop, no interphase or intraparticle mass-transfer limits, and
  no residence-time distribution (ideal plug flow and perfect mixing only).
- Integration budgets are fixed: 2000 RK4 steps for the PFR and at most
  20,000 relaxed iterations for the CSTR.

## Related crates

- Depends on [`tpt-proc-reaction`](../tpt-proc-reaction) for rate laws and
  reaction heats, and [`tpt-proc-core`](../../core/tpt-proc-core) for core
  types.
- Used by the `cstr-reactor` worked example in the workspace.
- For transient (dynamic) behavior, the suite's
  [`tpt-proc-dynamics`](../../dynamics/tpt-proc-dynamics) provides
  integrators and FOPDT models.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
