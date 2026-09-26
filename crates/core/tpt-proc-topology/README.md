# tpt-proc-topology

> Process flow diagram graph analysis: recycles, tear streams, and execution order.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-topology.svg)](https://crates.io/crates/tpt-proc-topology)
[![Documentation](https://docs.rs/tpt-proc-topology/badge.svg)](https://docs.rs/tpt-proc-topology)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A directed-multigraph toolkit that answers the structural questions a
flowsheet solver needs before executing anything. Part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

[`ProcessGraph`] models a flowsheet as unit operations (nodes) connected by
material streams (edges). Before a sequential-modular solver can run a PFD, it
must know whether the flowsheet is acyclic, where the recycle loops are, which
streams to tear, and which sections are independent. This crate answers all
four questions with classical graph theory:

- [`ProcessGraph::topological_order`] — Kahn-style execution order for
  acyclic flowsheets (sequential modular, no recycle); `None` when a cycle
  exists.
- [`ProcessGraph::find_recycles`] — strongly connected components with an
  internal cycle, via an iterative Tarjan pass, are the recycle loops of the
  flowsheet.
- [`ProcessGraph::tear_streams`] — a small deterministic set of streams whose
  removal breaks every recycle: one tear per cyclic component, chosen by the
  standard in-component out-degree heuristic and tie-broken by the lowest
  stream id.
- [`ProcessGraph::connected_components`] — independent sections that can be
  solved (or parallelized) separately.

Graphs can be built edge by edge, converted from a
`tpt_proc_core::Flowsheet` with [`ProcessGraph::from_flowsheet`], or parsed
from a compact PFD edge-list text format (`U1 -> U2 [S1]` lines, `#`
comments) with [`ProcessGraph::parse_pfd`].

All algorithms are iterative (no recursion), deterministic (ordered maps,
stable tie-breaks), and allocation-frugal.

## Features

- [`ProcessGraph::parse_pfd`] — parse PFD edge-list text, with
  `ParseError` (a `tpt_proc_core::CoreError` alias) for malformed input.
- [`ProcessGraph::from_flowsheet`] — project a
  `tpt_proc_core::Flowsheet` onto the unit digraph, ignoring boundary
  feeds/withdrawals.
- [`ProcessGraph::topological_order`] — `Option<Vec<UnitId>>` execution
  order; `None` on cycles.
- [`ProcessGraph::find_recycles`] — Tarjan SCCs filtered to genuinely cyclic
  components (including self-loops).
- [`ProcessGraph::tear_streams`] — deterministic tear selection; the torn
  graph is available via [`ProcessGraph::without_stream`].
- [`ProcessGraph::connected_components`] — undirected sections in
  deterministic order.
- [`ProcessGraph::adjacency`] and [`ProcessGraph::edges`] — raw structure
  access for downstream solvers.
- `#![forbid(unsafe_code)]`; no recursion, no `HashMap` iteration
  nondeterminism.

## Installation

```toml
[dependencies]
tpt-proc-topology = "0.1"
```

## Usage

```rust
use tpt_proc_topology::ProcessGraph;

// feed -> reactor -> separator, with unreacted feed recycled to the front
let text = r#"
    U1 -> U2 [S1]
    U2 -> U3 [S2]
    U3 -> U1 [S3]   # recycle
"#;
let graph = ProcessGraph::parse_pfd(text).unwrap();

assert!(graph.topological_order().is_none());     // cycle present
assert_eq!(graph.find_recycles().len(), 1);        // one recycle loop
assert_eq!(graph.tear_streams().len(), 1);         // tear one stream
```

## API tour

| Item | Purpose |
|---|---|
| `ProcessGraph` | The directed multigraph of units and streams |
| `ProcessGraph::parse_pfd` | Build from `U1 -> U2 [S1]` text |
| `ProcessGraph::from_flowsheet` | Build from a `tpt_proc_core::Flowsheet` |
| `ProcessGraph::topological_order` | Execution order for acyclic graphs |
| `ProcessGraph::find_recycles` | Recycle loops as Tarjan SCCs |
| `ProcessGraph::tear_streams` | Streams to cut to break every recycle |
| `ProcessGraph::connected_components` | Independent solvable sections |
| `Edge` | One stream edge (`from`, `to`, `stream`) |
| `ParseError` | Alias of `tpt_proc_core::CoreError` for parse failures |

## Scope and limitations

- Structural analysis only: the crate tells you *what* to converge, not *how*
  — recycle convergence itself lives in the flowsheet solver.
- Selecting the provably minimum tear set is NP-hard; `tear_streams` uses a
  degree heuristic (one tear per cyclic component), not a global optimum.
- One tear stream per recycle loop; more elaborate tear-set optimization
  (downstream-sensitivity criteria) is out of scope.
- Edge weights, equipment data, and stream physics are not modeled — a
  `ProcessGraph` is pure connectivity.

## Related crates

- [`tpt-proc-core`](../tpt-proc-core) — upstream: supplies `Flowsheet`,
  `StreamId`, `UnitId`, and the error type.
- [`tpt-proc-flowsheet`](../../simulation/tpt-proc-flowsheet) — downstream
  consumer: uses the tear set and execution order for sequential-modular
  convergence.
- [`tpt-proc-units`](../tpt-proc-units) — the unit behaviors that execute in
  the order this crate computes.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
