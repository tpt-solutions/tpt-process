# example-absorption-column

Absorption of a dilute solute (think NH3 or CO2 in a scrubber) from a gas
into a lean solvent in a counter-current column of ideal stages, designed
with the Kremser equation. The absorption factor A = L/(m·G) and the stage
count N fix the fraction of solute remaining in the gas: A > 1 gives deep
recovery, A < 1 leaves most solute in the gas. The example sweeps A and N,
prints the design point, and then regenerates the rich solvent in a
stripping column using the symmetric stripping-factor form S = 1/A.

## Run

From the repository root:

```sh
cargo run --release -p example-absorption-column
```

## What it shows

- The closed-form Kremser fraction `kremser_fraction_remaining(stages, A)`
  for stage-count and absorption-factor sweeps.
- `Absorber::new(num_stages)` and `Absorber::absorb(y_in, A)` returning an
  `AbsorptionResult` (`fraction_remaining`, `solute_recovery`,
  `exiting_solute`).
- Solvent regeneration with `Absorber::strip(x_in, S)` and the symmetry
  strip(S) == absorb(S).

## Crates used

- [tpt-proc-absorption](../../crates/separations/tpt-proc-absorption)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
