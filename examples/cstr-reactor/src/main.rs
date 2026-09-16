//! Steady-state CSTR and PFR with an exothermic first-order reaction,
//! comparing the analytic conversions and the adiabatic temperature rise.

use std::collections::BTreeMap;

use tpt_proc_reaction::{RateLaw, Reaction};
use tpt_proc_reactors::{CstrSpec, PfrSpec, ReactorSim};

fn main() {
    // A → B, k = 0.5 s⁻¹ at feed temperature (Ea = 0 isolates the
    // residence-time effect), ΔH_R = −60 kJ/mol.
    let mut stoich = BTreeMap::new();
    stoich.insert(0, -1.0);
    stoich.insert(1, 1.0);
    let mut orders = BTreeMap::new();
    orders.insert(0, 1.0);
    let reaction = Reaction::new(
        "A → B",
        stoich,
        RateLaw::Arrhenius {
            pre_exponential: 0.5,
            activation_energy: 0.0,
            orders,
        },
        -60.0e3,
    );
    let sim = ReactorSim::new(vec![reaction]);

    let volume = 2.0; // m³
    let feed_rate = 1.0; // m³/s → τ = 2 s
    let feed = [10.0, 0.0]; // mol/m³
    let t_in = 350.0; // K

    let cstr = sim.cstr(&CstrSpec { volume }, &feed, feed_rate, t_in, false);
    let pfr = sim.pfr(&PfrSpec { volume }, &feed, feed_rate, t_in, false);

    println!("First-order A → B, τ = 2 s, k = 0.5 s⁻¹:");
    println!("  CSTR: X = {:.4} (analytic 0.5000)", cstr.conversion[0]);
    println!(
        "  PFR:  X = {:.4} (analytic {:.4})",
        pfr.conversion[0],
        1.0 - (-1.0_f64).exp()
    );
    println!("  PFR outperforms CSTR for positive-order kinetics.");

    let adiabatic = sim.cstr(&CstrSpec { volume }, &feed, feed_rate, t_in, true);
    println!(
        "\nAdiabatic CSTR: outlet {:.3} K (ΔT = X·(−ΔH)·C₀/ρcp = {:.3} K)",
        adiabatic.temperature,
        adiabatic.temperature - t_in
    );
}
