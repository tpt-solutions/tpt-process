# example-liquid-extraction

Liquid-liquid extraction of a solute from a 100 kg feed into a solvent
with distribution coefficient K = Y/X = 8 (dilute system, constant flows).
One big equilibrium contact is compared against a crosscurrent battery
that splits the same total solvent equally over N stages — staging wins
because each fresh-solvent contact keeps the driving force up. The
closed-form fraction remaining (1/(1+E))^N with per-stage extraction
factor E = K·S_stage/F is cross-checked against the stage-by-stage mass
balance.

## Run

From the repository root:

```sh
cargo run --release -p example-liquid-extraction
```

## What it shows

- A single equilibrium stage: `single_stage(feed, solute, solvent, K)`
  returning (raffinate solute, extract solute) with an exact mass balance.
- A crosscurrent battery: `Extractor::new(stages).crosscurrent(...)` for
  N = 1, 2, 3, 5 at fixed total solvent.
- The closed form `fraction_remaining(stages, E)` validated against the
  staged balance.
- Why crosscurrent staging beats a single contact of equal total solvent.

## Crates used

- [tpt-proc-extraction](../../crates/separations/tpt-proc-extraction)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
