# tpt-proc-energy-balance

> Energy duty accounting and utility summaries across flowsheets.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-energy-balance.svg)](https://crates.io/crates/tpt-proc-energy-balance)
[![Documentation](https://docs.rs/tpt-proc-energy-balance/badge.svg)](https://docs.rs/tpt-proc-energy-balance)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The energy-ledger crate of the [tpt-process](../../../README.md)
process-engineering suite: collect the heating and cooling duties of a
flowsheet in one place and aggregate them into heating/cooling/net totals and
per-utility summaries.

## Overview

Every unit model in a flowsheet ultimately reports the same artifact: named
energy duties in watts, positive for heat into the process (reboilers,
preheaters) and negative for heat removed (condensers, coolers).
[`EnergyBalance`] is the ledger for those duties.

The API is deliberately minimal. [`EnergyBalance::add_duty`] inserts or
replaces a named duty; [`EnergyBalance::summary`] folds the ledger into an
[`EnergySummary`] with `total_heating` (sum of positive duties),
`total_cooling` (sum of the magnitudes of negative duties), and `net`.

For utility planning, name duties with a temperature-level tag —
`"reboiler@lp_steam"`, `"condenser@cooling_water"` — and
[`EnergyBalance::utility_summary`] groups the duties per utility level:
each duty is assigned to the utility whose name matches the tag after the
final `@`, with untagged duties collected under `"unassigned"`. That turns a
unit-by-unit duty list into the per-utility demand table you need for steam
and cooling-water sizing.

All quantities are SI watts; the ledger holds `f64` duties in a `BTreeMap`,
so iteration and grouping are deterministic.

## Features

- [`EnergyBalance`] — the duty ledger (`BTreeMap<String, f64>`, W).
- [`EnergyBalance::new`] — an empty ledger.
- [`EnergyBalance::add_duty`] — insert or replace a named duty
  (positive = heating, negative = cooling).
- [`EnergyBalance::duties`] — read access to the named duties.
- [`EnergyBalance::summary`] — aggregate [`EnergySummary`] with
  `total_heating`, `total_cooling`, and `net`.
- [`EnergyBalance::utility_summary`] — group duties per utility level via
  the `name@level` tag convention.
- Deterministic iteration and grouping (ordered map); `PartialEq`/`Clone`
  ledger for golden-file comparisons.
- `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
tpt-proc-energy-balance = "0.1"
```

## Usage

```rust
use tpt_proc_energy_balance::EnergyBalance;

let mut ledger = EnergyBalance::new();
ledger.add_duty("reboiler", 2.0e6);   // heating
ledger.add_duty("condenser", -1.5e6); // cooling
let summary = ledger.summary();
assert!((summary.net - 0.5e6).abs() < 1e-6);
```

## API tour

| Item | Purpose |
|---|---|
| [`EnergyBalance`] | Named-duty ledger in watts. |
| `add_duty` | Insert/replace a duty (W; + heating, − cooling). |
| `duties` | The named duties as a `BTreeMap<String, f64>`. |
| `summary` | [`EnergySummary`]: total heating, total cooling, net. |
| `utility_summary` | Duties grouped per utility via the `name@level` tag. |
| [`EnergySummary`] | `total_heating`, `total_cooling`, `net` (all W). |

## Scope and limitations

- Bookkeeping only: no property packages, no enthalpy calculation, no phase
  behavior — duties arrive in watts from elsewhere in the suite.
- Utility assignment is purely tag-based (`duty@utility` name matching):
  there is no temperature-interval analysis and no cost ranking of utility
  levels. Use [`tpt-proc-heat-network`](../../heat-transfer/tpt-proc-heat-network)
  for pinch targeting and heat-exchanger network synthesis.
- One value per duty name: adding the same name overwrites, so duty naming
  must be unique per exchanger.
- Steady-state totals: no time profiles, no start-up/shutdown accounting.
- SI watts only; no unit conversion.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — the stream and unit types of
  the suite (declared companion dependency).
- [`tpt-proc-heat-network`](../../heat-transfer/tpt-proc-heat-network) —
  pinch analysis and HEN synthesis; the thermodynamically rigorous
  counterpart for utility targeting.
- No suite crate consumes `tpt-proc-energy-balance` yet.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
