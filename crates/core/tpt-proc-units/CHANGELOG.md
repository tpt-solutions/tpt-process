# Changelog

All notable changes to `tpt-proc-units` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- The `UnitBehavior` trait: stateless `kind`/`num_inlets`/`num_outlets`/`solve` contract mapping inlet `MaterialStream`s to outlets (`Send + Sync`).
- `Mixer`: N-inlet, 1-outlet reference unit with exact flow conservation, flow-weighted outlet temperature and composition, and minimum inlet pressure.
- `Splitter`: 1-inlet, N-outlet reference unit with split-ratio normalization; outlets inherit inlet composition, temperature, and pressure.
- `UnitRegistry`: ordered (`BTreeMap`) mapping of `UnitId` to `Arc<dyn UnitBehavior>` with `register`, `get`, `len`, `is_empty`, and `execute` dispatch.
- `UnitError` taxonomy: `WrongInletCount`, `MissingStreamData`, `UnknownUnit`, `Unsupported`, `NotConverged`.
- `From<CoreError>` conversion so core errors flow into the unit error type.
- `#![forbid(unsafe_code)]` and deterministic iteration order throughout.
