# tpt-proc-materials

> Species mixtures from `tpt-process` translated to elemental compositions in `tpt-materials`, with an NMC precursor process design.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A workspace-internal integration bridge of the [tpt-process](../../../README.md)
process-engineering suite. It connects the process side (species-level streams:
water, metal sulfates, hydroxides) to the materials-science side (elemental
compositions in `tpt-materials` via `tpt-mat-core`). **This crate is not published to
crates.io**: it depends on git dependencies pinned to substrate revisions and is
consumed as a git or path dependency inside the TPT workspace.

## Overview

Process simulation and materials science speak different languages. A process stream
carries *species* — nickel sulfate, mixed-metal hydroxide — while battery work wants
*elements*: kg of Ni, Mn, Co per kg of material. This crate provides the exact
mass-balance translation, plus a worked battery-precursor process design:

- **Formula algebra.** [`molar_mass`] holds IUPAC standard atomic weights (rounded to
  4 decimals) for the 20 elements process species actually use. [`formula`] builds a
  formula from `(element, count)` pairs and returns its composition and molar mass;
  [`formula_weight_fractions`] converts one to element weight fractions.
- **Mixture translation.** [`elemental_composition`] applies the exact mass balance
  `elements = Σ species_fraction × species_element_fractions` over a species mixture
  whose fractions must sum to 1; [`to_material_composition`] wraps the result in a
  weight-basis [`tpt_mat_core::Composition`].
- **NMC precursor design.** [`PrecursorDesign`] is the mixed-metal hydroxide
  precursor M(OH)₂ feeding NMC cathode production: metal molar ratios (e.g. the 811
  ratio), [`PrecursorDesign::molar_mass`], the [`PrecursorDesign::target_composition`]
  of the hydroxide product, the [`PrecursorDesign::feed_composition`] of the
  mixed-sulfate + water co-precipitation feed, and
  [`PrecursorDesign::cstr_volume_m3`] sizing the co-precipitation CSTR from production
  rate, residence time, and slurry solids density.

The chemistry convention is standard co-precipitation practice: metal sulfates in
water precipitate the hydroxide; sulfur stays in the mother liquor but is honestly
accounted for in the feed composition. All arithmetic is exact mass balance — no
fitted factors.

## Features

- [`molar_mass`] — IUPAC standard atomic weights (kg/kmol) for 20 process-relevant elements.
- [`formula`] / [`Formula`] — build a formula from `(element, count)` pairs; returns the element map and molar mass.
- [`formula_weight_fractions`] — element weight fractions of a single species.
- [`elemental_composition`] — exact species→element mass balance over a normalized mixture.
- [`to_material_composition`] — element fractions → weight-basis `tpt_mat_core::Composition`.
- [`PrecursorDesign`] — M(OH)₂ NMC precursor design from metal molar ratios.
- [`PrecursorDesign::target_composition`] / [`PrecursorDesign::feed_composition`] — product and mixed-sulfate-feed element compositions.
- [`PrecursorDesign::molar_mass`] / [`PrecursorDesign::cstr_volume_m3`] — precursor molar mass and co-precipitation CSTR sizing.
- `#![forbid(unsafe_code)]`.

## Installation

Not on crates.io. Consume it as a git dependency of the workspace:

```toml
[dependencies]
tpt-proc-materials = { git = "https://github.com/tpt-solutions/tpt-process" }
```

Its git dependency (`tpt-mat-core` from `tpt-solutions/tpt-materials`) is resolved
through the workspace dependency table, pinned to substrate revisions, so a pinned
revision of this repository builds reproducibly.

## Usage

```rust
use tpt_mat_core::{Composition, CompositionBasis};
use tpt_proc_materials::{elemental_composition, formula_weight_fractions, PrecursorDesign};

// NMC hydroxide precursor M(OH)2 with the 811 ratio.
let design = PrecursorDesign::new(8.0, 1.0, 1.0).unwrap();
let composition: Composition = design.target_composition().unwrap();
assert_eq!(composition.basis(), CompositionBasis::Weight);

// A mixture of species maps to elements by exact mass balance.
let nickel_sulfate = formula_weight_fractions(&[("Ni", 1), ("S", 4)]).unwrap();
let oxygen = formula_weight_fractions(&[("O", 2)]).unwrap();
let mix = vec![(nickel_sulfate, 0.5), (oxygen, 0.5)];
let elements = elemental_composition(&mix).unwrap();
assert!((elements["Ni"] + elements["S"] + elements["O"] - 1.0).abs() < 1e-9);
```

## API tour

| Item | Purpose |
|---|---|
| [`molar_mass`] | Element symbol → molar mass (kg/kmol). |
| [`Formula`] / [`formula`] | Formula input type and builder returning element counts + molar mass. |
| [`formula_weight_fractions`] | One species → element weight fractions. |
| [`elemental_composition`] | Normalized species mixture → element weight fractions. |
| [`to_material_composition`] | Element fractions → `tpt-mat-core` `Composition` (weight basis). |
| [`PrecursorDesign`] | NMC hydroxide precursor M(OH)₂ design and sizing. |

## Scope and limitations

- The element table covers the 20 built-in symbols only; anything else returns an
  `Err` from [`formula`] and friends — no periodic-table crate, no isotope handling.
- Weight-basis translations only; there is no mole-basis or volume-basis composition
  conversion here (`tpt-mat-core` owns basis semantics).
- The precursor design models composition and one sizing rule ([`PrecursorDesign::cstr_volume_m3`]);
  it does not simulate co-precipitation kinetics, crystal growth, or washing steps.
- The bridge converts static compositions; it is not wired to live stream objects of
  the flowsheet crates — you pass the numbers it should translate.

## Related crates

- [`tpt-mat-core`](https://github.com/tpt-solutions/tpt-materials) — upstream substrate: `Composition`, `CompositionBasis` (git dependency, pinned revision).
- [`tpt-proc-energy`](../tpt-proc-energy) — sibling integration bridge, pinch targets → energy-system models.
- [`tpt-proc-construction`](../tpt-proc-construction) — sibling integration bridge, PFD → construction site/project models.

Note that despite its position in the process suite, this crate's only code dependency
is `tpt-mat-core`: the caller passes compositions in, so no `tpt-proc-*` crates are
linked.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
