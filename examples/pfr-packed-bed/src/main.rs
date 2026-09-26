//! A first-order gas reaction in an ideal PFR vs. CSTR, then the packed-bed
//! reality: internal pellet diffusion derates the rate constant through the
//! Thiele-modulus effectiveness factor, and the catalyst deactivates over
//! a year online.

use std::collections::BTreeMap;

use tpt_proc_catalysis::{
    deactivation_factor, effectiveness_factor, thiele_modulus, DeactivationModel, PelletGeometry,
};
use tpt_proc_reaction::{RateLaw, Reaction};
use tpt_proc_reactors::{CstrSpec, PfrSpec, ReactorSim};

fn first_order(k: f64) -> Reaction {
    let mut stoich = BTreeMap::new();
    stoich.insert(0, -1.0);
    stoich.insert(1, 1.0);
    let mut orders = BTreeMap::new();
    orders.insert(0, 1.0);
    Reaction::new(
        "A -> B",
        stoich,
        RateLaw::Arrhenius {
            pre_exponential: k,
            activation_energy: 0.0,
            orders,
        },
        -60.0e3,
    )
}

fn main() {
    // A -> B with k = 0.5 s⁻¹ at feed temperature; τ = 2 s at 1 m³/s
    // through a 2 m³ reactor, isothermal at 623 K (typical packed-bed
    // service temperature).
    let (k, feed_rate, t_in) = (0.5, 1.0, 623.0);
    let feed = [10.0, 0.0]; // mol/m³
    let sim = ReactorSim::new(vec![first_order(k)]);

    let cstr = sim.cstr(&CstrSpec { volume: 2.0 }, &feed, feed_rate, t_in, false);
    let pfr = sim.pfr(&PfrSpec { volume: 2.0 }, &feed, feed_rate, t_in, false);
    println!("First-order A -> B, k = {k} s⁻¹, τ = 2 s:");
    println!(
        "  CSTR: X = {:.4} (analytic kτ/(1+kτ) = 0.5000)",
        cstr.conversion[0]
    );
    println!(
        "  PFR:  X = {:.4} (analytic 1 − e^(−kτ) = {:.4})",
        pfr.conversion[0],
        1.0 - (-k * 2.0_f64).exp()
    );
    assert!(pfr.conversion[0] > cstr.conversion[0]);

    // Packed bed: 5 mm catalyst pellets, Deff = 2e-6 m²/s -> Thiele
    // modulus φ and effectiveness factor η for a sphere. The diffusion-
    // limited bed behaves like an ideal PFR with k_eff = η·k.
    let (pellet_r, d_eff) = (5.0e-3, 2.0e-6);
    let phi = thiele_modulus(pellet_r, k, d_eff);
    let eta = effectiveness_factor(phi, PelletGeometry::Sphere, 1.0);
    let k_eff = eta * k;
    println!("\nPacked bed: pellets R = {pellet_r:.0e} m, Deff = {d_eff:.1e} m²/s");
    println!("  Thiele modulus φ = {phi:.3}, effectiveness η = {eta:.3}, k_eff = {k_eff:.4} s⁻¹");
    assert!(eta < 1.0 && eta > 0.5);

    let bed = ReactorSim::new(vec![first_order(k_eff)]);
    let packed = bed.pfr(&PfrSpec { volume: 2.0 }, &feed, feed_rate, t_in, false);
    println!(
        "  packed-bed PFR at k_eff: X = {:.4} (vs {:.4} with undiffused pellets)",
        packed.conversion[0], pfr.conversion[0]
    );
    assert!(packed.conversion[0] < pfr.conversion[0]);

    // Deactivation: coking with kd = 0.05/month, one year online.
    let months = 12.0;
    let fresh = deactivation_factor(0.05, 0.0, DeactivationModel::Exponential);
    let year = deactivation_factor(0.05, months, DeactivationModel::Exponential);
    let power_law = deactivation_factor(0.05, months, DeactivationModel::PowerLaw);
    println!("\nDeactivation after {months:.0} months (kd = 0.05/month):");
    println!("  fresh activity = {fresh:.2}, exponential = {year:.3}, power law = {power_law:.3}");
    assert!(year < fresh && power_law > year);
}
