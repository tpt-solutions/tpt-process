# example-pfr-packed-bed

Reaction engineering of a first-order gas reaction A → B in three acts.
First, the ideal-reactor comparison at equal residence time: CSTR
conversion kτ/(1+kτ) vs. PFR conversion 1 − e^(−kτ). Second, the packed
bed: internal diffusion in 5 mm catalyst pellets (Thiele modulus φ =
R·√(k/Deff)) derates the rate constant through the effectiveness factor
η, and the bed is re-run as a PFR at k_eff = η·k. Third, aging: the
catalyst's activity after a year of coking under exponential and
power-law deactivation models. (The reactors crate implements ideal
CSTR/PFR only; the packed bed is represented through the catalysis
crate's pellet-diffusion correction.)

## Run

From the repository root:

```sh
cargo run --release -p example-pfr-packed-bed
```

## What it shows

- Kinetics from `tpt-proc-reaction`: `Reaction::new` with
  `RateLaw::Arrhenius` over a BTreeMap stoichiometry/orders.
- `ReactorSim::cstr(&CstrSpec { volume }, ...)` and
  `ReactorSim::pfr(&PfrSpec { volume }, ...)` conversions against the
  analytic limits.
- Pellet diffusion from `tpt-proc-catalysis`: `thiele_modulus(R, k,
  Deff)`, `effectiveness_factor(phi, PelletGeometry::Sphere, order)`, and
  the derated packed-bed PFR at k_eff = η·k.
- Catalyst aging with `deactivation_factor(kd, t, DeactivationModel)`
  for `Exponential` and `PowerLaw` models.

## Crates used

- [tpt-proc-reaction](../../crates/reaction/tpt-proc-reaction)
- [tpt-proc-reactors](../../crates/reaction/tpt-proc-reactors)
- [tpt-proc-catalysis](../../crates/reaction/tpt-proc-catalysis)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
