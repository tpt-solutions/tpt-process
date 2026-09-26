# tpt-proc (CLI)

> Config-driven flowsheet runner: TOML process flowsheet in, JSON results out.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc.svg)](https://crates.io/crates/tpt-proc)
[![Documentation](https://docs.rs/tpt-proc/badge.svg)](https://docs.rs/tpt-proc)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A command-line front door to the [tpt-process](../../../README.md)
process-engineering suite: describe a flowsheet in a small TOML document and
run it through the sequential-modular solver — no Rust required.

## Overview

`tpt-proc` builds a `Flowsheet` from a TOML config, attaches unit
behaviors, and runs the `tpt-proc-flowsheet` sequential-modular engine with
Wegstein-accelerated tear-stream convergence. Boundary streams resolve their
components through the built-in open chemical database, and flash-derived
units re-equilibrate through the Peng-Robinson (or SRK, or ideal/Raoult)
property package.

The library half of this crate (`run_config` / `run_config_str`) is the same
engine path the binary uses, so the runner is embeddable in Rust tests and
services as well.

## Features

- **TOML in, JSON out** — one `run` command; results per stream include
  temperature, pressure, molar flow, phase, and mole fractions, plus solver
  diagnostics (`converged`, `iterations`, `tear_residual`).
- **Recycle loops** — tear streams converge automatically (Wegstein
  acceleration); see `configs/recycle-demo.toml` (100 mol/s fresh feed,
  20 % recycle → 125 mol/s through the mixer).
- **Unit kinds**: `mixer`, `splitter`, `flash` (two outlets: vapor/liquid),
  `heater`, `cooler` (target outlet temperature), `valve` (outlet pressure),
  `pump` (pressure rise).
- **Thermodynamics**: `[thermo]` selects components from the built-in
  database and the EOS (`"peng-robinson"` (default), `"srk"`, `"ideal"`).
- **`components` command** — list the built-in chemical database.
- **Embeddable** — `tpt_proc::run_config_str` returns the typed
  `FlowsheetResult` for programmatic use.

## Installation

From the repository (the suite is not yet on crates.io):

```sh
git clone https://github.com/tpt-solutions/tpt-process
cd tpt-process
cargo build --release -p tpt-proc
```

## Usage

```sh
# Solve a flowsheet and print the JSON report
tpt-proc run crates/cli/tpt-proc/configs/recycle-demo.toml

# Write the report to a file
tpt-proc run flowsheet.toml -o results.json

# List available database components
tpt-proc components
```

A complete config (`configs/recycle-demo.toml`):

```toml
[flowsheet]
id = 1
name = "recycle demo"

[thermo]
components = ["water", "methanol"]
eos = "peng-robinson"

[[streams]]
id = 1
name = "fresh feed"
temperature_k = 350.0
pressure_pa = 101325.0
molar_flow_mol_s = 100.0
mole_fractions = [0.5, 0.5]

[[units]]
kind = "mixer"
id = 10
inlets = 2

[[units]]
kind = "heater"
id = 11
temperature_k = 360.0

[[units]]
kind = "splitter"
id = 12
ratios = [0.8, 0.2]

[[connections]]
stream = 1
to_unit = 10
to_port = 0

[[connections]]
stream = 21
from_unit = 10
from_port = 0
to_unit = 11
to_port = 0

[[connections]]
stream = 22
from_unit = 11
from_port = 0
to_unit = 12
to_port = 0

[[connections]]
stream = 23
from_unit = 12
from_port = 0

[[connections]]
stream = 24
from_unit = 12
from_port = 1
to_unit = 10
to_port = 1
```

Streams referenced only by `[[connections]]` are registered automatically as
zero-flow placeholders (seeded with the first declared stream's state);
boundary feeds (`from_unit` absent) must be fully declared in `[[streams]]`.

## Scope and limitations

- Flash-derived units (`flash`, `heater`, `cooler`, `valve`, `pump`) require
  a `[thermo]` section and re-equilibrate with a PT flash; the heater/cooler
  sets a *target outlet temperature* — a duty specification is not yet
  supported.
- `mixer` and `splitter` run on flow bookkeeping alone (flow-weighted
  temperature, minimum-pressure mixing rules from `tpt-proc-units`).
- Streams are molar-flow based (mol/s); mass/volumetric bases are not yet
  converted automatically.
- Rigorous columns, reactors, and heat-exchanger rating are not yet wired as
  behaviors; use the Rust crates directly for those.
- The database is the built-in 51-component set (see
  `tpt-proc-thermo-database`/PROVENANCE.md).

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — flowsheet/stream data model
  (with the `serde` feature this CLI enables).
- [`tpt-proc-units`](../../core/tpt-proc-units) — mixer/splitter behaviors.
- [`tpt-proc-flowsheet`](../../simulation/tpt-proc-flowsheet) — the solver.
- [`tpt-proc-thermo-database`](../../thermodynamics/tpt-proc-thermo-database)
  and [`tpt-proc-thermo-phase`](../../thermodynamics/tpt-proc-thermo-phase) —
  component data and flash equilibrium.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
