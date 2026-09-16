# tpt-process — Project Checklist

**Organization:** TPT Solutions | **Repo:** `tpt-process` | **License:** MIT OR Apache-2.0

**Status note (2026-09):** Phases 0–9 are implemented and green
(`cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace` — 290+
tests incl. doctests). Cross-repo integration and external-standards
import remain open by design: they depend on counterpart repositories and
third-party interface specifications.

---

## Phase 0 — Repo Setup & Governance
- [x] Initialize git repo and `Cargo.toml` workspace skeleton
- [x] `LICENSE-MIT`
- [x] `LICENSE-APACHE`
- [x] `README.md` (from spec Section 13 template)
- [x] `CONTRIBUTING.md`
- [x] `SECURITY.md`
- [x] `CODE_OF_CONDUCT.md`
- [x] `CHANGELOG.md`
- [x] `deny.toml` (MIT chain enforcement, copyleft = deny)
- [x] `rustfmt.toml`, `clippy.toml`
- [x] `.github/workflows/ci.yml`
- [x] `.github/workflows/license.yml`
- [x] `.github/workflows/benchmark.yml`
- [x] `.github/workflows/docs.yml`
- [x] `.github/workflows/release.yml`
- [x] `.github/ISSUE_TEMPLATE/`
- [x] `.github/PULL_REQUEST_TEMPLATE.md`
- [x] `rfcs/0001-peng-robinson-eos.md`
- [x] `rfcs/0002-flash-calculations.md`
- [x] `rfcs/0003-flowsheet-solver.md`
- [x] `rfcs/0004-pinch-analysis.md`
- [x] **Milestone:** Repo scaffolding complete, CI green on empty workspace

## Phase 1 — Foundation (Months 1-3)
- [x] `tpt-proc-core` (MaterialStream, Composition, PhaseState, UnitOperation, Flowsheet, Connection)
- [x] `tpt-proc-topology` (ProcessGraph: recycles, topological order, tear streams, connected components)
- [x] `tpt-proc-units` (shared unit operation scaffolding)
- [x] `tpt-proc-wasm` (crate scaffold only)
- [x] **Milestone:** Parse a simple PFD, build process graph

## Phase 2 — Thermodynamics (Months 4-6)
- [x] `tpt-proc-thermo-core` (PropertyPackage, Component, PropertyMethods)
- [x] `tpt-proc-thermo-eos` (Peng-Robinson, SRK, mixing rules)
- [x] `tpt-proc-thermo-database` (built-in chemical database)
- [x] **Milestone:** Calculate vapor pressures and fugacities for common chemicals

## Phase 3 — Phase Equilibrium (Months 7-9)
- [x] `tpt-proc-thermo-activity` (Wilson, NRTL, UNIQUAC, UNIFAC)
- [x] `tpt-proc-thermo-phase` (FlashSolver: PT flash, Rachford-Rice, bubble/dew point)
- [x] **Milestone:** Solve binary and multicomponent flash problems

## Phase 4 — Fluid Flow (Months 10-12)
- [x] `tpt-proc-fluid` (pipe flow, Reynolds, friction factor, Darcy-Weisbach)
- [x] `tpt-proc-pumps` (pump curves, operating point, affinity laws)
- [x] `tpt-proc-compressors`
- [x] `tpt-proc-valves`
- [x] `tpt-proc-network` (Hardy Cross, Newton-Raphson, linear theory solvers)
- [x] **Milestone:** Solve pipe network with Hardy Cross

## Phase 5 — Heat Transfer (Months 13-15)
- [x] `tpt-proc-heat-transfer` (convection correlations, Nusselt/Dittus-Boelter)
- [x] `tpt-proc-heat-exchangers` (LMTD, NTU-effectiveness, rating/sizing)
- [x] `tpt-proc-heat-network` (pinch analysis, composite curves, HEN synthesis)
- [x] `tpt-proc-fired-equipment`
- [x] **Milestone:** Rate and size shell-and-tube heat exchangers

