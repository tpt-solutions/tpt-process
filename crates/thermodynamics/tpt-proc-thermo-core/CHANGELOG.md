# Changelog

All notable changes to `tpt-proc-thermo-core` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- `Component`: pure-component constants (molecular weight, critical T/P/V, acentric factor, normal boiling point, ideal-gas Cp) with `new`, `validate`, `reduced_temperature`, `reduced_pressure`, and `critical_compressibility`.
- `CpCorrelation`: DIPPR-127 (`Cp/R = A + B·T + C·T² + D·T³ + E/T²`) and constant forms with `cp_ig` plus analytic `enthalpy_ig`/`entropy_ig` integrals.
- `PropertyPackage`: components + optional EOS + optional activity model, with `new`, `with_eos`, `with_activity`, `with_reference_temperature`, `cp_ig_mixture`, `vapor_pressure` (EOS saturation or Trouton–Clausius fallback), `trouton_vapor_pressure`, `activity_coefficients`, and `rackett_liquid_density`.
- The `EquationOfStateModel` trait: `ln_fugacity_coefficients`, `compressibility`, `enthalpy_departure`, `entropy_departure`, `pure_vapor_pressure`.
- The `ActivityCoefficientModel` trait for liquid excess-Gibbs models.
- `Phase`, `PhaseSelection` (`Vapor`/`Liquid`/`Stable`), and `PropertyMethods` enums for phase-root and method selection.
- `ThermoError` (`StateOutOfDomain`, `Numeric`, `Unsupported`, …) and the crate-wide `Result` alias.
- `R`: the universal gas constant, 8.314 462 618 J/(mol·K) (CODATA 2018).
