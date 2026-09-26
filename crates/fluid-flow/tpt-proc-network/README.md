# tpt-proc-network

> Pipe network solvers: Hardy Cross, Newton-Raphson, and Linear Theory on one graph.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-network.svg)](https://crates.io/crates/tpt-proc-network)
[![Documentation](https://docs.rs/tpt-proc-network/badge.svg)](https://docs.rs/tpt-proc-network)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`tpt-proc-network` solves steady-state hydraulics of whole pipe networks —
looped water distribution grids, multi-reservoir supply systems — with three
classic algorithms that cross-verify each other. Part of the
[tpt-process](../../../README.md) process-engineering suite.

## Overview

Head loss per pipe follows the quadratic law `h = r·Q·|Q|` with resistance

```text
r = 8·f·L / (π²·g·D⁵)
```

computed by the free function `pipe_resistance` from a (frozen) Darcy
friction factor — the classic textbook formulation. To update friction with
flow, re-evaluate `r` and re-solve.

A `PipeNetwork` is a set of `NetworkNode`s (fixed-head reservoirs or demand
junctions) joined by `NetworkPipe`s. Three solvers are provided:

- **Hardy Cross** (`solve_hardy_cross`) — the classic loop-correction method
  over a cycle basis derived from a spanning tree, including pseudo cycles
  that tie additional reservoirs to the root so multi-reservoir networks are
  properly determined.
- **Newton-Raphson** (`solve_newton_raphson`) — the global nodal method:
  unknowns are junction heads, residuals are continuity errors, solved with
  a dense Gaussian-elimination linear solver (partial pivoting).
- **Linear Theory** (`solve_linear_theory`, Wood) — the head-loss law
  linearized as `h ≈ (r·|Q₀|)·Q`, with the nodal system re-solved each pass.

The crate's own tests enforce that all three solvers agree on every pipe
flow of a two-loop benchmark network — the strongest available check short
of a published benchmark. Initial flows satisfy junction continuity exactly
(spanning-tree based), and junction heads are recovered for reporting.

## Features

- `PipeNetwork` — builder-style `add_node` / `add_pipe` construction.
- `NetworkNode` — reservoir (fixed head) or demand junction.
- `NetworkPipe` — directed pipe with resistance r in s²/m⁵.
- `pipe_resistance` — r from friction factor, length, and diameter.
- `solve_hardy_cross` — loop-correction method with pseudo cycles for
  multiple reservoirs.
- `solve_newton_raphson` — nodal Newton method on junction heads.
- `solve_linear_theory` — Wood's Linear Theory iteration.
- `NetworkSolution` — per-pipe flows, junction heads, `converged` flag,
  iteration count; plus `PipeNetwork::head_loss` for post-processing.

## Installation

```toml
[dependencies]
tpt-proc-network = "0.1"
```

## Usage

```rust
use tpt_proc_network::{pipe_resistance, NetworkNode, NetworkPipe, PipeNetwork};

// Two reservoirs joined by a 1000 m, DN200 pipe (f = 0.02).
let mut net = PipeNetwork::new();
net.add_node(1, NetworkNode { head: 110.0, is_reservoir: true, demand: 0.0 });
net.add_node(2, NetworkNode { head: 100.0, is_reservoir: true, demand: 0.0 });
net.add_pipe(NetworkPipe {
    from: 1,
    to: 2,
    resistance: pipe_resistance(0.02, 1000.0, 0.2),
});

let sol = net.solve_newton_raphson(1e-10, 100).unwrap();
assert!(sol.converged);
println!("flow = {} m³/s", sol.pipe_flows[0]);
```

## API tour

| Item | Purpose |
|---|---|
| `PipeNetwork` | The graph; node/pipe storage and the three solvers |
| `NetworkNode` | Fixed-head reservoir or demand junction |
| `NetworkPipe` | Pipe with from/to ids and resistance r |
| `pipe_resistance` | `r = 8fL/(π²gD⁵)` from a frozen friction factor |
| `solve_hardy_cross` | Loop-correction solver (needs at least one cycle) |
| `solve_newton_raphson` | Nodal Newton solver on junction heads |
| `solve_linear_theory` | Wood's Linear Theory solver |
| `NetworkSolution` | Flows, junction heads, convergence diagnostics |

## Scope and limitations

- **Frozen friction factor** — r is fixed per solve; iterate externally
  (recompute r from each pipe's Reynolds number) for friction updates.
- **Incompressible, single-phase liquid** — no gas networks, no pumps or
  valves as network elements, no pressure-dependent demands.
- **Quadratic head loss only** — no minor losses per pipe or laminar
  corrections inside the solver.
- **Hardy Cross requires at least one cycle** and errors on tree networks
  and malformed references; dense linear algebra limits practical size to
  modest networks (no sparse solver).
- SI units throughout (m, m³/s, s²/m⁵).

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types.
- [`tpt-proc-fluid`](../tpt-proc-fluid) — friction factors for computing the
  per-pipe resistances fed into the network.
- `benches/network-solver` in the repository root benchmarks the three
  solvers.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
