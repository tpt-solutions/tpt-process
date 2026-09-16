//! Peng-Robinson flash benchmark.

use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use tpt_proc_core::Composition;
use tpt_proc_thermo_core::PropertyPackage;
use tpt_proc_thermo_database::ChemicalDatabase;
use tpt_proc_thermo_eos::CubicEos;
use tpt_proc_thermo_phase::FlashSolver;

fn solver() -> FlashSolver {
    let db = ChemicalDatabase::builtin();
    let components = db.components_for(&["water", "methanol"]).unwrap();
    let eos = CubicEos::peng_robinson(components.clone());
    let package = PropertyPackage::new(components)
        .unwrap()
        .with_eos(Arc::new(eos));
    FlashSolver::new(package)
}

fn bench_pr_flash(c: &mut Criterion) {
    let flash = solver();
    let feed = Composition::from_mole_fractions(&[0.5, 0.5]).unwrap();
    c.bench_function("pr-pt-flash-binary", |b| {
        b.iter(|| {
            flash
                .pt_flash(black_box(&feed), black_box(350.0), black_box(101_325.0))
                .unwrap()
        })
    });
    c.bench_function("pr-bubble-point", |b| {
        b.iter(|| {
            flash
                .bubble_point_t(black_box(&feed), black_box(101_325.0))
                .unwrap()
        })
    });
}

criterion_group!(benches, bench_pr_flash);
criterion_main!(benches);
