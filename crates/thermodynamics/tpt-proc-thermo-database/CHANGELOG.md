# Changelog

All notable changes to `tpt-proc-thermo-database` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- Expanded the built-in database from 27 to 51 components (He, Ar, SO₂,
  acetylene, butadiene, isobutane, cyclohexane, n-nonane, n-decane, acetic
  acid, ethylene glycol, ethylene oxide, DME, MTBE, acetaldehyde,
  1-butanol, isopropanol, ethylbenzene, o/m/p-xylene, acetonitrile, THF,
  naphthalene), sourced per [PROVENANCE.md](PROVENANCE.md).
- `PROVENANCE.md`: per-entry source and verification documentation.
- Accuracy gate: `database_matches_reviewed_baseline` test pins the
  database against `test-data/golden/thermodynamics/database-baseline.txt`;
  changes fail CI until consciously reviewed and regenerated.
- Physical-plausibility sanity test over every database entry.
- `ChemicalDatabase::builtin`: a compiled-in (no I/O) reference collection of 27 components — water, alcohols, aromatics, C1–C8 hydrocarbons, light gases — covering teaching and pre-design work.
- Pure-component data per entry: critical temperature/pressure/volume, acentric factor, normal boiling point, and DIPPR-127 ideal-gas Cp correlations (compiled from open literature: NIST WebBook summaries, Smith–Van Ness–Abbott Appendix C).
- `ChemicalDatabase::get_component`: case-insensitive lookup by name or CAS number.
- `ChemicalDatabase::components_for`: ordered `Vec<Component>` for a named set, reporting unknown names via `CoreError::MissingReference`.
- 31 built-in Peng-Robinson binary interaction parameters for common pairs (Poling et al., *The Properties of Gases and Liquids*), with unlisted pairs defaulting to 0: `binary_interaction` and `register_binary_interaction`.
- `ChemicalDatabase::register`: override any built-in entry or add user-supplied components.
- `ChemicalDatabase::empty`, `component_names`, `len`, `is_empty` for building and inventorying user databases.
