# The tpt-proc CLI

`tpt-proc` (in `crates/cli/tpt-proc`) runs a flowsheet from a single
TOML file: it parses the config, builds the property package from the
built-in database, tears recycles, and solves with the
sequential-modular engine.

```sh
cargo run -p tpt-proc -- run <config.toml> [-o results.json]
cargo run -p tpt-proc -- components   # list the built-in chemicals
```

`run` prints a JSON report to stdout, or writes it with `-o`. The
report carries `converged`, `iterations`, `tear_residual`, the
component list, and one entry per stream: `id`, `name`,
`temperature_k`, `pressure_pa`, `molar_flow_mol_s`, `phase` (with β for
two-phase streams), and `mole_fractions`.

## Config schema

```toml
[flowsheet]
id = 1                      # numeric id
name = "my plant"           # optional label

[thermo]                    # optional, but required by flash/heater/
components = ["water", "methanol"]   # cooler/valve/pump units
eos = "peng-robinson"       # "peng-robinson" (default) | "srk" | "ideal"

[[streams]]                 # boundary feeds, fully specified
id = 1
name = "fresh feed"         # optional, defaults to S<id>
temperature_k = 350.0
pressure_pa = 101325.0
molar_flow_mol_s = 100.0
mole_fractions = [0.5, 0.5] # aligned with [thermo] components

[[units]]
kind = "mixer"              # mixer | splitter | flash | heater |
id = 10                     # cooler | valve | pump
inlets = 2                  # mixer: inlet port count (≥ 2, default 2)

[[units]]
kind = "splitter"
id = 12
ratios = [0.8, 0.2]         # required, ≥ 2 entries, normalized

[[units]]
kind = "heater"
id = 11
temperature_k = 360.0       # heater/cooler: outlet target (required)

[[units]]
kind = "flash"
id = 20
temperature_k = 350.0       # optional; inlet T carried if omitted
pressure_pa = 101325.0      # optional; inlet P carried if omitted

[[units]]
kind = "valve"
id = 30
pressure_pa = 50000.0       # valve: let-down outlet pressure (required)

[[units]]
kind = "pump"
id = 40
pressure_rise_pa = 500000.0 # pump: pressure rise (required)

[[connections]]
stream = 1                  # carrying stream id
to_unit = 10                # omit from_unit for a boundary feed
to_port = 0                 # ports default to 0

[[connections]]
stream = 23
from_unit = 12
from_port = 0               # omit to_unit for a product withdrawal
```

Semantics worth knowing:

- Mixer and splitter run on flow bookkeeping alone; flash, heater,
  cooler, valve, and pump re-equilibrate the inlet with a PT flash, so
  they need `[thermo]`. A flash has two outlets — port 0 vapor,
  port 1 liquid.
- A boundary feed (a connection without `from_unit`) must be declared
  in `[[streams]]`; nothing else ever writes it.
- Internal streams referenced by connections but not declared are
  created as zero-flow placeholders (seeded from the first declared
  stream) so units can execute before their tear-stream upstream on
  the first pass.
- The solver is called with a 1e-8 residual tolerance and 200
  iteration cap; non-convergence is reported, not hidden.

## A worked example

`crates/cli/tpt-proc/configs/recycle-demo.toml` — feed → mixer →
heater → splitter with a 20% recycle. At steady state the mixer passes
100/(1 − 0.2) = 125 mol/s: product 100 mol/s, recycle 25 mol/s.

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

Run it:

```sh
cargo run -p tpt-proc -- run crates/cli/tpt-proc/configs/recycle-demo.toml
```

Expect `"converged": true` and the product stream S23 at 360 K with
100 mol/s. The same schema is the library API: `tpt_proc::run_config_str`
takes the TOML text and returns the solver result plus component names.
