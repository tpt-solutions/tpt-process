# tpt-proc-construction

> PFD units from `tpt-process`, exported as deterministic site and project records for `tpt-construction`.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A workspace-internal integration bridge of the [tpt-process](../../../README.md)
process-engineering suite. It connects the process side (diagrams in
[`tpt-proc-pfd`](../../data/tpt-proc-pfd)) to the construction side (sites, elements,
and property sets in `tpt-construction` via the `tpt-c-*` crates). **This crate is
not published to crates.io**: it depends on git dependencies pinned to substrate
revisions and is consumed as a git or path dependency inside the TPT workspace.

## Overview

A flowsheet is the seed of a plant layout: every unit on the PFD is a piece of
equipment someone must place on a site. This crate performs that first export step.
[`plant_project`] turns a [`tpt_proc_pfd::Pfd`] into a [`tpt_c_model::Project`]
containing:

- one [`tpt_c_model::Site`] named after the diagram title,
- one [`tpt_c_model::Element`] per PFD node, categorized
  [`PROCESS_EQUIPMENT_CATEGORY`] (`"ProcessEquipment"`), each carrying a
  `"Placement"` [`tpt_c_model::PropertySet`] with the node's x/y diagram coordinates,
- the site referencing all exported element ids.

The export is *deterministic*: element ids are UUIDv5 values derived from the PFD
node id ([`tpt_c_ids::IdFactory::deterministic_element`]), so re-exporting an
unchanged diagram is idempotent and downstream diffs stay meaningful. On top of the
model export, [`plant_project_json`] returns the project with its pretty-printed JSON
serialization (via `serde_json`) for downstream `tpt-c-*` tooling, and [`line_list`]
extracts the stream connectivity as `(from, stream, to)` triples for when the
construction side needs a line list.

## Features

- [`plant_project`] — PFD → `tpt-c-model::Project` with one site, one equipment element per node, and `Placement` property sets.
- [`PROCESS_EQUIPMENT_CATEGORY`] — the stable `"ProcessEquipment"` category constant assigned to every exported unit.
- [`plant_project_json`] — the same export plus pretty-printed project JSON (`serde_json`).
- [`line_list`] — PFD edges as `(from label, stream label, to label)` triples.
- Deterministic UUIDv5 element ids via `tpt-c-ids`, making re-exports idempotent and diff-friendly.
- `#![forbid(unsafe_code)]`.

## Installation

Not on crates.io. Consume it as a git dependency of the workspace:

```toml
[dependencies]
tpt-proc-construction = { git = "https://github.com/tpt-solutions/tpt-process" }
```

Its git dependencies (`tpt-c-core`, `tpt-c-ids`, `tpt-c-model` from
`tpt-solutions/tpt-construction`) are resolved through the workspace dependency
table, pinned to substrate revisions, so a pinned revision of this repository builds
reproducibly.

## Usage

```rust
use tpt_proc_construction::plant_project;
use tpt_proc_pfd::{Node, Pfd};

let mut pfd = Pfd::new("distillation train");
pfd.add_node(Node::new("u1", "Feed pump", 40.0, 100.0));
pfd.add_node(Node::new("u2", "Column C-101", 240.0, 100.0));
pfd.connect("u1", "u2", "S1");

let project = plant_project(&pfd);
assert_eq!(project.element_count(), 2);
assert_eq!(project.sites.len(), 1);
```

## API tour

| Item | Purpose |
|---|---|
| [`plant_project`] | Export a `Pfd` as a `tpt-construction` `Project` (site + equipment elements). |
| [`plant_project_json`] | [`plant_project`] plus pretty-printed JSON of the project. |
| [`line_list`] | Stream connectivity triples for line-list consumers. |
| [`PROCESS_EQUIPMENT_CATEGORY`] | Category constant (`"ProcessEquipment"`) on exported elements. |

## Scope and limitations

- Placement is limited to the two PFD coordinates: x/y become the `Placement`
  property set; there is no elevation, orientation, footprint sizing, plot-plan
  layout, spacing/routing rules, or collision checking.
- No geometry: exported elements are records with property sets, not 3D models or
  2D drawings.
- Streams export only through [`line_list`] — the `Project` itself carries equipment,
  not piping.
- One flat site per diagram; no multi-area hierarchy, buildings, or phased schedules.

## Related crates

- [`tpt-proc-pfd`](../../data/tpt-proc-pfd) — upstream: the `Pfd`/`Node`/`Edge` model this bridge exports.
- `tpt-c-core`, `tpt-c-ids`, `tpt-c-model` ([`tpt-solutions/tpt-construction`](https://github.com/tpt-solutions/tpt-construction)) — downstream substrate: `Project`, `Site`, `Element`, `PropertySet`, deterministic `IdFactory` (git dependencies, pinned revision).
- `serde_json` — project JSON serialization.
- [`tpt-proc-energy`](../tpt-proc-energy) — sibling integration bridge, pinch targets → energy-system models.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
