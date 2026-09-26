# Changelog

All notable changes to `tpt-proc-topology` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- `ProcessGraph`: a directed multigraph of unit operations and material streams with `add_unit`, `add_stream`, `without_stream`, `units`, `edges`, and `adjacency`.
- `ProcessGraph::parse_pfd`: parses PFD edge-list text (`U1 -> U2 [S1]` lines with `#` comments), with `ParseError` for malformed input.
- `ProcessGraph::from_flowsheet`: projects a `tpt_proc_core::Flowsheet` onto the unit digraph, ignoring boundary feeds/withdrawals.
- `ProcessGraph::topological_order`: Kahn-style execution order for acyclic flowsheets; `None` when a cycle exists.
- `ProcessGraph::find_recycles`: strongly connected components (iterative Tarjan) with internal cycles, ordered lexicographically for determinism.
- `ProcessGraph::tear_streams`: deterministic tear selection (in-component out-degree heuristic, one tear per cyclic component, lowest-stream-id tie-break).
- `ProcessGraph::connected_components`: independent undirected sections, units and sections in deterministic order.
- Iterative, allocation-frugal algorithms with stable tie-breaks and `#![forbid(unsafe_code)]`.

### Changed
- Removed internal `.expect` panics in the Tarjan SCC and Kahn topological-sort code paths; map lookups are now total (behavior on valid graphs is unchanged).
