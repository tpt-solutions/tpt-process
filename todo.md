# tpt-process — Project Checklist

**Organization:** TPT Solutions | **Repo:** `tpt-process` | **License:** MIT OR Apache-2.0

---

## Phase 0 — Repo Setup & Governance
- [ ] Initialize git repo and `Cargo.toml` workspace skeleton
- [ ] `LICENSE-MIT`
- [ ] `LICENSE-APACHE`
- [ ] `README.md` (from spec Section 13 template)
- [ ] `CONTRIBUTING.md`
- [ ] `SECURITY.md`
- [ ] `CODE_OF_CONDUCT.md`
- [ ] `CHANGELOG.md`
- [ ] `deny.toml` (MIT chain enforcement, copyleft = deny)
- [ ] `rustfmt.toml`, `clippy.toml`
- [ ] `.github/workflows/ci.yml`
- [ ] `.github/workflows/license.yml`
- [ ] `.github/workflows/benchmark.yml`
- [ ] `.github/workflows/docs.yml`
- [ ] `.github/workflows/release.yml`
- [ ] `.github/ISSUE_TEMPLATE/`
- [ ] `.github/PULL_REQUEST_TEMPLATE.md`
- [ ] `rfcs/0001-peng-robinson-eos.md`
- [ ] `rfcs/0002-flash-calculations.md`
- [ ] `rfcs/0003-flowsheet-solver.md`
- [ ] `rfcs/0004-pinch-analysis.md`
- [ ] **Milestone:** Repo scaffolding complete, CI green on empty workspace

## Phase 1 — Foundation (Months 1-3)
- [ ] `tpt-proc-core` (MaterialStream, Composition, PhaseState, UnitOperation, Flowsheet, Connection)
- [ ] `tpt-proc-topology` (ProcessGraph: recycles, topological order, tear streams, connected components)
- [ ] `tpt-proc-units` (shared unit operation scaffolding)
- [ ] `tpt-proc-wasm` (crate scaffold only)
- [ ] **Milestone:** Parse a simple PFD, build process graph

## Phase 2 — Thermodynamics (Months 4-6)
- [ ] `tpt-proc-thermo-core` (PropertyPackage, Component, PropertyMethods)
- [ ] `tpt-proc-thermo-eos` (Peng-Robinson, SRK, mixing rules)
- [ ] `tpt-proc-thermo-database` (built-in chemical database)
- [ ] **Milestone:** Calculate vapor pressures and fugacities for common chemicals

## Phase 3 — Phase Equilibrium (Months 7-9)
- [ ] `tpt-proc-thermo-activity` (Wilson, NRTL, UNIQUAC, UNIFAC)
- [ ] `tpt-proc-thermo-phase` (FlashSolver: PT flash, Rachford-Rice, bubble/dew point)
- [ ] **Milestone:** Solve binary and multicomponent flash problems

## Phase 4 — Fluid Flow (Months 10-12)
- [ ] `tpt-proc-fluid` (pipe flow, Reynolds, friction factor, Darcy-Weisbach)
- [ ] `tpt-proc-pumps` (pump curves, operating point, affinity laws)
- [ ] `tpt-proc-compressors`
- [ ] `tpt-proc-valves`
- [ ] `tpt-proc-network` (Hardy Cross, Newton-Raphson, linear theory solvers)
- [ ] **Milestone:** Solve pipe network with Hardy Cross

## Phase 5 — Heat Transfer (Months 13-15)
- [ ] `tpt-proc-heat-transfer` (convection correlations, Nusselt/Dittus-Boelter)
- [ ] `tpt-proc-heat-exchangers` (LMTD, NTU-effectiveness, rating/sizing)
- [ ] `tpt-proc-heat-network` (pinch analysis, composite curves, HEN synthesis)
- [ ] `tpt-proc-fired-equipment`
- [ ] **Milestone:** Rate and size shell-and-tube heat exchangers

## Phase 6 — Separations (Months 16-18)
- [ ] `tpt-proc-distillation` (McCabe-Thiele, FUG shortcut, rigorous MESH)
- [ ] `tpt-proc-absorption` (Kremser equation)
- [ ] `tpt-proc-extraction`
- [ ] `tpt-proc-membranes`
- [ ] `tpt-proc-crystallization`
- [ ] **Milestone:** Design binary distillation column with FUG shortcut

## Phase 7 — Reaction Engineering (Months 19-21)
- [ ] `tpt-proc-reaction` (Arrhenius, power law, Langmuir-Hinshelwood, Michaelis-Menten kinetics)
- [ ] `tpt-proc-reactors` (Batch, CSTR, PFR, packed bed, fluidized bed)
- [ ] `tpt-proc-catalysis`
- [ ] **Milestone:** Simulate CSTR and PFR with exothermic reaction

## Phase 8 — Flowsheet & Core Applications (Months 22-24)
- [ ] `tpt-proc-flowsheet` (sequential modular + equation-oriented solvers)
- [ ] `tpt-proc-mass-balance` (overall/component balance, reconciliation)
- [ ] `tpt-proc-energy-balance`
- [ ] `tpt-proc-optimization`
- [ ] `tpt-proc-hvac` (psychrometrics)
- [ ] `tpt-proc-water` (treatment train: coagulation → RO)
- [ ] `tpt-proc-hydrogen` (electrolysis, SMR, ATR)
- [ ] **Milestone:** Solve complete process flowsheet with recycle

## Phase 9 — Dynamics, Remaining Applications & Data
- [ ] `tpt-proc-dynamics`
- [ ] `tpt-proc-control`
- [ ] `tpt-proc-pharma`
- [ ] `tpt-proc-refining`
- [ ] `tpt-proc-database` (general process data store)
- [ ] `tpt-proc-pfd` (process flow diagram representation/export)
- [ ] `tpt-proc-economics`
- [ ] `tpt-proc-wasm` full bindings (WasmFlashCalculator, WasmHeatExchanger)
- [ ] **Milestone:** Dynamic simulation + economics on a sample plant, deployed to WASM

## Cross-Cutting
- [ ] `examples/flash-calculation`
- [ ] `examples/heat-exchanger-rating`
- [ ] `examples/binary-distillation`
- [ ] `examples/cstr-reactor`
- [ ] `examples/pinch-analysis`
- [ ] `examples/hvac-psychrometrics`
- [ ] `examples/water-treatment-plant`
- [ ] `test-data/golden/thermodynamics/*` (PR vapor pressure, NRTL activity, flash VLE, bubble/dew)
- [ ] `test-data/golden/fluid-flow/*` (pipe pressure drop, pump operating point, Hardy Cross)
- [ ] `test-data/golden/heat-transfer/*` (LMTD, NTU, pinch analysis)
- [ ] `test-data/golden/separations/*` (McCabe-Thiele, FUG, Kremser)
- [ ] `test-data/golden/reactors/*` (CSTR conversion, PFR profile, Arrhenius rate)
- [ ] `test-data/golden/applications/*` (psychrometrics, water treatment)
- [ ] Cross-repo integration: `tpt-energy` (heat integration, utility optimization)
- [ ] Cross-repo integration: `tpt-materials` (chemical manufacturing processes)
- [ ] Cross-repo integration: `tpt-medical` (pharmaceutical batch processes)
- [ ] Cross-repo integration: `tpt-construction` (plant layout and construction)
- [ ] Standards support: ASPEN/HYSYS import, CAPE-OPEN, P&ID
- [ ] `docs/book`, `docs/api` publishing pipeline
