# example-pinch-analysis

Pinch analysis of a four-stream heat-recovery problem. Given the hot and
cold process streams and a minimum approach temperature (ΔT_min = 10 K),
pinch analysis sets the *targets* — the minimum hot and cold utility a
heat-exchanger network could ever achieve — before any exchanger is
drawn. The example computes those targets and the pinch location, builds
the hot/cold composite curves and the grand composite curve used to place
utility levels, and synthesizes a minimum-utility process-process match
network.

## Run

From the repository root:

```sh
cargo run --release -p example-pinch-analysis
```

## What it shows

- Building the stream set with `ProcessStream::hot(t_in, t_out, cp)` and
  `ProcessStream::cold(...)` inside `PinchAnalysis::new(hots, colds, dt_min)`.
- Utility targeting and pinch location via `minimum_utilities()`
  (`min_heating_duty`, `min_cooling_duty`, `pinch_hot_temp`, `pinch_cold_temp`).
- Composite curves for graphical work: `composite_curves()`.
- Utility-level placement from `grand_composite_curve()`.
- Network synthesis with `network_synthesis()`, listing each
  process-process match and its duty.

## Crates used

- [tpt-proc-heat-network](../../crates/heat-transfer/tpt-proc-heat-network)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
