# Changelog

All notable changes to `tpt-proc-mass-balance` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- Public types: `BalanceReport`, `Reconciliation`.
- Public functions: `overall_balance`, `component_balance`, `reconcile_flows`.
- Initial workspace release: full test suite, doctests, crate README, and CI gates.
