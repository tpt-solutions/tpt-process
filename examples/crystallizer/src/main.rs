//! Cooling crystallization of KNO3 from water (yield from the solubility
//! curve) and an MSMPR population-balance estimate of nucleation rate and
//! dominant crystal size.

use tpt_proc_crystallization::{cooling_yield, crystal_mass_kg, Crystallizer};

fn main() {
    // KNO3 in water: solubility ≈ 0.51 kg/kg free solvent at 80 °C falling
    // to ≈ 0.15 kg/kg at 20 °C.
    let (c_hot, c_cold) = (0.51, 0.15);
    let solution = 1000.0; // kg of initial solution

    let yield_fraction = cooling_yield(solution, c_hot, c_cold);
    let solvent = solution / (1.0 + c_hot);
    let crystals = crystal_mass_kg(solvent, c_hot, c_cold);
    println!(
        "Cooling crystallization of KNO3: {solution:.0} kg solution, c = {c_hot} -> {c_cold} kg/kg"
    );
    println!(
        "  crystals = {crystals:.1} kg, yield = {:.1}% of the initial solution",
        100.0 * yield_fraction
    );
    println!(
        "  final mother liquor: {:.1} kg holding {:.1} kg dissolved KNO3",
        solution - crystals,
        solvent * c_cold
    );
    assert!((crystals - solvent * (c_hot - c_cold)).abs() < 1e-9);
    assert!((yield_fraction - crystals / solution).abs() < 1e-12);

    // MSMPR crystallizer: mixed suspension, mixed product removal.
    let crystallizer = Crystallizer::new(2.0); // m³ of suspension
    let growth = 1.0e-8; // m/s
    let residence = 3600.0; // s (1 h)
    let nucleation = crystallizer.msmpr_nucleation(growth, 1.0e8, 200.0);
    let dominant = crystallizer.msmpr_dominant_size(growth, residence);
    println!("\nMSMPR crystallizer (V = 2 m³, G = {growth:.1e} m/s, τ = {residence:.0} s):");
    println!("  nucleation rate B0 = {nucleation:.3e} crystals/(m³·s)");
    println!("  dominant size L_D = 3.67·G·τ = {:.1} µm", dominant * 1e6);
    assert!(nucleation > 0.0);

    // Longer residence coarsens the product linearly.
    let coarse = crystallizer.msmpr_dominant_size(growth, 2.0 * residence);
    println!(
        "  doubling τ to {:.0} s gives L_D = {:.1} µm",
        2.0 * residence,
        coarse * 1e6
    );
    assert!((coarse / dominant - 2.0).abs() < 1e-12);
}
