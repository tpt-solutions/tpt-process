# Changelog

All notable changes to `tpt-proc-thermo-activity` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- `Nrtl` and `NrtlParams`: electrolyte-free NRTL activity coefficients with temperature-independent τ and non-randomness α (`NrtlParams::binary`, `NrtlParams::set`, `Nrtl::from_params`, `gamma`).
- `Wilson` and `WilsonParams`: Wilson's local-composition model (miscible systems only) with `binary` pair parameters.
- `Uniquac` and `UniquacParams`: UNIQUAC combinatorial + residual model (Abrams & Prausnitz, 1975).
- `Unifac`, `UnifacTable`, and `GroupCount`: original UNIFAC group-contribution model (Fredenslund, Jones & Prausnitz, 1975) with `UnifacTable::builtin_subset` (CH2, ACH, OH, CH3OH, H2O demonstration table) and table-agnostic evaluation via `Unifac::new`/`with_table`.
- All models implement `tpt_proc_thermo_core::ActivityCoefficientModel` for attachment to a `PropertyPackage` (modified-Raoult K-value path `Kᵢ = γᵢ·P_sat,ᵢ/P`).
- `PairMap`: shared `BTreeMap<(usize, usize), f64>` parameter-map type keyed by component index pairs.
- `excess_gibbs_over_rt`: G^E/RT (= Σ xᵢ ln γᵢ) helper for mixing-rule coupling and test invariants.
- Composition validation (finite, non-negative, sums to 1) returning `ThermoError` instead of panicking; `#![forbid(unsafe_code)]`.
