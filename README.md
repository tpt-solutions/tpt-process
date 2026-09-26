# tpt-process

[![CI](https://github.com/tpt-solutions/tpt-process/actions/workflows/ci.yml/badge.svg)](https://github.com/tpt-solutions/tpt-process/actions/workflows/ci.yml)
[![License check](https://github.com/tpt-solutions/tpt-process/actions/workflows/license.yml/badge.svg)](https://github.com/tpt-solutions/tpt-process/actions/workflows/license.yml)
[![Crates.io](https://img.shields.io/crates/v/tpt-proc-core.svg)](https://crates.io/crates/tpt-proc-core)
[![docs.rs](https://docs.rs/tpt-proc-core/badge.svg)](https://docs.rs/tpt-proc-core)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![MSRV](https://img.shields.io/badge/MSRV-1.84-blue.svg)

A fully open-source, MIT-licensed computational engine for chemical and process
engineering: thermodynamics, fluid flow, heat transfer, separations, and
reaction engineering.

> **100% Open Source. Chemical & Process Engineering Without Vendor Lock-in.
> No GPL Traps. No Desktop Monopolies.**

## 5-Minute Quickstart

Create a project and depend on the suite (published to crates.io at the next
release; until then use the git dependency):

```toml
# Cargo.toml
[dependencies]
tpt-proc-core = { git = "https://github.com/tpt-solutions/tpt-process" }
tpt-proc-thermo-core = { git = "https://github.com/tpt-solutions/tpt-process" }
tpt-proc-thermo-database = { git = "https://github.com/tpt-solutions/tpt-process" }
tpt-proc-thermo-eos = { git = "https://github.com/tpt-solutions/tpt-process" }
tpt-proc-thermo-phase = { git = "https://github.com/tpt-solutions/tpt-process" }
```

Paste this into `src/main.rs` — a Peng-Robinson flash of a 50/50
water/methanol feed at 350 K, 1 atm:

```rust
use std::sync::Arc;

use tpt_proc_core::Composition;
use tpt_proc_thermo_core::PropertyPackage;
use tpt_proc_thermo_database::ChemicalDatabase;
use tpt_proc_thermo_eos::CubicEos;
use tpt_proc_thermo_phase::FlashSolver;

fn main() {
    let db = ChemicalDatabase::builtin();
    let components = db.components_for(&["water", "methanol"]).unwrap();
    let eos = CubicEos::peng_robinson(components.clone());
    let package = PropertyPackage::new(components).unwrap().with_eos(Arc::new(eos));
    let flash = FlashSolver::new(package);

    let feed = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
    let result = flash.pt_flash(&feed, 350.0, 101_325.0).unwrap();

    println!("Vapor fraction: {}", result.vapor_fraction);
    println!("Vapor composition: {:?}", result.vapor_composition.as_slice());
}
```

```sh
cargo run
# Vapor fraction: 0.55…
```

Prefer no Rust at all? The `tpt-proc` CLI runs a TOML-described flowsheet —
mixers, splitters, flash drums, heaters, valves, pumps with recycles — see
[crates/cli/tpt-proc](crates/cli/tpt-proc). Worked Rust examples live in
[`examples/`](examples):

```sh
git clone https://github.com/tpt-solutions/tpt-process
cd tpt-process
cargo run --release -p example-flash-calculation
```

## Why?

Commercial process simulators (Aspen HYSYS, Aspen Plus, ChemCAD, PRO/II,
gPROMS) cost $30k–$100k+ per seat, are closed-source, and lock your property
data behind recurring license fees. `tpt-process` breaks all three locks:

- **Pure Rust** — fast enough for real-time control, safe enough for long
  dynamic simulations, deterministic enough for auditable designs.
- **Open databank** — a built-in chemical database that is a public good,
  not a ransom.
- **WebAssembly** — flash calculations, heat exchanger ratings, and
  distillation shortcuts run in the browser or on edge controllers.
- **Strict permissive license chain** — MIT OR Apache-2.0 end to end, with
  `cargo deny` enforcement so no copyleft dependency can ever enter the tree.

## Crates

Status tiers reflect real coverage, not aspiration: **✅ Stable** = the
implemented physics is textbook-benchmarked and the API is settled;
**🧪 Beta** = solid for pre-design but model coverage is narrower than the
domain (see each crate's README "Scope and limitations");
**🚧 Alpha** = under active development; **🔗 Integration** = bridges into
the other TPT substrate repositories, workspace-only (git dependency, never
on crates.io).

| Crate | Description | Status |
|---|---|---|
| `tpt-proc-core` | Core stream and unit types | ✅ Stable |
| `tpt-proc-topology` | PFD graph analysis, recycles, tear streams | ✅ Stable |
| `tpt-proc-units` | Unit operation scaffolding, mixer/splitter | ✅ Stable |
| `tpt-proc-thermo-core` | Property package framework | ✅ Stable |
| `tpt-proc-thermo-eos` | Peng-Robinson, SRK equations of state | ✅ Stable |
| `tpt-proc-thermo-activity` | Wilson, NRTL, UNIQUAC, UNIFAC | ✅ Stable |
| `tpt-proc-thermo-phase` | Flash calculations, VLE, bubble/dew point | ✅ Stable |
| `tpt-proc-thermo-database` | Built-in chemical database (51 components, sourced + gated) | ✅ Stable |
| `tpt-proc-fluid` | Pipe flow, Darcy-Weisbach, Colebrook-White | ✅ Stable |
| `tpt-proc-pumps` | Pump curves, operating point, affinity laws | ✅ Stable |
| `tpt-proc-compressors` | Isentropic/polytropic compression | ✅ Stable |
| `tpt-proc-valves` | Cv/Kv sizing, inherent characteristics | ✅ Stable |
| `tpt-proc-network` | Hardy Cross, Newton-Raphson, Linear Theory | ✅ Stable |
| `tpt-proc-heat-transfer` | Convection correlations, Nusselt number | ✅ Stable |
| `tpt-proc-heat-exchangers` | LMTD, NTU-effectiveness, rating/sizing | ✅ Stable |
| `tpt-proc-heat-network` | Pinch analysis, composite curves, HEN | ✅ Stable |
| `tpt-proc-fired-equipment` | Furnaces and fired heaters | ✅ Stable |
| `tpt-proc-distillation` | McCabe-Thiele, FUG, rigorous MESH | ✅ Stable |
| `tpt-proc-absorption` | Kremser absorption/stripping | ✅ Stable |
| `tpt-proc-reaction` | Kinetics: Arrhenius, power law, LHHW, Michaelis-Menten | ✅ Stable |
| `tpt-proc-mass-balance` | Balances and data reconciliation | ✅ Stable |
| `tpt-proc-optimization` | Golden section, Nelder-Mead, gradient | ✅ Stable |
| `tpt-proc-hvac` | Psychrometrics and air systems | ✅ Stable |
| `tpt-proc-economics` | CAPEX/OPEX, NPV/IRR, levelized cost | ✅ Stable |
| `tpt-proc-database` | General process data store | ✅ Stable |
| `tpt-proc-extraction` | Liquid-liquid extraction | 🧪 Beta |
| `tpt-proc-membranes` | Solution-diffusion membrane separations | 🧪 Beta |
| `tpt-proc-crystallization` | Cooling crystallization, MSMPR | 🧪 Beta |
| `tpt-proc-reactors` | Steady-state CSTR and PFR | 🧪 Beta |
| `tpt-proc-catalysis` | Effectiveness factor, deactivation | 🧪 Beta |
| `tpt-proc-flowsheet` | Sequential-modular solver, tear streams + Wegstein | 🧪 Beta |
| `tpt-proc-energy-balance` | Energy accounting and utility summary | 🧪 Beta |
| `tpt-proc-water` | Water treatment train simulation | 🧪 Beta |
| `tpt-proc-pharma` | Pharmaceutical batch processes | 🧪 Beta |
| `tpt-proc-refining` | Refining stream characterization | 🧪 Beta |
| `tpt-proc-hydrogen` | Electrolysis, SMR, ATR | 🧪 Beta |
| `tpt-proc-dynamics` | Dynamic simulation, FOPDT, integrators | 🧪 Beta |
| `tpt-proc-control` | PID control and tuning | 🧪 Beta |
| `tpt-proc-pfd` | PFD representation and export (SVG/DOT/Mermaid) | 🧪 Beta |
| `tpt-proc` (CLI) | Config-driven flowsheet runner (TOML → JSON) | 🧪 Beta |
| `tpt-proc-wasm` | WebAssembly bindings | 🚧 Alpha |
| `tpt-proc-energy` | Pinch targets → `tpt-energy` system models | 🔗 Integration |
| `tpt-proc-materials` | Streams → `tpt-materials` compositions | 🔗 Integration |
| `tpt-proc-construction` | PFD → `tpt-construction` site/project | 🔗 Integration |

Every crate has its own `README.md` (with scope-and-limitations) and
`CHANGELOG.md` alongside its sources under [`crates/`](crates).

## Governance

- **License:** MIT OR Apache-2.0 (dual). Contributions are DCO-signed and
  CLA-free.
- **Releases:** SemVer, 6-week cadence.
- **Decisions:** Benevolent Dictator + public
  [RFC process](rfcs/0001-peng-robinson-eos.md) for new unit operations and
  thermodynamic models.
- **Security:** see [SECURITY.md](SECURITY.md) for private disclosure.

## The Process Connector

`tpt-process` is the flowsheet brain of the TPT ecosystem. Live
integration crates (in `crates/integration/`, pinned to the substrate
repositories) connect the domains today:

- [`tpt-proc-energy`](crates/integration/tpt-proc-energy) → `tpt-energy`:
  pinch utility targets become electrical loads and CHP capacity on an
  `EnergySystem` power-flow model.
- [`tpt-proc-materials`](crates/integration/tpt-proc-materials) →
  `tpt-materials`: species mixtures translate to elemental `Composition`s,
  with an NMC hydroxide precursor process design feeding battery work.
- [`tpt-proc-construction`](crates/integration/tpt-proc-construction) →
  `tpt-construction`: PFD units export as deterministic
  `Site`/`Element` records for plant layout.
- `tpt-medical` ← pharmaceutical batch processes — planned; awaiting code
  in the counterpart repository.

## Validation

Solver results are checked against golden reference files in
[`test-data/golden/`](test-data/golden) and against public benchmarks
(DIPPR, NIST, AIChE, API, ASHRAE). See
[CONTRIBUTING.md](CONTRIBUTING.md) for the verification & validation policy.

## License

Licensed under either of

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