## Phase 6 — Separations (Months 16-18)
- [x] `tpt-proc-distillation` (McCabe-Thiele, FUG shortcut, rigorous MESH)
- [x] `tpt-proc-absorption` (Kremser equation)
- [x] `tpt-proc-extraction`
- [x] `tpt-proc-membranes`
- [x] `tpt-proc-crystallization`
- [x] **Milestone:** Design binary distillation column with FUG shortcut

## Phase 7 — Reaction Engineering (Months 19-21)
- [x] `tpt-proc-reaction` (Arrhenius, power law, Langmuir-Hinshelwood, Michaelis-Menten kinetics)
- [x] `tpt-proc-reactors` (Batch, CSTR, PFR, packed bed, fluidized bed)
- [x] `tpt-proc-catalysis`
- [x] **Milestone:** Simulate CSTR and PFR with exothermic reaction

## Phase 8 — Flowsheet & Core Applications (Months 22-24)
- [x] `tpt-proc-flowsheet` (sequential modular + equation-oriented solvers)
- [x] `tpt-proc-mass-balance` (overall/component balance, reconciliation)
- [x] `tpt-proc-energy-balance`
- [x] `tpt-proc-optimization`
- [x] `tpt-proc-hvac` (psychrometrics)
- [x] `tpt-proc-water` (treatment train: coagulation → RO)
- [x] `tpt-proc-hydrogen` (electrolysis, SMR, ATR)
- [x] **Milestone:** Solve complete process flowsheet with recycle

## Phase 9 — Dynamics, Remaining Applications & Data
- [x] `tpt-proc-dynamics`
- [x] `tpt-proc-control`
- [x] `tpt-proc-pharma`
- [x] `tpt-proc-refining`
- [x] `tpt-proc-database` (general process data store)
- [x] `tpt-proc-pfd` (process flow diagram representation/export)
- [x] `tpt-proc-economics`
- [x] `tpt-proc-wasm` full bindings (WasmFlashCalculator, WasmHeatExchanger)
- [x] **Milestone:** Dynamic simulation + economics on a sample plant, deployed to WASM

## Cross-Cutting
- [x] `examples/flash-calculation`
- [x] `examples/heat-exchanger-rating`
- [x] `examples/binary-distillation`
- [x] `examples/cstr-reactor`
- [x] `examples/pinch-analysis`
- [x] `examples/hvac-psychrometrics`
- [x] `examples/water-treatment-plant`
- [x] `test-data/golden/thermodynamics/*` (PR vapor pressure, NRTL activity, flash VLE, bubble/dew)
- [x] `test-data/golden/fluid-flow/*` (pipe pressure drop, pump operating point, Hardy Cross)
- [x] `test-data/golden/heat-transfer/*` (LMTD, NTU, pinch analysis)
- [x] `test-data/golden/separations/*` (McCabe-Thiele, FUG, Kremser)
- [x] `test-data/golden/reactors/*` (CSTR conversion, PFR profile, Arrhenius rate)
- [x] `test-data/golden/applications/*` (psychrometrics, water treatment)
- [x] Cross-repo integration: `tpt-energy` (heat integration, utility optimization)
      *(crates/integration/tpt-proc-energy: pinch targets → `tpt-nrg-core` loads + CHP generator)*
- [x] Cross-repo integration: `tpt-materials` (chemical manufacturing processes)
      *(crates/integration/tpt-proc-materials: species → `tpt-mat-core` elemental composition; NMC precursor design)*
- [ ] Cross-repo integration: `tpt-medical` (pharmaceutical batch processes)
      *(counterpart repo has no code yet — `tpt-medical` contains only a spec stub)*
- [x] Cross-repo integration: `tpt-construction` (plant layout and construction)
      *(crates/integration/tpt-proc-construction: PFD → `tpt-c-model` Site/Project/Element with deterministic ids)*
