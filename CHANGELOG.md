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
