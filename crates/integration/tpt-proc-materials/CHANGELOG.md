# Changelog

All notable changes to `tpt-proc-materials` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- Public types: `PrecursorDesign`.
- Public functions: `molar_mass`, `formula`, `formula_weight_fractions`, `elemental_composition`, `to_material_composition`.
- Initial workspace release: full test suite, doctests, crate README, and CI gates.

### Changed
- `formula_weight_fractions` computes fractions in a single validated pass (no internal `.expect`); `PrecursorDesign` uses atomic-weight constants shared with the `molar_mass` table. No behavior change.
