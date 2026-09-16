# tpt-process

A fully open-source, MIT-licensed computational engine for chemical and process
engineering: thermodynamics, fluid flow, heat transfer, separations, and
reaction engineering.

> **100% Open Source. Chemical & Process Engineering Without Vendor Lock-in.
> No GPL Traps. No Desktop Monopolies.**

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

| Crate | Description | Status |
|---|---|---|
| `tpt-proc-core` | Core stream and unit types | ✅ Stable |
| `tpt-proc-topology` | PFD graph analysis, recycles, tear streams | ✅ Stable |
| `tpt-proc-units` | Unit operation scaffolding, mixer/splitter | ✅ Stable |
| `tpt-proc-thermo-core` | Property package framework | ✅ Stable |
| `tpt-proc-thermo-eos` | Peng-Robinson, SRK equations of state | ✅ Stable |
| `tpt-proc-thermo-activity` | Wilson, NRTL, UNIQUAC, UNIFAC | ✅ Stable |
| `tpt-proc-thermo-phase` | Flash calculations, VLE, bubble/dew point | ✅ Stable |
| `tpt-proc-thermo-database` | Built-in chemical database | ✅ Stable |
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
| `tpt-proc-extraction` | Liquid-liquid extraction | ✅ Stable |
| `tpt-proc-membranes` | Solution-diffusion membrane separations | ✅ Stable |
| `tpt-proc-crystallization` | Cooling crystallization, MSMPR | ✅ Stable |
| `tpt-proc-reaction` | Kinetics: Arrhenius, LHHW, Michaelis-Menten | ✅ Stable |
| `tpt-proc-reactors` | Batch, CSTR, PFR, packed bed, fluidized bed | ✅ Stable |
| `tpt-proc-catalysis` | Effectiveness factor, deactivation | ✅ Stable |
| `tpt-proc-flowsheet` | Sequential modular + equation-oriented | ✅ Stable |
| `tpt-proc-mass-balance` | Balances and data reconciliation | ✅ Stable |
| `tpt-proc-energy-balance` | Energy accounting and utility summary | ✅ Stable |
| `tpt-proc-optimization` | Golden section, Nelder-Mead, gradient | ✅ Stable |
| `tpt-proc-hvac` | Psychrometrics and air systems | ✅ Stable |
| `tpt-proc-water` | Water treatment train simulation | ✅ Stable |
| `tpt-proc-hydrogen` | Electrolysis, SMR, ATR | ✅ Stable |
| `tpt-proc-dynamics` | Dynamic simulation, FOPDT, integrators | ✅ Stable |
| `tpt-proc-control` | PID control and tuning | ✅ Stable |
| `tpt-proc-pharma` | Pharmaceutical batch processes | ✅ Stable |
| `tpt-proc-refining` | Refining stream characterization | ✅ Stable |
| `tpt-proc-database` | General process data store | ✅ Stable |
| `tpt-proc-pfd` | PFD representation and export (SVG/DOT) | ✅ Stable |
| `tpt-proc-economics` | CAPEX/OPEX, NPV/IRR, levelized cost | ✅ Stable |
| `tpt-proc-wasm` | WebAssembly bindings | 🚧 Alpha |
| `tpt-proc-energy` | Pinch targets → `tpt-energy` system models | 🔗 Integration |
| `tpt-proc-materials` | Streams → `tpt-materials` compositions | 🔗 Integration |
| `tpt-proc-construction` | PFD → `tpt-construction` site/project | 🔗 Integration |

## Quick Start

```rust
use std::sync::Arc;

use tpt_proc_core::Composition;
use tpt_proc_thermo_core::PropertyPackage;
use tpt_proc_thermo_database::ChemicalDatabase;
use tpt_proc_thermo_eos::CubicEos;
use tpt_proc_thermo_phase::FlashSolver;

// 50/50 water/methanol feed flashed at 350 K, 1 atm (Peng-Robinson).
let db = ChemicalDatabase::builtin();
let components = db.components_for(&["water", "methanol"]).unwrap();
let eos = CubicEos::peng_robinson(components.clone());
let package = PropertyPackage::new(components).unwrap().with_eos(Arc::new(eos));
let flash = FlashSolver::new(package);

let feed = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
let result = flash.pt_flash(&feed, 350.0, 101_325.0).unwrap();

println!("Vapor fraction: {}", result.vapor_fraction);
println!("Vapor composition: {:?}", result.vapor_composition.as_slice());
```

Run the worked examples:

```sh
git clone https://github.com/tpt-solutions/tpt-process
cd tpt-process
cargo run --release -p example-flash-calculation
```

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
