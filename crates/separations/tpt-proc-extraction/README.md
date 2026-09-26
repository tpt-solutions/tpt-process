# tpt-proc-extraction

> Single-stage and crosscurrent liquid-liquid extraction on a constant distribution coefficient.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-extraction.svg)](https://crates.io/crates/tpt-proc-extraction)
[![Documentation](https://docs.rs/tpt-proc-extraction/badge.svg)](https://docs.rs/tpt-proc-extraction)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-extraction` computes solute recovery in liquid-liquid extraction
contactors — one equilibrium stage or a multistage crosscurrent battery — and
is part of the [tpt-process](../../../README.md) process-engineering suite.

## Overview

The crate models dilute extraction with a constant distribution coefficient:
the solute distributes between the raffinate and solvent phases as
`Y = K·X`, where `X` and `Y` are mass-of-solute per mass of raffinate and
solvent respectively, and the carrier and solvent flows are treated as
constant. A single equilibrium stage splits the solute exactly by the
stage balance `X = M/(F + K·S)`; splitting the total solvent into equal
portions across `n` crosscurrent stages recovers strictly more solute than
one contact with all the solvent — the classic staged-contacting result —
and reduces in closed form to the per-stage extraction factor
`E = K·S/F` raised to the `n`-th power: `(1/(1+E))^n`.

Every routine closes its solute mass balance by construction: the raffinate
and extract amounts returned by `single_stage` and
`Extractor::crosscurrent` always sum to the entering solute.

## Features

- `single_stage(feed_mass, solute_mass, solvent_mass, k)` splits the feed
  solute between raffinate and extract for one equilibrium stage.
- `Extractor::new(num_stages)` builds a crosscurrent extraction battery.
- `Extractor::crosscurrent(feed_mass, solute_mass, total_solvent, k)`
  distributes the solvent equally across stages and returns
  `(raffinate_solute, total_extract_solute)`.
- `fraction_remaining(stages, extraction_factor)` gives the closed form
  `(1/(1+E))^n` for the per-stage factor `E = K·S/F`, consistent with the
  stage-by-stage `crosscurrent` result.
- Degenerate inputs are damped, not panicked: non-positive `K` and zero
  solvent are clamped to small positive values, so results stay finite.
- Mass-balance closure is exact to floating-point round-off in both the
  staged and closed-form paths.

## Installation

```toml
[dependencies]
tpt-proc-extraction = "0.1"
```

## Usage

```rust
use tpt_proc_extraction::{single_stage, Extractor};

// 100 kg feed with 10 kg solute, K = 8, equal solvent mass:
let (raffinate_solute, extract_solute) =
    single_stage(100.0, 10.0, 100.0, 8.0);
assert!((raffinate_solute + extract_solute - 10.0).abs() < 1e-9);
assert!(raffinate_solute < extract_solute);

// Three crosscurrent stages with a third of the solvent each.
let (multi_raf, _) = Extractor::new(3).crosscurrent(100.0, 10.0, 300.0, 8.0);
assert!(multi_raf < single_stage(100.0, 10.0, 300.0, 8.0).0);
```

## API tour

| Item | Purpose |
|---|---|
| `Extractor` | Crosscurrent battery with `num_stages` equilibrium stages |
| `Extractor::crosscurrent` | Staged contact with the solvent split equally per stage |
| `single_stage` | One equilibrium stage; returns `(raffinate, extract)` solute |
| `fraction_remaining` | Closed form `(1/(1+E))^n` on a per-stage extraction factor |

## Scope and limitations

- Dilute systems only: carrier and solvent flows are treated as constant
  (no mutual solubility of carrier and solvent, no solvent transfer into the
  raffinate).
- Crosscurrent contacting only — there is no countercurrent cascade with a
  single solvent stream, and no refluxed (extract-reflux) extractor.
- The distribution coefficient `K` is a user-supplied constant; no ternary
  phase-equilibrium data, tie lines, or plait-point behavior are modeled.
- Mass basis throughout (kg); no energy balance and no stage efficiency.

## Related crates

- Depends on [`tpt-proc-core`](../../core/tpt-proc-core) for core stream
  types.
- Sits in the separations domain alongside `tpt-proc-distillation`,
  `tpt-proc-absorption`, `tpt-proc-membranes`, and `tpt-proc-crystallization`.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
