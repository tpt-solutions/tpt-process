//! Membrane separations: CO2/CH4 gas flux through a polymer film
//! (solution-diffusion), ideal selectivity and permeance, then a
//! seawater-RO liquid calculation with osmotic pressure and rejection.

use tpt_proc_membranes::Membrane;

fn main() {
    // ---- Gas separation: CO2-selective cellulose-acetate-like film ----
    // 2 µm selective layer; CO2 = 30 Barrer, CH4 = 5 Barrer.
    let membrane = Membrane::new(2.0e-6, 30.0);
    let (p_feed_co2, p_perm_co2) = (1.2e6, 0.1e6); // Pa partial pressures
    let flux_co2 = membrane.gas_flux(p_feed_co2, p_perm_co2, 30.0);
    println!("Gas membrane: 2 µm film, CO2 permeability 30 Barrer");
    println!(
        "  CO2 flux J = P·Δp/ℓ = {:.3e} mol/(m²·s) at Δp = {:.2} MPa",
        flux_co2,
        (p_feed_co2 - p_perm_co2) / 1e6
    );
    let selectivity = Membrane::selectivity(30.0, 5.0);
    println!(
        "  ideal CO2/CH4 selectivity α = {:.1}, permeance = {:.0} GPU",
        selectivity,
        membrane.permeance_gpu(30.0)
    );
    assert!(flux_co2 > 0.0);
    assert!((selectivity - 6.0).abs() < 1e-12);

    // Permeated CO2 per m² of membrane per day.
    let seconds_per_day = 86_400.0;
    println!(
        "  CO2 production: {:.2} mol/(m²·day) = {:.2} kg/(m²·day)",
        flux_co2 * seconds_per_day,
        flux_co2 * seconds_per_day * 44.0e-3
    );

    // ---- Liquid separation: seawater RO ----
    // 35 g/L NaCl ≈ 600 mol/m³, van 't Hoff i ≈ 2 at 25 °C.
    let osmotic = Membrane::osmotic_pressure(600.0, 2.0, 298.15);
    println!(
        "\nSeawater RO: osmotic pressure π = {:.2} MPa",
        osmotic / 1e6
    );
    let water_permeability = 1.0e-11; // m/(s·Pa)
    let (dp, d_pi) = (5.5e6, osmotic);
    let flux_water = Membrane::liquid_flux(water_permeability, dp, d_pi);
    println!(
        "  net driving force ΔP − Δπ = {:.2} MPa -> water flux J_w = {:.2e} m/s = {:.2} L/(m²·h)",
        (dp - d_pi) / 1e6,
        flux_water,
        flux_water * 3.6e6
    );
    // Below the osmotic pressure there is no permeation.
    assert_eq!(Membrane::liquid_flux(water_permeability, 2.5e6, d_pi), 0.0);

    let rejection = Membrane::rejection(35.0, 0.35);
    println!(
        "  salt rejection R = 1 − cp/cf = {:.2} (cf = 35, cp = 0.35 g/L)",
        rejection
    );
    assert!(rejection > 0.98);
}
