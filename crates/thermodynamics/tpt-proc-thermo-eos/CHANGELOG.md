# Changelog

All notable changes to `tpt-proc-thermo-eos` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- `CubicEos`: the generalized two-parameter cubic EOS with `CubicEos::peng_robinson` (u = 2, w = −1) and `CubicEos::srk` (u = 1, w = 0) constructors over a `Component` list.
- `CubicFlavor` (`PengRobinson` / `SoaveRedlichKwong`) and flavor-specific Ωa/Ωb constants and alpha-correlation `m(ω)`.
- `MixingRule` with the classical van der Waals one-fluid rule implemented; `HuronVidal` and `WongSandler` selectable but rejected at evaluation until activity-crate integration (RFC 0001).
- Symmetric binary interaction parameters via `with_binary_interaction` / `binary_interaction` (diagonal implied zero).
- `compressibility_roots`: all positive real compressibility roots at (T, P), ascending.
- `select_root`: vapor/liquid/stable root selection with a minimum-Gibbs (`ln φ`) tie-break.
- `ln_phi_at_root` and `ln_phi_mixture`: component and mixture log fugacity coefficients.
- `enthalpy_departure` and `entropy_departure` (via `EquationOfStateModel`) from analytic `da_mix_dt`; public `a_mix`, `b_mix`, and `pressure` for custom solvers.
- `pure_vapor_pressure`: saturation pressure below Tc by fugacity equality.

### Changed
- `select_root` and the vapor-pressure residual return `ThermoError::Numeric` when no compressibility root exists, instead of panicking.
