//! Distillation shortcut and stage-to-stage benchmark.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use tpt_proc_distillation::{fug_shortcut, mccabe_thiele_stages, FugSpec};

fn spec() -> FugSpec {
    FugSpec {
        relative_volatility: 2.4,
        feed_mole_fraction: 0.5,
        light_key_recovery: 0.95,
        heavy_key_recovery: 0.05,
        actual_reflux_ratio: 1.5,
    }
}

fn bench_distillation(c: &mut Criterion) {
    c.bench_function("fug-shortcut", |b| {
        b.iter(|| fug_shortcut(black_box(&spec())).unwrap())
    });
    c.bench_function("mccabe-thiele", |b| {
        b.iter(|| {
            mccabe_thiele_stages(black_box(&spec()), black_box(0.95), black_box(0.05)).unwrap()
        })
    });
}

criterion_group!(benches, bench_distillation);
criterion_main!(benches);
