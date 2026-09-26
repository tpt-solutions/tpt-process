# example-water-treatment-plant

A complete surface-water treatment train, from raw river water to
potable-quality product: coagulation, flocculation, sedimentation,
filtration, ion exchange for hardness, reverse osmosis for TDS, and
chlorine disinfection credited through a CT value. The example prints the
full water quality (turbidity, TDS, hardness, coliform) before and after
the train, then walks the train process by process so each unit's effect
on turbidity and TDS is visible.

## Run

From the repository root:

```sh
cargo run --release -p example-water-treatment-plant
```

## What it shows

- A raw-water state: `WaterQuality::new(turbidity_ntu, tds_mg_l, hardness_mg_l, coliform_cfu_ml)`.
- Composing a treatment train from the `Process` enum
  (`Coagulation`, `Flocculation`, `Sedimentation`, `Filtration`,
  `IonExchange`, `ReverseOsmosis`, `Disinfection`) in `Train::new(...)`.
- Simulating the whole chain with `Train::simulate(&raw)`.
- Stepping unit by unit with the free function `apply(process, &quality)`
  to trace turbidity and TDS through the plant.

## Crates used

- [tpt-proc-water](../../crates/applications/tpt-proc-water)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
