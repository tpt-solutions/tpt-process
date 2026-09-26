# Changelog

All notable changes to `tpt-proc-flowsheet` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- Public types: `FlowsheetResult`, `FlowsheetError`, `FlowsheetSolver`.
- Initial workspace release: full test suite, doctests, crate README, and CI gates.

### Added
- `FlowsheetResult.streams`: final state of every stream after the last pass, keyed by stream id (drives the `tpt-proc` CLI's JSON report).
- Convergence edge-case tests: divergent (repulsive-gain) recycle reports non-convergence without panicking, multiple independent loops converge together, far-off tear starts recover for contractive loops, zero iteration budget reports not-converged, and non-finite unit output degrades to an inlet clone instead of poisoning the tear stream.
### Changed
- Wegstein acceleration now uses the classic secant between successive (start, recomputed) tear values; previously the stored 'previous' value made the secant degenerate to NaN after the first pass, so acceleration never actually engaged.
### Fixed
- A NaN tear value can no longer read as convergence: the residual maps poisoned values to infinity (f64::max ignores NaN).
