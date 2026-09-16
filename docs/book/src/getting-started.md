# Getting Started

Add the crates you need; each domain is independent except for the core.

```toml
[dependencies]
tpt-proc-thermo-database = "0.1"
tpt-proc-thermo-phase = "0.1"
```

A first flash calculation:

```rust
use std::sync::Arc;
use tpt_proc_core::Composition;
use tpt_proc_thermo_core::PropertyPackage;
use tpt_proc_thermo_database::ChemicalDatabase;
use tpt_proc_thermo_eos::CubicEos;
use tpt_proc_thermo_phase::FlashSolver;

let db = ChemicalDatabase::builtin();
let components = db.components_for(&["water", "methanol"]).unwrap();
let eos = CubicEos::peng_robinson(components.clone());
let package = PropertyPackage::new(components).unwrap().with_eos(Arc::new(eos));
let flash = FlashSolver::new(package);

let feed = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
let result = flash.pt_flash(&feed, 350.0, 101_325.0).unwrap();
println!("β = {}", result.vapor_fraction);
```

The worked binaries under `examples/` cover flash, heat-exchanger rating,
distillation, reactors, pinch, psychrometrics, and water treatment.
