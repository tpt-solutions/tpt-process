# Changelog

All notable changes to `tpt-proc` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- `tpt-proc run <config.toml> [-o results.json]`: config-driven flowsheet
  runner over the sequential-modular solver (TOML in, JSON out).
- `tpt-proc components`: lists the built-in chemical database.
- TOML config schema: `[flowsheet]`, `[thermo]` (components + EOS choice
  `peng-robinson`/`srk`/`ideal`), `[[streams]]`, `[[units]]` (mixer,
  splitter, flash, heater, cooler, valve, pump), `[[connections]]`.
- Behavior adapters: PT-flash-backed `flash` (vapor/liquid two-outlet),
  `heater`/`cooler` (target outlet temperature), `valve` (outlet pressure),
  and `pump` (pressure rise) over `tpt-proc-thermo-phase`; mixer and
  splitter from `tpt-proc-units`.
- Automatic zero-flow placeholder streams for connection-only stream ids,
  seeded from the first declared stream so first-pass tear execution
  behaves deterministically.
- Library API `run_config` / `run_config_str` returning the typed
  `FlowsheetResult` for embeddable use.
- `configs/recycle-demo.toml`: verified recycle-loop demo (100 mol/s fresh
  feed, 20 % recycle, converges to 125 mol/s through the mixer).
- Integration-style tests: recycle convergence, flash split with
  composition enrichment, missing-thermo and unknown-component errors.
