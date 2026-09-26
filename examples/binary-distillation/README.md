# example-binary-distillation

Shortcut design of a binary distillation. The FUG method (Fenske-Underwood-Gilliland)
gives the minimum stages, minimum reflux, and actual stage count for a
separation specified by key recoveries and a chosen reflux ratio; McCabe-Thiele
graphical stepping gives an independent stage count for the same column at
constant relative volatility. Comparing the two is the classic sanity check
taught in every separations course. The example separates a 50/50
benzene-toluene feed (α ≈ 2.4) at reflux ratio R = 1.5, and prints the
deviation between the two methods.

## Run

From the repository root:

```sh
cargo run --release -p example-binary-distillation
```

## What it shows

- Specifying the separation with `FugSpec` (relative volatility, feed
  composition, light/heavy key recoveries, actual reflux ratio).
- The FUG shortcut via `fug_shortcut(&spec)`: `min_stages` (Fenske),
  `min_reflux` (Underwood), and `actual_stages` (Gilliland).
- Independent stage counting with `mccabe_thiele_stages(&spec, x_d, x_b)`,
  including the `feed_stage` location and its own `min_reflux`.
- Quantifying the FUG vs. McCabe-Thiele deviation.

## Crates used

- [tpt-proc-distillation](../../crates/separations/tpt-proc-distillation)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