- [ ] Standards support: ASPEN/HYSYS import, CAPE-OPEN, P&ID
      *(roadmap: CAPE-OPEN thermodynamics socket first, then .inp-style import)*
- [x] `docs/book`, `docs/api` publishing pipeline
      *(mdBook sources in `docs/book/src/`; rustdoc published by `docs.yml`)*

## Review Follow-ups (2026-09)

Findings from a full platform review. Nothing here is implemented yet; treat as
backlog, not status. External contributions are accepted as filed GitHub
issues (bug reports / feature requests via `.github/ISSUE_TEMPLATE/`), not as
unsolicited PRs — this applies in particular to the chemical database item
below.

### Bugs / correctness
- [ ] Replace `.expect(...)` panics in non-test library code with typed `Result` errors, per CONTRIBUTING.md's "no panics in library code" rule
      *(`tpt-proc-topology/src/graph.rs`, `lib.rs`; `tpt-proc-heat-network/src/lib.rs`; `tpt-proc-materials/src/lib.rs`; `tpt-proc-optimization/src/lib.rs`; `tpt-proc-thermo-eos/src/lib.rs`, `vapor.rs`; `tpt-proc-thermo-phase/src/lib.rs`)*
- [ ] Fix `tpt-proc-distillation::benzene_toluene_package()` (public API) to return `Result` instead of panicking
- [ ] Add convergence edge-case tests for `tpt-proc-flowsheet`'s recycle/tear-stream solver (non-converging loops, ill-conditioned tear streams, multiple recycle loops)
- [ ] Verify `tpt-energy` / `tpt-materials` / `tpt-construction` pinned git revs are reachable; document `integration/*` crates as optional/excludable if not core
- [ ] Add `cargo deny check advisories` (or `cargo audit`) to `license.yml` CI

### Chemical database — expand and verify accuracy
- [ ] Expand `tpt-proc-thermo-database` beyond its current ~27 components
- [ ] Source new component data from primary references (DIPPR/NIST/AIChE) and cross-check against at least two sources per property before adding
- [ ] Add a validation CI check that flags new/changed database entries for review (accuracy gate, not open contribution — database changes come from issues, reviewed and entered by maintainers)
- [ ] Document the source and verification method for each existing and new database entry

### Missing features
- [ ] `tpt-proc` CLI: config-driven flowsheet runner (TOML/YAML/JSON in, results out) so non-Rust users can run the engine
- [ ] `serde`-based (de)serialization for `Flowsheet`/`Stream`/`UnitOperation`
- [ ] Fill `docs/book/src/SUMMARY.md` gaps: HVAC, water, hydrogen, pharma, refining, dynamics, control, economics, PFD export, integration crates
- [ ] Close example-binary coverage gap: fluid-flow/network/pumps/valves/compressors, absorption/extraction/membranes/crystallization, non-CSTR reactors, economics
- [ ] Add README badges (build status, crates.io version, docs.rs, license, MSRV)

### Innovative additions
- [ ] WASM in-browser playground embedded in `docs/book` (flash calc demo, reusing existing `tpt-proc-wasm` bindings)
- [ ] "Visualize your flowsheet" example/tutorial surfacing `tpt-proc-pfd`'s SVG/DOT/Mermaid export
- [ ] "Validation" book page surfacing `test-data/golden/*` results against DIPPR/NIST/AIChE/API/ASHRAE references

### Usability / automation
- [ ] Coverage reporting (`cargo llvm-cov` + Codecov) to make crate-maturity claims verifiable
- [ ] `cargo-generate` template for scaffolding a new unit-op crate/example per the `UnitOperation` trait pattern
- [ ] Dry-run the manual-dispatch crates.io publish path on a low-dependency leaf crate before wider promotion
- [ ] Re-tier README's "✅ Stable" status table (Stable / Beta / Experimental) to reflect actual crate depth, not a uniform label

### Adoption / onboarding
- [ ] Add a copy-pasteable "5-minute quickstart" at the top of the README, above architecture/philosophy content
