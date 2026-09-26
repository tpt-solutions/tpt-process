# example-pipe-network

Steady-state hydraulics of a small water-distribution network: two
reservoirs at different elevations feed four demand junctions through a
two-loop grid of commercial-steel pipes. The pipe resistances follow the
quadratic head-loss law h = r·Q·|Q|, with r derived from the Darcy
(Colebrook-White) friction factor. The network is solved with the three
classic methods — Hardy Cross loop corrections, the Newton-Raphson nodal
method, and Wood's Linear Theory — and the example verifies the network
crate's contract that all three solvers agree on every pipe flow, plus
global continuity (supply = demand) and the junction heads.

## Run

From the repository root:

```sh
cargo run --release -p example-pipe-network
```

## What it shows

- Hydraulics from `tpt-proc-fluid`: `PipeFlow::new(d, L, roughness)` and
  `friction_factor(velocity)` (Colebrook-White).
- Converting friction to resistance with `pipe_resistance(f, L, D)`.
- Building the graph with `PipeNetwork::add_node` / `add_pipe` using
  `NetworkNode { head, is_reservoir, demand }` and `NetworkPipe`.
- Three independent solvers: `solve_hardy_cross`, `solve_newton_raphson`,
  `solve_linear_theory`, returning `NetworkSolution` (pipe flows, junction
  heads, convergence, iteration count).
- Cross-checking the solvers' agreement and total supply vs. demand.

## Crates used

- [tpt-proc-fluid](../../crates/fluid-flow/tpt-proc-fluid)
- [tpt-proc-network](../../crates/fluid-flow/tpt-proc-network)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
