//! A surface-water treatment train with RO polishing, from raw water to
//! potable-quality permeate.

use tpt_proc_water::{Process, Train, WaterQuality};

fn main() {
    let raw = WaterQuality::new(28.0, 850.0, 240.0, 50_000.0);
    println!(
        "Raw water: turbidity {:.0} NTU, TDS {:.0} mg/L, hardness {:.0} mg/L, coliform {:.0} CFU/mL",
        raw.turbidity_ntu, raw.tds_mg_l, raw.hardness_mg_l, raw.coliform_cfu_ml
    );

    let train = Train::new(vec![
        Process::Coagulation { dose_mg_l: 40.0 },
        Process::Flocculation {
            retention_time_min: 30.0,
        },
        Process::Sedimentation {
            removal_fraction: 0.9,
        },
        Process::Filtration {
            removal_fraction: 0.95,
        },
        Process::IonExchange {
            capacity_mg_l: 150.0,
        },
        Process::ReverseOsmosis {
            rejection: 0.97,
            recovery: 0.75,
        },
        Process::Disinfection {
            ct_value: 4.0,
            removal_rate: 2.0,
        },
    ]);

    let out = train.simulate(&raw);
    println!(
        "Product:  turbidity {:.2} NTU, TDS {:.0} mg/L, hardness {:.0} mg/L, coliform {:.2} CFU/mL",
        out.turbidity_ntu, out.tds_mg_l, out.hardness_mg_l, out.coliform_cfu_ml
    );

    // Process-by-process trace.
    println!("\nStep trace (turbidity / TDS):");
    let mut q = raw;
    let steps: Vec<Process> = train.processes().to_vec();
    for (i, process) in steps.iter().enumerate() {
        q = tpt_proc_water::apply(process, &q);
        println!(
            "  {}. {:.2} NTU / {:.0} mg/L",
            i + 1,
            q.turbidity_ntu,
            q.tds_mg_l
        );
    }
}
