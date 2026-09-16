//! Binary distillation design: FUG shortcut cross-checked against
//! McCabe-Thiele stepping.

use tpt_proc_distillation::{fug_shortcut, mccabe_thiele_stages, FugSpec};

fn main() {
    // 50/50 benzene-toluene feed, recover 95% of the light key and leave
    // 5% of the heavy key overhead at R = 1.5 (α ≈ 2.4).
    let spec = FugSpec {
        relative_volatility: 2.4,
        feed_mole_fraction: 0.5,
        light_key_recovery: 0.95,
        heavy_key_recovery: 0.05,
        actual_reflux_ratio: 1.5,
    };

    let fug = fug_shortcut(&spec).expect("FUG converges");
    println!("FUG shortcut (α = 2.4, R = 1.5):");
    println!("  N_min (Fenske):       {:.2}", fug.min_stages);
    println!("  R_min (Underwood):    {:.3}", fug.min_reflux);
    println!("  N actual (Gilliland): {:.1}", fug.actual_stages);

    // Distillate is x_D = 0.95 (binary basis), bottoms x_B = 0.05.
    let mccabe = mccabe_thiele_stages(&spec, 0.95, 0.05).expect("stepping converges");
    println!("\nMcCabe-Thiele (constant α):");
    println!("  stages:     {:.1}", mccabe.stages);
    println!("  feed stage: {:.1}", mccabe.feed_stage);
    println!("  R_min:      {:.3}", mccabe.min_reflux);

    let deviation = (mccabe.stages - fug.actual_stages).abs() / fug.actual_stages * 100.0;
    println!("\nFUG vs McCabe deviation: {deviation:.1}%");
}
