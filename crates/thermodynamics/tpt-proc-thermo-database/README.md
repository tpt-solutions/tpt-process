# tpt-proc-thermo-database

> The built-in open chemical property database: critical constants, ideal-gas Cp, and binary interaction parameters.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-thermo-database.svg)](https://crates.io/crates/tpt-proc-thermo-database)
[![Documentation](https://docs.rs/tpt-proc-thermo-database/badge.svg)](https://docs.rs/tpt-proc-thermo-database)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The point of `tpt-process` is that property data should be a public good, not
a license line-item. This crate ships the open databank. Part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

[`ChemicalDatabase`] compiles in constants for the chemicals that cover the
bulk of teaching and pre-design work — 51 components spanning water,
alcohols (C₁–C₄, glycol), aromatics (benzene through the xylenes and
naphthalene), light hydrocarbons and paraffins (C₁–C₁₀, isobutane,
cyclohexane, butadiene, acetylene), common gases (H₂, N₂, O₂, CO, CO₂, H₂S,
SO₂, NH₃, He, Ar), and key process chemicals (ethylene oxide, DME, MTBE,
acetonitrile, THF, acetic acid, acetaldehyde). Every entry carries critical
temperature/pressure/volume, acentric factor, normal boiling point, and an
ideal-gas heat-capacity correlation.

**Data provenance and accuracy:** constants are compiled from open literature
tabulations — DIPPR-style correlations (the DIPPR-127 form
`Cp/R = A + B·T + C·T² + D·T³ + E/T²`, T in K), NIST WebBook summary values,
and Smith–Van Ness–Abbott Appendix C heat capacities. They are
**design-estimate grade**: critical constants are typically within
experimental uncertainty, and Cp fits are within a few percent over
250–1000 K. For licensed final design, validate against experiment — or
replace entries via [`ChemicalDatabase::register`]. Every entry is
overridable; nothing is hard-coded in the physics crates.

The crate also ships a table of 31 Peng-Robinson binary interaction
parameters for common pairs (typical hydrocarbon/non-hydrocarbon literature
values from Poling et al., *The Properties of Gases and Liquids*); unlisted
pairs default to 0.

**Accuracy gate:** every entry is pinned by the committed baseline
[`test-data/golden/thermodynamics/database-baseline.txt`](../../../test-data/golden/thermodynamics/database-baseline.txt);
CI fails on any unreviewed database change. Per-entry sources and
verification status are documented in
[PROVENANCE.md](PROVENANCE.md).

## Features

- [`ChemicalDatabase::builtin`] — the compiled-in reference collection
  (no I/O); [`ChemicalDatabase::empty`] for fully user-supplied data.
- [`ChemicalDatabase::get_component`] — case-insensitive lookup by name *or*
  CAS number.
- [`ChemicalDatabase::components_for`] — build an ordered
  `Vec<Component>` for a named set, reporting the first unknown name via
  `CoreError::MissingReference`.
- [`ChemicalDatabase::binary_interaction`] — symmetric kᵢⱼ lookup, 0.0 when
  unlisted.
- [`ChemicalDatabase::register`] and
  [`ChemicalDatabase::register_binary_interaction`] — override any built-in
  value or add your own components.
- [`ChemicalDatabase::component_names`], `len`, `is_empty` — inventory
  access; names are always sorted for determinism.

## Installation

```toml
[dependencies]
tpt-proc-thermo-database = "0.1"
```

## Usage

```rust
use tpt_proc_thermo_database::ChemicalDatabase;
use tpt_proc_thermo_eos::CubicEos;

let db = ChemicalDatabase::builtin();
let water = db.get_component("water").unwrap();
assert_eq!(water.cas_number, "7732-18-5");

// Build a PR EOS directly from the database.
let components = db.components_for(&["water", "methanol"]).unwrap();
let eos = CubicEos::peng_robinson(components)
    .with_binary_interaction(0, 1, db.binary_interaction("water", "methanol"));
assert_eq!(eos.components().len(), 2);
```

## API tour

| Item | Purpose |
|---|---|
| `ChemicalDatabase` | Name/CAS-keyed component store plus a kᵢⱼ table |
| `ChemicalDatabase::builtin` | The compiled-in 51-component reference set |
| `ChemicalDatabase::get_component` | Lookup by case-insensitive name or CAS |
| `ChemicalDatabase::components_for` | Ordered component list for a named set |
| `ChemicalDatabase::binary_interaction` | Symmetric PR kᵢⱼ (default 0.0) |
| `ChemicalDatabase::register` / `register_binary_interaction` | Override or extend the databank |

## Scope and limitations

- **51 components** — a teaching and pre-design set, not a commercial
  databank with thousands of species; bring your own `Component`s for
  anything exotic.
- Newly added entries (see [PROVENANCE.md](PROVENANCE.md)) use ideal-gas Cp
  frozen at 298.15 K pending a DIPPR-127 refit — fine near ambient, weak
  at high reduced temperature.
- **Design-estimate grade**: Cp correlations are accurate to a few percent
  over roughly 250–1000 K; do not use for licensor-grade final design without
  validation.
- Pure-component constants and EOS kᵢⱼ only: no transport properties
  (viscosity, conductivity), no liquid density or surface tension
  tabulations, no latent heats, no antoine constants.
- No persistence: the database is compiled in or built at runtime; there is
  no file/JSON loading in this crate (and no `serde` support).
- Binary interaction parameters are Peng-Robinson-specific values; they are
  not parameterized per model.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — upstream: error types.
- [`tpt-proc-thermo-core`](../tpt-proc-thermo-core) — upstream: the
  `Component` and `CpCorrelation` types stored here.
- [`tpt-proc-thermo-eos`](../tpt-proc-thermo-eos) — downstream: consumes the
  kᵢⱼ table via `with_binary_interaction` (as shown above).
- [`tpt-proc-thermo-phase`](../tpt-proc-thermo-phase) — downstream: the flash
  solver examples and tests resolve components through this database.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
