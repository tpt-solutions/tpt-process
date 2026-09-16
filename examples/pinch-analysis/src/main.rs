//! Pinch analysis of a four-stream problem: utility targets, pinch
//! location, composite curves, and network synthesis.

use tpt_proc_heat_network::{PinchAnalysis, ProcessStream};

fn main() {
    let pinch = PinchAnalysis::new(
        vec![
            ProcessStream::hot(433.15, 318.15, 15.0), // kW/K
            ProcessStream::hot(493.15, 333.15, 25.0),
        ],
        vec![
            ProcessStream::cold(318.15, 448.15, 10.0),
            ProcessStream::cold(333.15, 408.15, 20.0),
        ],
        10.0, // K
    );

    let targets = pinch.minimum_utilities();
    println!("Pinch targets at ΔT_min = 10 K:");
    println!("  Q_H,min = {:.0} kW", targets.min_heating_duty);
    println!("  Q_C,min = {:.0} kW", targets.min_cooling_duty);
    if let (Some(hot), Some(cold)) = (targets.pinch_hot_temp, targets.pinch_cold_temp) {
        println!(
            "  pinch: {:.1} °C hot / {:.1} °C cold",
            hot - 273.15,
            cold - 273.15
        );
    }

    // Composite curves: piecewise (duty, temperature) polylines.
    let curves = pinch.composite_curves();
    println!("\nHot composite curve points: {}", curves.hot.len());
    println!("Cold composite curve points: {}", curves.cold.len());

    // Grand composite curve for utility-level placement.
    let gcc = pinch.grand_composite_curve();
    println!(
        "Grand composite (top): {:.0} kW at {:.1} K",
        gcc.points[0].1, gcc.points[0].0
    );

    // Synthesize a minimum-utility network.
    let network = pinch.network_synthesis();
    println!(
        "\nSynthesized network: {} process-process matches",
        network.matches.len()
    );
    for m in &network.matches {
        println!(
            "  hot[{}] ↔ cold[{}]: {:.0} kW",
            m.hot_index, m.cold_index, m.duty
        );
    }
}
