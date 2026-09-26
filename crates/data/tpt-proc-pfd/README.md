# tpt-proc-pfd

> Process flow diagram representation with SVG, Graphviz DOT, and Mermaid export.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-pfd.svg)](https://crates.io/crates/tpt-proc-pfd)
[![Documentation](https://docs.rs/tpt-proc-pfd/badge.svg)](https://docs.rs/tpt-proc-pfd)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The diagram layer of the [tpt-process](../../../README.md) process-engineering suite: a
minimal, deterministic in-memory process flow diagram (PFD) that exports to the three
formats engineers actually paste into documents and repositories.

## Overview

A PFD is, structurally, nothing more than labeled boxes connected by labeled arrows —
which is exactly the model this crate keeps. [`Pfd`] holds a title, a set of [`Node`]s
(unit operations or boundary markers, each with an id, a display label, and diagram
coordinates), and a list of [`Edge`]s (material or energy streams, with a label drawn
on the arrow). [`Pfd::add_node`] inserts or replaces a node; [`Pfd::connect`] adds a
stream, creating placeholder nodes for any missing endpoint so a diagram can be
sketched connectivity-first.

Three exporters render the same structure:

- [`Pfd::to_svg`] emits a standalone SVG document — node boxes, labeled arrows with a
  marker-end arrowhead, title text, and XML-escaped labels — sized from the node
  coordinates.
- [`Pfd::to_dot`] emits Graphviz DOT with `rankdir=LR`, node labels, and `pos`
  attributes, ready for `dot -Tsvg` or any Graphviz tooling.
- [`Pfd::to_mermaid`] emits `flowchart LR` syntax for Mermaid-rendered documentation
  (READMEs, wikis, rustdoc).

Determinism is a feature: nodes iterate in sorted id order (`BTreeMap`) and edges in
insertion order, so the same diagram always produces byte-identical output — diffs of
generated diagrams stay meaningful, and downstream consumers such as
[`tpt-proc-construction`](../../integration/tpt-proc-construction) can rely on stable
traversal.

## Features

- [`Pfd`] — the diagram: title, ordered nodes, insertion-ordered edges.
- [`Node`] — a unit operation or boundary marker with id, label, and x/y diagram position.
- [`Edge`] — a stream: source id, destination id, label.
- [`Pfd::new`] / [`Pfd::add_node`] / [`Pfd::connect`] — build a diagram; `connect` auto-creates missing endpoints.
- [`Pfd::nodes`] / [`Pfd::edges`] — deterministic accessors for traversal.
- [`Pfd::to_svg`] — standalone SVG export with boxes, labeled arrowheads, and escaped text.
- [`Pfd::to_dot`] — Graphviz DOT export (`rankdir=LR`, `pos` hints).
- [`Pfd::to_mermaid`] — Mermaid `flowchart LR` export.
- HTML/XML attribute escaping on all exported labels and ids; `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
tpt-proc-pfd = "0.1"
```

## Usage

```rust
use tpt_proc_pfd::{Node, Pfd};

let mut pfd = Pfd::new("demo");
pfd.add_node(Node::new("u1", "Feed pump", 40.0, 100.0));
pfd.add_node(Node::new("u2", "Heater", 240.0, 100.0));
pfd.connect("u1", "u2", "S1");

let dot = pfd.to_dot();
assert!(dot.contains("\"u1\" -> \"u2\""));
let svg = pfd.to_svg();
assert!(svg.contains("<svg"));
```

## API tour

| Item | Purpose |
|---|---|
| [`Pfd`] | The process flow diagram. |
| [`Node`] | Labeled, positioned diagram node. |
| [`Edge`] | Labeled stream between two node ids. |
| `Pfd::add_node` / `Pfd::connect` | Mutators; `connect` creates missing endpoint nodes. |
| `Pfd::nodes` / `Pfd::edges` | Deterministic iteration over the structure. |
| `Pfd::to_svg` | Render as standalone SVG. |
| `Pfd::to_dot` | Render as Graphviz DOT. |
| `Pfd::to_mermaid` | Render as Mermaid flowchart syntax. |

## Scope and limitations

- Structural model only: nodes are plain labeled rectangles — no unit-operation
  symbols, drawing conventions (ISA/ISO), stream property tables, heat/material
  balance annotation, or nested diagrams.
- Layout is exactly what you supply: x/y coordinates are used verbatim; there is no
  automatic layout engine for SVG (DOT consumers may re-layout via Graphviz).
- SVG output covers boxes, lines, labels, and one arrowhead marker — no styling API,
  colors, shapes, or zoom/pan affordances.
- No import: diagrams are built in code (or by a producer crate); there is no parser
  for DOT/SVG/Mermaid input.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types (compile-time dependency).
- [`tpt-proc-construction`](../../integration/tpt-proc-construction) — consumes `Pfd` to generate `tpt-construction` site/project models.
- [`tpt-proc-database`](../tpt-proc-database) — complementary process data store for the numbers behind the diagram.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
