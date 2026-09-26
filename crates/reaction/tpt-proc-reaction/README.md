# tpt-proc-reaction

> Reaction kinetics: Arrhenius power laws, Langmuir-Hinshelwood, and Michaelis-Menten rate laws.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-reaction.svg)](https://crates.io/crates/tpt-proc-reaction)
[![Documentation](https://docs.rs/tpt-proc-reaction/badge.svg)](https://docs.rs/tpt-proc-reaction)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-reaction` defines chemical reactions and their rate laws for use in
reactor models, and is part of the [tpt-process](../../../README.md)
process-engineering suite.

## Overview

The crate expresses a reaction as stoichiometry plus a rate law plus a heat
of reaction. `RateLaw` is an enum over the rate expressions that cover most
engineering work:

- **Arrhenius** — `r = A·exp(−Ea/RT)·Π cᵢ^orderᵢ`, the temperature-dependent
  power law (an Ea ≈ 54.5 kJ/mol reaction doubles between 300 and 310 K).
- **PowerLaw** — the temperature-independent form `r = k·Π cᵢ^orderᵢ` with
  the rate constant `k` fixed at its reference condition.
- **LangmuirHinshelwood** — the single-site inhibition form
  `r = k·Π cᵢ^orderᵢ / (1 + Σ Kⱼcⱼ)` for catalytic surfaces where reactants
  and products compete for sites.
- **MichaelisMenten** — enzyme kinetics `r = Vmax·c_S/(Km + c_S)`, which
  saturates: `c_S = Km` gives the half-maximal rate and saturating substrate
  approaches `Vmax`.

A `Reaction` binds a `RateLaw` to named stoichiometry (component index →
coefficient, negative for reactants) and a heat of reaction ΔH_R in J/mol.
`Reaction::dcdt` fills a `dc/dt` slot vector (`dcᵢ/dt = νᵢ·r`) for integrators,
and `Reaction::equilibrium_constant` applies the van 't Hoff relation
`K(T) = K_ref·exp(−ΔH_R/R·(1/T − 1/T_ref))`, so an exothermic reaction's K
falls with temperature.

Units are SI throughout: concentrations in mol/m³, time in s, rates in
mol/(m³·s), energies in J/mol.

## Features

- `RateLaw::rate(c, temperature)` evaluates the net rate in mol/(m³·s) for
  all four law families.
- `RateLaw::rate_constant(temperature)` returns `k(T)` — the Arrhenius
  factor where applicable, the plain `k` for PowerLaw and LHHW, and 1 for
  temperature-independent Michaelis-Menten.
- `Reaction::new(name, stoichiometry, law, heat_of_reaction)` builds a
  reaction from a `BTreeMap` stoichiometry (deterministic iteration order).
- `Reaction::dcdt(c, temperature, out)` accumulates `νᵢ·r` into an output
  slice — the hook used by `tpt-proc-reactors` for its CSTR/PFR solvers.
- `Reaction::equilibrium_constant(temperature, k_ref, t_ref)` evaluates the
  van 't Hoff temperature correction from ΔH_R.
- Negative concentrations are clamped to zero before exponentiation, so
  non-integer orders stay real-valued on integration paths that overshoot.
- Missing component indices default to zero concentration rather than
  panicking.

## Installation

```toml
[dependencies]
tpt-proc-reaction = "0.1"
```

## Usage

```rust
use tpt_proc_reaction::{RateLaw, Reaction};
use std::collections::BTreeMap;

// A → B, first order in A, Ea = 60 kJ/mol.
let mut stoich = BTreeMap::new();
stoich.insert(0, -1.0);
stoich.insert(1, 1.0);
let mut orders = BTreeMap::new();
orders.insert(0, 1.0);
let reaction = Reaction::new(
    "A → B",
    stoich,
    RateLaw::Arrhenius {
        pre_exponential: 1.0e6,
        activation_energy: 60.0e3,
        orders,
    },
    -80.0e3, // J/mol, exothermic
);

let c = [1.0, 0.0]; // mol/m³
let rate = reaction.rate(&c, 400.0);
assert!(rate > 0.0); // positive rate law; A declines via stoichiometry
```

## API tour

| Item | Purpose |
|---|---|
| `RateLaw` | Enum: `Arrhenius`, `PowerLaw`, `LangmuirHinshelwood`, `MichaelisMenten` |
| `RateLaw::rate` | Net rate at concentrations `c` and temperature `T` |
| `RateLaw::rate_constant` | Temperature-dependent `k(T)` for laws that have one |
| `Reaction` | Name + stoichiometry + rate law + heat of reaction |
| `Reaction::rate` | Rate through the reaction's own law |
| `Reaction::dcdt` | Accumulates `νᵢ·r` into a `dc/dt` output slice |
| `Reaction::equilibrium_constant` | van 't Hoff `K(T)` from ΔH_R and a reference |

## Scope and limitations

- Forward rate laws only: there is no automatic reverse rate or
  micro-reversibility — equilibrium effects enter only through the van 't
  Hoff `K(T)` helper, not through the `rate` evaluation.
- Concentrations are used directly as activities; fugacity and
  activity-coefficient corrections are not applied.
- The LHHW form is the single-site inhibition expression
  `k·Πc^order/(1 + ΣKⱼcⱼ)`; multi-site coverages, dissociative adsorption
  factors, and reversible driving-force terms are not modeled.
- One rate law per reaction; pressure-based rate laws and an adsorption
  isotherm library are out of scope.

## Related crates

- Depends on [`tpt-proc-core`](../../core/tpt-proc-core) for core types.
- [`tpt-proc-reactors`](../tpt-proc-reactors) consumes `Reaction` objects
  directly in its CSTR and PFR solvers, as does the `cstr-reactor` worked
  example.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
