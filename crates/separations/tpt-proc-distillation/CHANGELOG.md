# Changelog

All notable changes to `tpt-proc-distillation` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- Public types: `FugSpec`, `FugResult`, `MccabeResult`, `ColumnSpec`, `MeshResult`.
- Public functions: `fenske_min_stages`, `underwood_min_reflux`, `gilliland_stages`, `fug_shortcut`, `benzene_toluene_package`, `relative_volatility_from_package`, `package_k_values`.
- Initial workspace release: full test suite, doctests, crate README, and CI gates.

### Changed
- **Breaking:** `benzene_toluene_package` now returns `Result<Arc<PropertyPackage>, ThermoError>` instead of panicking.
