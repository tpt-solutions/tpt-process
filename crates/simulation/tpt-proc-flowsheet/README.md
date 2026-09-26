# tpt-proc-flowsheet

> Sequential-modular flowsheet solving with tear streams and Wegstein acceleration.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-flowsheet.svg)](https://crates.io/crates/tpt-proc-flowsheet)
[![Documentation](https://docs.rs/tpt-proc-flowsheet/badge.svg)](https://docs.rs/tpt-proc-flowsheet)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The flowsheet execution engine of the [tpt-process](../../../README.md)
process-engineering suite: it drives the units of a `Flowsheet` in dependency
order and converges recycle loops by accelerated fixed-point iteration, per
accepted [RFC 0003](../../../rfcs/0003-flowsheet-solver.md).

## Overview

Real flowsheets contain recycles — reactor effluent split-backs, solvent
recoveries, heat-integrated loops. A naive fixed-point iteration on those
loops converges slowly or diverges. This crate implements the
sequential-modular (SM) answer, as specified in
[RFC 0003](../../../rfcs/0003-flowsheet-solver.md):

1. Build the process graph (`ProcessGraph` from [`tpt-proc-topology`](../../core/tpt-proc-topology))
   and detect the recycle (tear) streams that break every cycle.
2. Execute units in topological order over the torn graph, one full pass at a
   time (Gauss-Seidel style: each unit sees the freshest upstream values).
3. After each pass, compare the recomputed tear-stream flows and accelerate
   with bounded Wegstein acceleration — the secant-derived `q` factor is
   clamped to `[-5, 1]`, so the scheme degrades to direct substitution when
   the secant estimate is unreliable.
4. Report an honest convergence state: the [`FlowsheetResult`] carries the
   flag, the pass count, the last relative tear residual, and the final
   tear-stream flows for diagnostics.

The solver is deterministic (ordered maps throughout), treats unit behavior
as a pluggable registry entry, and is `#![forbid(unsafe_code)]`.

## Features

- [`FlowsheetSolver`] — the sequential-modular engine over a
  `tpt_proc_core::Flowsheet` plus a `tpt_proc_units::UnitRegistry`.
- [`FlowsheetSolver::run`] — executes passes until the tear streams converge
  to `tolerance` or `max_iterations` is exhausted.
- Automatic tear-stream detection and topological execution order via the
  `tpt-proc-topology` graph (`ProcessGraph::from_flowsheet`,
  `ProcessGraph::tear_streams`, `ProcessGraph::topological_order`).
- Bounded Wegstein acceleration on tear-stream total flow, with direct
  substitution on the first pass and on degenerate secants.
- [`FlowsheetResult`] — `converged`, `iterations`, `tear_residual`, and
  per-stream `tear_values` for auditing.
- [`FlowsheetError`] — precise failure modes: `Invalid` (structural
  validation), `Unit` (behavior error propagation), `MissingInlet`,
  `NoBehavior`.
- Works with any unit implementing the `tpt-proc-units` behavior trait
  (`Mixer`, `Splitter`, and future unit-operation crates).
- `#![forbid(unsafe_code)]`; no dependencies beyond the three suite crates it
  builds on.

## Installation

```toml
[dependencies]
tpt-proc-flowsheet = "0.1"
```

## Usage

```rust
use std::sync::Arc;
use tpt_proc_core::{Flowsheet, UnitId};
use tpt_proc_units::{Mixer, Splitter, UnitRegistry};
use tpt_proc_flowsheet::FlowsheetSolver;

let flowsheet = Flowsheet::new(0, "loop");
let mut registry = UnitRegistry::new();
registry.register(UnitId(1), Arc::new(Mixer));
registry.register(UnitId(2), Arc::new(Splitter::new(vec![0.8, 0.2])));

let solver = FlowsheetSolver::new(flowsheet, registry);
let result = solver.run(1e-9, 200).unwrap();
println!("converged: {}", result.converged);
```

## API tour

| Item | Purpose |
|---|---|
| [`FlowsheetSolver`] | The sequential-modular solver over a flowsheet + unit registry. |
| [`FlowsheetSolver::run`] | Run passes to tear-stream convergence (`tolerance`, `max_iterations`). |
| [`FlowsheetResult`] | `converged`, `iterations`, `tear_residual`, `tear_values`. |
| [`FlowsheetError`] | `Invalid`, `Unit`, `MissingInlet`, `NoBehavior` failure modes. |

## Scope and limitations

- Convergence is tracked on the tear streams' **total flow only**;
  accelerated updates rescale streams preserving T/P/composition, so
  composition or enthalpy teardown of recycles is not re-converged.
- Sequential modular only: the equation-oriented (EO) solver and the
  Broyden acceleration sketched in RFC 0003 are not implemented yet.
- Energy and specification loops are not converged — the residual is a flow
  residual, not a full mass-and-energy balance report.
- Acyclic flowsheets run a single effective pass; there is no degree-of-
  freedom analysis beyond the structural validation of `Flowsheet::validate`.
- Units must be registered in the `UnitRegistry`; an unregistered unit aborts
  with `FlowsheetError::NoBehavior` rather than being skipped.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — `Flowsheet`, `MaterialStream`,
  `UnitId`, `StreamId`, `PortId` types the solver executes over.
- [`tpt-proc-topology`](../../core/tpt-proc-topology) — `ProcessGraph`, tear
  stream selection, and topological ordering.
- [`tpt-proc-units`](../../core/tpt-proc-units) — the `UnitRegistry` and the
  concrete behaviors (`Mixer`, `Splitter`) executed each pass.
- No suite crate consumes `tpt-proc-flowsheet` yet; per the root README it is
  the designated execution engine for higher-level flowsheet assembly.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
