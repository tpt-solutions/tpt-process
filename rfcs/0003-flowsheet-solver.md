# RFC 0003 — Flowsheet Solver

- **Status:** Accepted
- **Start date:** 2026-03
- **Crates:** `tpt-proc-flowsheet`, `tpt-proc-topology`, `tpt-proc-core`

## Summary

A sequential-modular (SM) flowsheet solver as the default execution engine —
topological execution over `tpt-proc-topology` with tear streams and Wegstein
acceleration for recycles — plus a scaffold for an equation-oriented (EO)
solver sharing the same unit-operation trait.

## Motivation

Real flowsheets contain recycle loops (reactor effluent split-back, solvent
recoveries). Naive fixed-point iteration on recycles converges slowly or
diverges. The solver must:

- execute units in dependency order, computing each unit exactly once per
  pass;
- detect and break recycles with a small set of tear streams;
- accelerate convergence (Wegstein, then Broyden for tight tolerances);
- report honest convergence state plus mass/energy balance residuals, since
  these feeds auditability;
- remain deterministic for reproducible designs.

## Detailed design

### Execution model

1. Build the `ProcessGraph` (units as nodes, streams as edges).
2. Compute strongly connected components (Tarjan). Acyclic components get
   a topological execution order directly.
3. For each cyclic component, select tear streams: prefer streams that
   break the most cycles, tie-break on downstream unit cost (heuristic:
   tear one stream per SCC at the outlet of the unit with the cheapest
   model). Initialize tear streams with the inlet estimate or zero flow.
4. Pass loop: execute units in topological order of the torn graph; after
   each pass compare recomputed tear-stream values; apply Wegstein
   acceleration per tear stream (bounded q ∈ [−5, 1] to remain stable):

   `x⁽ⁿ⁺¹⁾ = q·x⁽ⁿ⁺¹⁾_direct + (1−q)·x⁽ⁿ⁾`, `q = s/(s−1)`,
   `s = (x⁽ⁿ⁾−x⁽ⁿ⁻¹⁾)/(x⁽ⁿ⁺¹⁾_direct − x⁽ⁿ⁻¹⁾)`

5. Convergence: relative difference per tear stream < 1e-9 (flow, molar
   composition, enthalpy), max 100 passes. Report
   `mass_balance_error` (global in−out residual) and
   `energy_balance_error`.

### Unit contract

```rust
pub trait UnitBehavior {
    fn ports(&self) -> PortSpec;             // named in/out ports
    fn solve(&self, inlets: &[MaterialStream], energy: EnergyInputs,
             package: &dyn PropertyPackage)
        -> Result<UnitOutput, UnitError>;    // outlets + duties + internals
}
```

Units are pure functions of their inlets and parameters — no hidden state —
which makes SM execution trivially parallelizable per topological level and
makes the EO solver (same equations, simultaneous solve) share the residual
code.

### Equation-oriented scaffold

EO collects unit residuals into one sparse system (unit Jacobians via
forward finite differences initially) and solves with a damped Newton loop.
Phase 8 delivers the scaffold with basic convergence on small flowsheets;
large-scale sparse factorization is delegated to `tpt-math-linalg-fixed`
when that substrate lands.

### Determinism rules

- Ordered maps everywhere (`BTreeMap`); unit execution order within a level
  is by `UnitId`.
- Identical inputs → bit-identical outputs on the same platform (CI test).

## Verification & validation

- Textbook recycle flowsheet (mixer → heater → splitter with purge, recycle
  ratio 5) converges to the analytic solution within 1e-9.
- Acyclic PFD executes each unit exactly once (unit counter test).
- Tear-set minimality on a hand-built double-recycle graph.
- Golden file: recycle flowsheet stream table
  (`test-data/golden/flowsheets/recycle-mixer-heater.json`).

## Alternatives considered

- **Simultaneous modular (tear + unit Jacobians):** good middle ground;
  reachable later as a third strategy without API change.
- **EO-first:** fastest convergence for design-grade problems, but fragile
  initialization and much higher implementation risk for v1; SM covers the
  simulation use cases and EO grows from the shared unit contract.

## Unresolved questions

- Control-block semantics (design specs solved by nesting tear iterations)
  — follow-up RFC when `tpt-proc-control` integration lands.
- Numeric Jacobian vs autodiff for EO (workspace decision pending).
