//! A two-reservoir, two-loop pipe network solved by all three classic
//! methods — Hardy Cross, Newton-Raphson, and Linear Theory — which the
//! network crate requires to agree.

use tpt_proc_fluid::{FluidProperties, PipeFlow};
use tpt_proc_network::{pipe_resistance, NetworkNode, NetworkPipe, PipeNetwork};

fn main() {
    // Reservoirs 1 and 2 feed four demand junctions (3-6) through a
    // two-loop grid of commercial-steel pipes (SI units throughout).
    let fluid = FluidProperties {
        density: 998.0,
        viscosity: 1.0e-3,
    };

    // (from, to, diameter [m], length [m])
    let layout = [
        (1u64, 3u64, 0.30, 1000.0),
        (1, 4, 0.25, 1500.0),
        (2, 5, 0.25, 1200.0),
        (2, 6, 0.30, 900.0),
        (3, 4, 0.20, 750.0),
        (3, 5, 0.20, 1100.0),
        (4, 6, 0.20, 800.0),
        (5, 6, 0.20, 650.0),
    ];
    // Junction demands, m³/s.
    let demands = [(3u64, 0.10), (4, 0.05), (5, 0.08), (6, 0.07)];

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
    for (id, demand) in demands {
        net.add_node(
            id,
            NetworkNode {
                head: 0.0,
                is_reservoir: false,
                demand,
            },
        );
    }

    // Resistance r = 8·f·L/(π²·g·D⁵) with the Darcy friction factor from
    // tpt-proc-fluid (Colebrook-White) evaluated at a nominal 1 m/s.
    println!("Pipes (frozen-friction quadratic law h = r·Q·|Q|):");
    for (from, to, diameter, length) in layout {
        let hydraulics = PipeFlow::new(diameter, length, 4.6e-5).with_fluid(fluid);
        let f = hydraulics.friction_factor(1.0);
        let r = pipe_resistance(f, length, diameter);
        println!(
            "  {from}->{to}: D = {diameter:.2} m, L = {length:.0} m, f = {f:.4}, r = {r:.1} s²/m⁵"
        );
        net.add_pipe(NetworkPipe {
            from,
            to,
            resistance: r,
        });
    }

    let hardy = net
        .solve_hardy_cross(1e-9, 1000)
        .expect("Hardy Cross converges");
    let newton = net
        .solve_newton_raphson(1e-10, 100)
        .expect("Newton-Raphson converges");
    let linear = net
        .solve_linear_theory(1e-10, 200)
        .expect("Linear Theory converges");
    assert!(hardy.converged && newton.converged && linear.converged);

    println!("\nPipe flows (m³/s, declared from->to positive):");
    println!("  pipe    Hardy Cross  Newton-Raphson  Linear Theory");
    for (i, pipe) in net.pipes.iter().enumerate() {
        println!(
            "  {:>3}->{:<3}{:>12.5}{:>13.5}{:>14.5}",
            pipe.from, pipe.to, hardy.pipe_flows[i], newton.pipe_flows[i], linear.pipe_flows[i]
        );
    }

    // The three solvers must agree (network-crate contract).
    let max_spread = hardy
        .pipe_flows
        .iter()
        .zip(&newton.pipe_flows)
        .zip(&linear.pipe_flows)
        .map(|((h, n), l)| (h - n).abs().max((h - l).abs()))
        .fold(0.0, f64::max);
    println!("\nMax flow spread across the three solvers: {max_spread:.2e} m³/s");
    assert!(max_spread < 2e-3, "solvers disagree: {max_spread}");

    // Global continuity: reservoir supply equals total demand.
    let total_demand: f64 = demands.iter().map(|(_, d)| d).sum();
    println!(
        "Total demand: {total_demand:.3} m³/s (Hardy Cross iterations: {})",
        hardy.iterations
    );

    let mut supply = 0.0;
    for (i, pipe) in net.pipes.iter().enumerate() {
        if net.nodes[&pipe.from].is_reservoir {
            supply += hardy.pipe_flows[i];
        }
    }
    println!("Reservoir supply: {supply:.4} m³/s");
    assert!((supply - total_demand).abs() < 1e-3);

    println!("\nNewton-Raphson junction heads:");
    for (id, head) in &newton.junction_heads {
        println!("  junction {id}: H = {head:.3} m");
    }
}
