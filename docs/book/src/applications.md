# Applications

The `crates/applications/` layer packages the domain crates into
whole-process answers: a heating-coil sizing, a treatment-train outlet, a
plant's hydrogen balance. Each application crate is deliberately small —
a state object, a model enum or builder, and closed-form or one-pass
calculations at pre-design (feasibility/screening) grade.

Shared conventions:

- SI units everywhere (K, Pa, W, mol) unless the field name says
  otherwise (e.g. `dose_mg_l`).
- Models are stateless pure functions of their inputs; identical inputs
  give identical outputs.
- Application-level golden tests live under
  `test-data/golden/applications/` alongside the domain data.

The five application areas:

- [HVAC & Psychrometrics](hvac.md) — moist-air states at
  near-atmospheric pressure (`tpt-proc-hvac`).
- [Water Treatment](water.md) — coagulation-to-disinfection train
  simulation (`tpt-proc-water`).
- [Hydrogen Production](hydrogen.md) — electrolysis, SMR, ATR, and
  water-gas shift energy accounting (`tpt-proc-hydrogen`).
- [Pharmaceutical Batch Processes](pharma.md) — batch cycles,
  dissolution, and shelf life (`tpt-proc-pharma`).
- [Refining](refining.md) — API gravity, Watson K, and crude assays
  (`tpt-proc-refining`).

Worked example binaries for the psychrometrics and water-treatment
models are under `examples/` in the repository root.
