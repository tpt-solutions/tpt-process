//! Water treatment train simulation: coagulation → flocculation →
//! sedimentation → filtration → ion exchange → reverse osmosis →
//! disinfection.
//!
//! Each process transforms a [`WaterQuality`] stream; the train simulates
//! them in declaration order.
//!
//! # Example
//!
//! ```
//! use tpt_proc_water::{Process, Train, WaterQuality};
//!
//! let raw = WaterQuality::new(25.0, 800.0, 250.0, 10_000.0);
//! let train = Train::new(vec![
//!     Process::Coagulation { dose_mg_l: 30.0 },
//!     Process::Filtration { removal_fraction: 0.95 },
//!     Process::ReverseOsmosis { rejection: 0.98, recovery: 0.75 },
//!     Process::Disinfection { ct_value: 40.0, removal_rate: 4.0 },
//! ]);
//! let out = train.simulate(&raw);
//! assert!(out.turbidity_ntu < 1.0);
//! assert!(out.tds_mg_l < 800.0 * 0.05);
//! assert!(out.coliform_cfu_ml < 1.0);
//! ```

#![forbid(unsafe_code)]

/// Water quality state (the process stream of the treatment train).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterQuality {
    /// Turbidity, NTU.
    pub turbidity_ntu: f64,
    /// Total dissolved solids, mg/L.
    pub tds_mg_l: f64,
    /// Hardness as CaCO₃, mg/L.
    pub hardness_mg_l: f64,
    /// Coliform count, CFU/mL.
    pub coliform_cfu_ml: f64,
}

impl WaterQuality {
    /// Creates a quality state.
    #[must_use]
    pub const fn new(
        turbidity_ntu: f64,
        tds_mg_l: f64,
        hardness_mg_l: f64,
        coliform_cfu_ml: f64,
    ) -> Self {
        Self {
            turbidity_ntu,
            tds_mg_l,
            hardness_mg_l,
            coliform_cfu_ml,
        }
    }
}

/// One treatment process.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Process {
    /// Coagulant dosing: sweeps colloids; nominal turbidity removal.
    Coagulation {
        /// Coagulant dose, mg/L.
        dose_mg_l: f64,
    },
    /// Flocculation contact time, minutes (grows flocs; small extra
    /// turbidity removal).
    Flocculation {
        /// Retention time, min.
        retention_time_min: f64,
    },
    /// Gravity settling with a fractional turbidity removal.
    Sedimentation {
        /// Fraction of suspended turbidity removed, [0, 1].
        removal_fraction: f64,
    },
    /// Media filtration.
    Filtration {
        /// Fraction of remaining turbidity removed, [0, 1].
        removal_fraction: f64,
    },
    /// Ion exchange softening with a finite capacity.
    IonExchange {
        /// Exchange capacity, mg/L as CaCO₃ treated.
        capacity_mg_l: f64,
    },
    /// Reverse osmosis desalination.
    ReverseOsmosis {
        /// Salt rejection fraction, [0, 1].
        rejection: f64,
        /// Permeate recovery fraction, [0, 1].
        recovery: f64,
    },
    /// Disinfection by CT (concentration × time) concept:
    /// log removal = k·CT.
    Disinfection {
        /// CT value, mg·min/L.
        ct_value: f64,
        /// Log-removal rate per unit CT, 1/(mg·min/L).
        removal_rate: f64,
    },
}

/// A treatment train: processes applied in order.
#[derive(Clone, Debug, PartialEq)]
pub struct Train {
    processes: Vec<Process>,
}

impl Train {
    /// Creates a train.
    #[must_use]
    pub fn new(processes: Vec<Process>) -> Self {
        Self { processes }
    }

    /// The ordered process list.
    pub fn processes(&self) -> &[Process] {
        &self.processes
    }

    /// Simulates the train on `raw` water.
    #[must_use]
    pub fn simulate(&self, raw: &WaterQuality) -> WaterQuality {
        let mut q = *raw;
        for process in &self.processes {
            q = apply(process, &q);
        }
        q
    }
}

