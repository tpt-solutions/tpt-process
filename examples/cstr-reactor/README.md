# example-cstr-reactor

Steady-state reactor design for an exothermic first-order reaction A → B.
At the same residence time a plug-flow reactor always out-converts a
perfectly mixed CSTR for positive-order kinetics — the example quantifies
that gap against the analytic formulas (kτ/(1+kτ) for the CSTR,
1 − e^(−kτ) for the PFR) and then repeats the CSTR run with the adiabatic
energy balance enabled to show the outlet temperature rise.

## Run

From the repository root:

```sh
cargo run --release -p example-cstr-reactor
```

## What it shows

- Defining kinetics with `Reaction::new(...)` and
  `RateLaw::Arrhenius { pre_exponential, activation_energy, orders }`
  over a `BTreeMap` stoichiometry.
- Assembling a simulation with `ReactorSim::new(vec![reaction])`.
- Solving a 2 m³ CSTR with `ReactorSim::cstr(&CstrSpec { volume }, ...)`
  and a PFR with `ReactorSim::pfr(&PfrSpec { volume }, ...)` at τ = 2 s,
  reading `conversion` and `temperature` from the results.
- Enabling the adiabatic energy balance (last `adiabatic` flag) and
  comparing the outlet temperature to ΔT·X·(−ΔH)·C₀/(ρcp).

## Crates used

- [tpt-proc-reaction](../../crates/reaction/tpt-proc-reaction)
- [tpt-proc-reactors](../../crates/reaction/tpt-proc-reactors)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
