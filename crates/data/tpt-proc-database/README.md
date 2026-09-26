# tpt-proc-database

> A typed, unit-annotated process data store with deterministic CSV round-trip.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-database.svg)](https://crates.io/crates/tpt-proc-database)
[![Documentation](https://docs.rs/tpt-proc-database/badge.svg)](https://docs.rs/tpt-proc-database)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The general-purpose process data store of the [tpt-process](../../../README.md)
process-engineering suite: a lightweight key-value container for tag-like plant data
(`feed.flow = 125.0 kg/h`, `p1.pressure = 12.5 bar`) that serializes to plain CSV and
back without losing types or ordering.

## Overview

Every flowsheet calculation ends up producing the same artifact: a bag of named
values, some numeric with an engineering unit attached, some textual (unit names,
notes, tags). [`DataStore`] is that bag, kept deliberately simple and deterministic:

- Values are typed. [`DataStore::set_number`] stores a value *and* its unit string;
  [`DataStore::set_text`] stores plain text. Reads are type-strict — a text entry
  does not answer [`DataStore::number`], and a numeric entry does not answer
  [`DataStore::text`] — so unit annotations cannot silently drift away from their
  values.
- Iteration is deterministic. Entries live in a `BTreeMap`, so [`DataStore::keys`]
  yields keys in sorted order and two stores built in different orders still compare
  equal and serialize identically. That makes golden-file testing and diffing of
  exported data meaningful.
- CSV is the interchange format. [`DataStore::to_csv`] emits a `key,value,unit`
  table (text entries carry an empty unit column); [`DataStore::from_csv`] parses it
  back, auto-detecting numeric vs. text rows and reporting the offending line number
  for malformed input.

The design point is the same one that governs the rest of the suite: auditable,
diff-friendly engineering data in a format any spreadsheet, historian export, or
script can read, with zero heavyweight dependencies.

## Features

- [`DataStore`] — the process data store with deterministic (ordered) iteration.
- [`DataStore::set_number`] / [`DataStore::number`] — numeric values with an attached unit string.
- [`DataStore::set_text`] / [`DataStore::text`] — textual values.
- [`DataStore::keys`] — ordered key iteration; [`DataStore::len`] and [`DataStore::is_empty`] for sizing.
- [`DataStore::to_csv`] — serializes to a `key,value,unit` CSV table.
- [`DataStore::from_csv`] — parses the same format, with per-line error reporting for malformed rows.
- [`Entry`] — the stored value: `Number(f64, String)` (value + unit) or `Text(String)`.
- Round-trip guarantee: `from_csv(to_csv(store)) == store` (verified by the crate tests).
- `#![forbid(unsafe_code)]` and no dependencies beyond `tpt-proc-core`.

## Installation

```toml
[dependencies]
tpt-proc-database = "0.1"
```

## Usage

```rust
use tpt_proc_database::DataStore;

let mut store = DataStore::new();
store.set_number("feed.flow", 125.0, "kg/h");
store.set_text("feed.name", "crude-40");

assert_eq!(store.number("feed.flow"), Some((125.0, "kg/h")));
assert_eq!(store.text("feed.name"), Some("crude-40"));
```

## API tour

| Item | Purpose |
|---|---|
| [`DataStore`] | Ordered, typed key-value store for process data. |
| [`Entry`] | A stored value: number with unit, or text. |
| `set_number` / `number` | Write / read a numeric value with its unit. |
| `set_text` / `text` | Write / read a text value. |
| `keys`, `len`, `is_empty` | Ordered key iteration and sizing. |
| `to_csv` | Serialize to the `key,value,unit` CSV format. |
| `from_csv` | Parse that CSV format; errors carry the offending line number. |

## Scope and limitations

- Single value per key: there is no time-stamped series storage, historian, or
  interpolation — use it for snapshots and results tables, not trending.
- No hierarchical namespaces, queries, or indices beyond exact string keys and
  sorted iteration.
- CSV parsing is line-based `splitn` on commas: no quoted fields, embedded commas,
  or header variations; malformed rows return an error rather than best-effort recovery.
- No unit conversion or validation — units are carried verbatim as annotation strings.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types (compile-time dependency).
- [`tpt-proc-thermo-database`](../../thermodynamics/tpt-proc-thermo-database) — the *chemical* property databank; this crate is the general *process data* store, a different concern.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
