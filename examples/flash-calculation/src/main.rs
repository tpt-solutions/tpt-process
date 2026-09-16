//! Binary VLE flash of a water/methanol feed with the Peng-Robinson EOS.

use tpt_proc_core::Composition;
use tpt_proc_thermo_core::PropertyPackage;
use tpt_proc_thermo_database::ChemicalDatabase;
use tpt_proc_thermo_eos::CubicEos;
use tpt_proc_thermo_phase::FlashSolver;

fn main() {
    // Build a PR package from the built-in database.
    let db = ChemicalDatabase::builtin();
    let components = db
        .components_for(&["water", "methanol"])
        .expect("both components ship in the database");
    let eos = CubicEos::peng_robinson(components.clone());
    let package = PropertyPackage::new(components)
        .expect("valid components")
        .with_eos(std::sync::Arc::new(eos));
    let flash = FlashSolver::new(package);

    // Flash a 50/50 feed at 350 K, 1 atm.
    let feed = Composition::from_mole_fractions(&[0.5, 0.5]).expect("valid feed");
    let result = flash
        .pt_flash(&feed, 350.0, 101_325.0)
        .expect("flash converges");

    println!("PT flash of 50/50 water/methanol at 350 K, 1 atm:");
    println!("  vapor fraction: {:.4}", result.vapor_fraction);
    println!("  iterations:     {}", result.iterations);
    println!(
        "  water:    x = {:.4}  y = {:.4}",
        result.liquid_composition.get(0).unwrap_or(0.0),
        result.vapor_composition.get(0).unwrap_or(0.0)
    );
    println!(
        "  methanol: x = {:.4}  y = {:.4}",
        result.liquid_composition.get(1).unwrap_or(0.0),
        result.vapor_composition.get(1).unwrap_or(0.0)
    );

    // Pure water at its boiling point demonstrates the degenerate flag.
    let water_only = db.components_for(&["water"]).expect("water");
    let water_eos = CubicEos::peng_robinson(water_only.clone());
    let water_package = PropertyPackage::new(water_only)
        .expect("valid")
        .with_eos(std::sync::Arc::new(water_eos));
    let pure_flash = FlashSolver::new(water_package);
    let pure = Composition::from_mole_fractions(&[1.0]).expect("pure");
    let sat = pure_flash
        .pt_flash(&pure, 373.15, 101_325.0)
        .expect("converges");
    println!(
        "\nPure water at 100 °C, 1 atm: degenerate = {}",
        sat.degenerate
    );

    let water = db.get_component("water").expect("water");
    println!(
        "\nWater critical constants: Tc = {:.2} K, Pc = {:.2} MPa, Zc = {:.3}",
        water.critical_temperature,
        water.critical_pressure / 1e6,
        water.critical_compressibility()
    );
}
