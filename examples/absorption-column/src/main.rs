//! Kremser-design absorption of a dilute solute from a gas into a lean
//! solvent, plus the symmetric stripping calculation for regenerating the
//! rich solvent.

use tpt_proc_absorption::{kremser_fraction_remaining, Absorber};

fn main() {
    let feed_solute = 0.05; // mole fraction solute in the feed gas

    // Effect of the absorption factor A = L/(m·G) on a 6-stage column.
    println!("Kremser absorption, 6 theoretical stages, feed y_in = {feed_solute}:");
    for absorption_factor in [0.8, 1.0, 1.4, 2.0] {
        let result = Absorber::new(6).absorb(feed_solute, absorption_factor);
        println!(
            "  A = {absorption_factor:.1}: y_out = {:.5}, recovery = {:.2}%",
            result.exiting_solute,
            100.0 * result.solute_recovery
        );
    }

    // Effect of stage count at a fixed A = 1.5.
    let absorber = Absorber::new(6);
    println!("\nStage count at A = 1.5:");
    for stages in [2, 4, 6, 8] {
        let frac = kremser_fraction_remaining(stages, 1.5);
        println!(
            "  N = {stages}: fraction remaining = {frac:.5} ({:.1}% removed)",
            100.0 * (1.0 - frac)
        );
    }

    let design = absorber.absorb(feed_solute, 1.5);
    println!(
        "\nDesign point (N = 6, A = 1.5): fraction remaining = {:.5}, recovery = {:.2}%, y_out = {:.5}",
        design.fraction_remaining,
        100.0 * design.solute_recovery,
        design.exiting_solute
    );
    assert!(design.solute_recovery > 0.95);

    // Regeneration: strip the rich solvent with S = 1/A symmetric form.
    let rich_solute = 0.10;
    let stripper = Absorber::new(4).strip(rich_solute, 1.8);
    println!(
        "\nStripping 4 stages at S = 1.8: fraction remaining = {:.5}, stripped = {:.2}%",
        stripper.fraction_remaining,
        100.0 * stripper.solute_recovery
    );
    // The Kremser form is symmetric: strip(S) == absorb(S).
    let mirror = Absorber::new(4).absorb(rich_solute, 1.8);
    assert!((stripper.fraction_remaining - mirror.fraction_remaining).abs() < 1e-12);
}
