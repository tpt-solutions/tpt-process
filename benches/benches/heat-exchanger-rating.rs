//! Heat-exchanger rating benchmark.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use tpt_proc_heat_exchangers::{FlowConfiguration, HeatExchanger};

fn bench_rating(c: &mut Criterion) {
    let hx = HeatExchanger::new(25.0, 900.0, FlowConfiguration::CounterCurrent);
    c.bench_function("epsilon-ntu-rating", |b| {
        b.iter(|| {
            hx.rate(
                black_box(2.0e3),
                black_box(4.0e3),
                black_box(f64::MAX),
                black_box(330.0),
                black_box(290.0),
            )
        })
    });
    c.bench_function("lmtd-sizing", |b| {
        b.iter(|| {
            hx.required_area(
                black_box(60.0e3),
                black_box(373.15),
                black_box(313.15),
                black_box(290.0),
                black_box(320.0),
            )
        })
    });
}

criterion_group!(benches, bench_rating);
criterion_main!(benches);
