//! Pipe-network solver benchmark (two-loop grid).

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use tpt_proc_network::{NetworkNode, NetworkPipe, PipeNetwork};

fn network() -> PipeNetwork {
    let mut net = PipeNetwork::new();
    net.add_node(
        1,
        NetworkNode {
            head: 100.0,
            is_reservoir: true,
            demand: 0.0,
        },
    );
    net.add_node(
        2,
        NetworkNode {
            head: 90.0,
            is_reservoir: true,
            demand: 0.0,
        },
    );
    net.add_node(
        3,
        NetworkNode {
            head: 0.0,
            is_reservoir: false,
            demand: 0.10,
        },
    );
    net.add_node(
        4,
        NetworkNode {
            head: 0.0,
            is_reservoir: false,
            demand: 0.05,
        },
    );
    net.add_node(
        5,
        NetworkNode {
            head: 0.0,
            is_reservoir: false,
            demand: 0.08,
        },
    );
    net.add_node(
        6,
        NetworkNode {
            head: 0.0,
            is_reservoir: false,
            demand: 0.07,
        },
    );
    for (from, to, resistance) in [
        (1u64, 3u64, 300.0),
        (1, 4, 500.0),
        (2, 5, 400.0),
        (2, 6, 600.0),
        (3, 4, 200.0),
        (3, 5, 700.0),
        (4, 6, 350.0),
        (5, 6, 250.0),
    ] {
        net.add_pipe(NetworkPipe {
            from,
            to,
            resistance,
        });
    }
    net
}

fn bench_network(c: &mut Criterion) {
    let net = network();
    c.bench_function("hardy-cross", |b| {
        b.iter(|| {
            net.solve_hardy_cross(black_box(1e-8), black_box(500))
                .unwrap()
        })
    });
    c.bench_function("newton-raphson", |b| {
        b.iter(|| {
            net.solve_newton_raphson(black_box(1e-8), black_box(100))
                .unwrap()
        })
    });
    c.bench_function("linear-theory", |b| {
        b.iter(|| {
            net.solve_linear_theory(black_box(1e-8), black_box(200))
                .unwrap()
        })
    });
}

criterion_group!(benches, bench_network);
criterion_main!(benches);