/// Applies a single process to a quality state.
#[must_use]
pub fn apply(process: &Process, q: &WaterQuality) -> WaterQuality {
    let mut out = *q;
    match process {
        Process::Coagulation { dose_mg_l } => {
            // Sweep floc removal: up to ~70% of turbidity per typical dose.
            let removal = (dose_mg_l / 50.0).clamp(0.0, 0.7);
            out.turbidity_ntu *= 1.0 - removal;
            out.coliform_cfu_ml *= 1.0 - 0.5 * removal;
        }
        Process::Flocculation { retention_time_min } => {
            // Gentle additional removal that saturates with contact time.
            out.turbidity_ntu *= 0.8 + 0.2 * (-retention_time_min / 30.0).exp();
        }
        Process::Sedimentation { removal_fraction } => {
            out.turbidity_ntu *= 1.0 - removal_fraction.clamp(0.0, 1.0);
        }
        Process::Filtration { removal_fraction } => {
            out.turbidity_ntu *= 1.0 - removal_fraction.clamp(0.0, 1.0);
            out.coliform_cfu_ml *= 1.0 - 0.9 * removal_fraction.clamp(0.0, 1.0);
        }
        Process::IonExchange { capacity_mg_l } => {
            let exchanged = q.hardness_mg_l.min(*capacity_mg_l);
            out.hardness_mg_l -= exchanged;
            out.tds_mg_l -= exchanged * 0.85; // Na swaps for Ca/Mg
        }
        Process::ReverseOsmosis {
            rejection,
            recovery,
        } => {
            let r = rejection.clamp(0.0, 1.0);
            // Permeate TDS = feed·(1−r); recovery concentrates the reject
            // but the product quality is the permeate value.
            out.tds_mg_l *= 1.0 - r;
            out.hardness_mg_l *= 1.0 - r.clamp(0.9, 1.0);
            out.coliform_cfu_ml *= 1.0 - r.clamp(0.9, 1.0);
            let _ = recovery;
        }
        Process::Disinfection {
            ct_value,
            removal_rate,
        } => {
            let log_removal = removal_rate * ct_value;
            out.coliform_cfu_ml *= 10.0_f64.powf(-log_removal);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conventional_train_meets_drinking_standards() {
        let raw = WaterQuality::new(25.0, 800.0, 250.0, 10_000.0);
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
            Process::Disinfection {
                ct_value: 40.0,
                removal_rate: 3.0,
            },
        ]);
        let out = train.simulate(&raw);
        assert!(out.turbidity_ntu < 1.0, "turbidity {}", out.turbidity_ntu);
        assert!(
            out.coliform_cfu_ml < 1.0,
            "coliform {}",
            out.coliform_cfu_ml
        );
    }

    #[test]
    fn ion_exchange_softens_within_capacity() {
        let hard = WaterQuality::new(5.0, 500.0, 300.0, 10.0);
        let softened = apply(
            &Process::IonExchange {
                capacity_mg_l: 200.0,
            },
            &hard,
        );
        assert!((softened.hardness_mg_l - 100.0).abs() < 1e-9);
        // Beyond capacity nothing more is removed.
        let softened2 = apply(
            &Process::IonExchange {
                capacity_mg_l: 500.0,
            },
            &hard,
        );
        assert!(softened2.hardness_mg_l.abs() < 1e-9);
    }

    #[test]
    fn reverse_osmosis_desalinates() {
        let brackish = WaterQuality::new(1.0, 3000.0, 200.0, 100.0);
        let out = apply(
            &Process::ReverseOsmosis {
                rejection: 0.97,
                recovery: 0.7,
            },
            &brackish,
        );
        assert!(out.tds_mg_l < 3000.0 * 0.05, "TDS {}", out.tds_mg_l);
        assert!(out.hardness_mg_l < 20.0);
    }

    #[test]
    fn disinfection_log_removal() {
        let infested = WaterQuality::new(2.0, 400.0, 100.0, 1.0e6);
        let out = apply(
            &Process::Disinfection {
                ct_value: 2.0,
                removal_rate: 2.0,
            },
            &infested,
        );
        // 4-log removal: 1e6 → 100.
        assert!((out.coliform_cfu_ml - 100.0).abs() < 1.0);
    }

    #[test]
    fn empty_train_is_a_no_op() {
        let raw = WaterQuality::new(10.0, 500.0, 100.0, 50.0);
        let out = Train::new(vec![]).simulate(&raw);
        assert_eq!(out, raw);
    }
}
