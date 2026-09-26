# Changelog

All notable changes to `tpt-proc-thermo-phase` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- `FlashSolver`: the two-phase VLE calculator over a `PropertyPackage`, with `new` and the `package` accessor.
- `FlashSolver::peng_robinson(components, mixing) -> Result<Self, FlashError>`: convenience constructor building a Peng-Robinson package from a component list and mixing rule.
- `FlashSolver::pt_flash`: fixed-T/P flash by successive substitution on Wilson-seeded K-values with a bracketed Rachford-Rice bisection; EOS fugacity path when attached, modified-Raoult (activity or ideal γ) otherwise.
- `FlashSolver::ph_flash`: fixed-P, specified-enthalpy flash via outer bisection on the PT-flash enthalpy.
- `FlashSolver::bubble_point_t` and `FlashSolver::dew_point_t`: bubble/dew temperatures at fixed pressure by monotone bisection.
- `FlashResult`: temperature, pressure, vapor fraction, liquid/vapor compositions, converged K-values, iteration count, and a `degenerate` flag for pure-component saturation points.
- `FlashError` taxonomy (`NotConverged`, `InvalidInput`, `Thermo`); non-convergence is an error, never a wrong answer, and the mass balance `z = (1−β)·x + β·y` holds by construction.
- `prelude` module re-exporting `FlashSolver`, `FlashResult`, `FlashError`, and `Composition` for one-crate flash usage.

### Changed
- **Breaking:** `FlashSolver::peng_robinson` now returns `Result<Self, FlashError>` instead of panicking on invalid components; component validation is surfaced, not assumed.
