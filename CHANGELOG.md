# Changelog

All notable changes to `tpt-process` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Releases follow a 6-week cadence (see `CONTRIBUTING.md`).

## [Unreleased]

### Added

- Project specification (`spec.txt`) and roadmap (`todo.md`).
- Repository governance: dual MIT OR Apache-2.0 licensing, contribution,
  security, and code-of-conduct policies.
- RFC process with initial RFCs: Peng-Robinson EOS (0001), flash
  calculations (0002), flowsheet solver (0003), pinch analysis (0004).
- CI: formatting, clippy, tests, license chain enforcement (`cargo deny`),
  benchmarks, documentation publishing, and release tooling.
- `tpt-proc-core` — material streams, compositions, phase states, unit
  operations, flowsheets, connections, and the property-package trait.
- `tpt-proc-topology` — process graph analysis: recycle detection,
  topological ordering, tear-stream selection, connected components.
- `tpt-proc-units` — unit-operation behavior trait, ports, mixer/splitter
  reference models, and the unit registry.
- `tpt-proc-wasm` — WebAssembly bindings (flash calculator, heat-exchanger
  rating, database lookup); builds for native and `wasm32-unknown-unknown`.
- Thermodynamics: `tpt-proc-thermo-core` (property framework, DIPPR-127
  ideal-gas Cp), `tpt-proc-thermo-eos` (Peng-Robinson and SRK with van der
  Waals mixing, analytic cubic roots, fugacity-equality saturation
  pressure), `tpt-proc-thermo-activity` (Wilson, NRTL, UNIQUAC, UNIFAC),
  `tpt-proc-thermo-phase` (PT/PH flash, Rachford-Rice, bubble/dew points),
  `tpt-proc-thermo-database` (~27-component open databank + kᵢⱼ table).
- Fluid flow: `tpt-proc-fluid` (Colebrook-White, Darcy-Weisbach),
  `tpt-proc-pumps`, `tpt-proc-compressors`, `tpt-proc-valves`,
  `tpt-proc-network` (Hardy Cross with reservoir pseudo-cycles, nodal
  Newton-Raphson, Linear Theory).
- Heat transfer: `tpt-proc-heat-transfer` (convection correlations, wall
  resistances), `tpt-proc-heat-exchangers` (LMTD + Bowman F, ε-NTU,
  rating/sizing), `tpt-proc-heat-network` (problem-table pinch analysis,
  composite/grand composite curves, network synthesis),
  `tpt-proc-fired-equipment` (combustion stoichiometry, stack-loss
  efficiency).
- Separations: `tpt-proc-distillation` (FUG, McCabe-Thiele, Lewis-Sorel
  MESH), `tpt-proc-absorption` (Kremser), `tpt-proc-extraction`,
  `tpt-proc-membranes`, `tpt-proc-crystallization`.
- Reaction engineering: `tpt-proc-reaction` (Arrhenius, power-law,
  Langmuir-Hinshelwood, Michaelis-Menten), `tpt-proc-reactors` (CSTR and
  PFR with RK4 and adiabatic coupling), `tpt-proc-catalysis` (Thiele
  modulus, effectiveness factors, deactivation).
- Simulation: `tpt-proc-flowsheet` (sequential modular with tear streams
  and Wegstein acceleration), `tpt-proc-mass-balance` (weighted
  reconciliation), `tpt-proc-energy-balance`, `tpt-proc-optimization`
  (golden section, Nelder-Mead, gradient descent).
- Applications: `tpt-proc-hvac` (psychrometrics), `tpt-proc-water`
  (treatment trains), `tpt-proc-hydrogen` (electrolysis, SMR, ATR),
  `tpt-proc-pharma`, `tpt-proc-refining`.
- Data & deliverables: `tpt-proc-database` (unit-annotated store with CSV
  round-trip), `tpt-proc-pfd` (SVG/DOT/Mermaid export),
  `tpt-proc-economics` (six-tenths capex, NPV, IRR, payback, levelized
  cost).
- Seven worked examples, four criterion benchmark suites, golden reference
  data under `test-data/golden/`, and an mdBook user guide under
  `docs/book/`.

### Added (2026-09 platform review)

- Per-crate documentation: every crate under `crates/` now ships a
  comprehensive `README.md` (overview, features, verified usage example,
  API tour, scope and limitations, related crates) and a `CHANGELOG.md`,
  plus crates.io publish metadata (`description`, `keywords`,
  `categories`, `readme`).
- `tpt-proc` CLI (`crates/cli/tpt-proc`): config-driven flowsheet runner —
  TOML config in (`[thermo]`, `[[streams]]`, `[[units]]` for
  mixer/splitter/flash/heater/cooler/valve/pump, `[[connections]]`),
  JSON results out, with recycle-loop support and a worked demo config.
- `serde` support in `tpt-proc-core` (opt-in `serde` feature): streams,
  compositions, unit operations, connections, and flowsheets round-trip
  through JSON/TOML; the property package is skipped and must be
  re-attached.
- Chemical database expansion: 27 → 51 components with per-entry source
  documentation (`PROVENANCE.md`) and an accuracy gate — a committed
  baseline (`test-data/golden/thermodynamics/database-baseline.txt`)
  fails CI on any unreviewed database change.
- Documentation book: new pages for the application crates (HVAC, water,
  hydrogen, pharma, refining), dynamics & control, economics, CLI usage,
  PFD export ("Visualize your flowsheet"), cross-repo integration,
  validation results from the golden data, and a WASM playground page
  with a standalone HTML demo.
- CI: `cargo deny check advisories` (RustSec) job in `license.yml`; a
  coverage job (`cargo llvm-cov` + Codecov) in `ci.yml`; `codecov.yml`.
- `templates/unit-op-crate`: a `cargo-generate` template scaffolding a new
  `UnitBehavior` unit-operation crate with house rules and tests.
- Workspace: `default-members` excludes the git-dependent integration
  crates so daily commands work without network access (CI still covers
  them via `--workspace`).
- Eight new worked examples (pipe network, pump pipeline, absorption,
  extraction, membranes, crystallizer, PFR/packed bed, project
  economics) and READMEs for every example crate.

### Changed

- **Breaking:** `FlashSolver::peng_robinson` returns
  `Result<Self, FlashError>` instead of panicking; `benzene_toluene_package`
  returns `Result<Arc<PropertyPackage>, ThermoError>`.
- Wegstein tear acceleration uses the classic secant between successive
  (start, recomputed) values — the previous bookkeeping made the secant
  degenerate to NaN after the first pass, so acceleration never engaged.
- Library code is panic-free on the paths flagged by the 2026-09 review:
  internal `.expect()` panics in `tpt-proc-topology`, `tpt-proc-heat-network`,
  `tpt-proc-optimization`, `tpt-proc-materials`, and `tpt-proc-thermo-eos`
  are replaced with total lookups or propagated typed errors.
- README: badges (CI, license check, crates.io, docs.rs, MSRV), a
  copy-pasteable 5-minute quickstart at the top, and an honest
  Stable/Beta/Alpha/Integration tiering of the crate status table.

### Fixed

- A NaN tear value can no longer read as converged in
  `tpt-proc-flowsheet` (`f64::max` ignores NaN; the residual now maps
  poisoned values to infinity).
- Vapor-pressure root selection keeps the true vapor root (largest)
  after the panic-free refactor; convergence edge cases (divergent
  loops, multiple recycle loops, ill-conditioned tear starts, zero
  iteration budgets) are pinned by new tests.
