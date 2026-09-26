//! Liquid-liquid extraction of a solute from a 100 kg feed: a single
//! equilibrium stage versus a crosscurrent battery with the same total
//! solvent, with the closed-form fraction remaining for crosscurrent
//! staging.

use tpt_proc_extraction::{fraction_remaining, single_stage, Extractor};

fn main() {
    // 100 kg feed carrying 10 kg solute; distribution K = Y/X = 8 on a
    // mass-of-solute/mass-of-solvent basis (dilute system).
    let (feed, solute, k) = (100.0, 10.0, 8.0);
    let total_solvent = 100.0;

    let (raffinate, extract) = single_stage(feed, solute, total_solvent, k);
    println!("Single stage with {total_solvent:.0} kg solvent (K = {k}):");
    println!("  raffinate solute = {raffinate:.3} kg, extract solute = {extract:.3} kg");
    println!("  fraction remaining = {:.4}", raffinate / solute);
    assert!((raffinate + extract - solute).abs() < 1e-9);

    // Same total solvent, split equally over N crosscurrent stages:
    // staging beats one big contact.
    let total_solvent = 300.0;
    println!("\nCrosscurrent extraction, {total_solvent:.0} kg solvent total:");
    println!("  stages   raffinate solute [kg]   fraction remaining");
    for stages in [1, 2, 3, 5] {
        let (raffinate, extract) =
            Extractor::new(stages).crosscurrent(feed, solute, total_solvent, k);
        println!(
            "  {stages:>5}    {raffinate:>18.4}    {:>17.5}",
            raffinate / solute
        );
        if stages == 1 {
            assert!(extract > 0.0);
        }
    }

    // Closed form: per-stage extraction factor E = K·S_stage/F, so
    // fraction remaining = (1/(1+E))^N.
    let stages = 3;
    let e = k * (total_solvent / f64::from(stages)) / feed;
    let closed_form = solute * fraction_remaining(stages, e);
    let (staged, _) = Extractor::new(stages).crosscurrent(feed, solute, total_solvent, k);
    println!("\n3-stage check: closed form {closed_form:.5} kg vs staged balance {staged:.5} kg");
    assert!((closed_form - staged).abs() < 1e-9);

    let (one_big, _) = Extractor::new(1).crosscurrent(feed, solute, total_solvent, k);
    assert!(
        staged < one_big,
        "crosscurrent staging must beat one contact"
    );
}
