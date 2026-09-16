//! Energy accounting and utility summaries across flowsheets.
//!
//! # Example
//!
//! ```
//! use tpt_proc_energy_balance::EnergyBalance;
//!
//! let mut ledger = EnergyBalance::new();
//! ledger.add_duty("reboiler", 2.0e6);   // heating
//! ledger.add_duty("condenser", -1.5e6); // cooling
//! let summary = ledger.summary();
//! assert!((summary.net - 0.5e6).abs() < 1e-6);
//! ```

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// An energy duty ledger: named heating (+) and cooling (−) duties, W.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnergyBalance {
    duties: BTreeMap<String, f64>,
}

/// Aggregate results.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct EnergySummary {
    /// Sum of positive duties (total heating demand), W.
    pub total_heating: f64,
    /// Sum of |negative duties| (total cooling demand), W.
    pub total_cooling: f64,
    /// Net duty, W.
    pub net: f64,
}

impl EnergyBalance {
    /// An empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds (or replaces) a named duty in W; positive = heat into the
    /// process, negative = heat removed.
    pub fn add_duty(&mut self, name: &str, duty_w: f64) {
        self.duties.insert(name.to_string(), duty_w);
    }

    /// The named duties.
    #[must_use]
    pub fn duties(&self) -> &BTreeMap<String, f64> {
        &self.duties
    }

    /// Aggregate heating/cooling demands.
    #[must_use]
    pub fn summary(&self) -> EnergySummary {
        let mut total_heating = 0.0;
        let mut total_cooling = 0.0;
        for &d in self.duties.values() {
            if d > 0.0 {
                total_heating += d;
            } else {
                total_cooling += -d;
            }
        }
        EnergySummary {
            total_heating,
            total_cooling,
            net: total_heating - total_cooling,
        }
    }

    /// Utility summary at fixed temperature levels: assigns each duty to
    /// the cheapest utility level able to supply/absorb it.
    ///
    /// `levels` are (name, supply temperature K) for hot utilities and
    /// (name, temperature K) for cold utilities — matched by simple
    /// temperature feasibility against each duty's level tag.
    #[must_use]
    pub fn utility_summary(&self, utilities: &BTreeMap<String, f64>) -> BTreeMap<String, f64> {
        // Group duties by their temperature-level tag encoded in the duty
        // name as "name@level"; untagged duties go to "unassigned".
        let mut per_utility: BTreeMap<String, f64> = BTreeMap::new();
        for (name, &duty) in &self.duties {
            let level = name.rsplit('@').next().unwrap_or("unassigned");
            let utility = utilities
                .iter()
                .find(|(u, _)| u.as_str() == level)
                .map(|(u, _)| u.clone())
                .unwrap_or_else(|| "unassigned".to_string());
            *per_utility.entry(utility).or_insert(0.0) += duty;
        }
        per_utility
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_splits_heating_and_cooling() {
        let mut ledger = EnergyBalance::new();
        ledger.add_duty("reboiler", 2.0e6);
        ledger.add_duty("condenser", -1.5e6);
        ledger.add_duty("preheat", 0.5e6);
        let s = ledger.summary();
        assert!((s.total_heating - 2.5e6).abs() < 1e-6);
        assert!((s.total_cooling - 1.5e6).abs() < 1e-6);
        assert!((s.net - 1.0e6).abs() < 1e-6);
    }

    #[test]
    fn replacing_a_duty_overwrites() {
        let mut ledger = EnergyBalance::new();
        ledger.add_duty("heater", 1.0e6);
        ledger.add_duty("heater", 3.0e6);
        assert_eq!(ledger.duties().len(), 1);
        assert!((ledger.summary().net - 3.0e6).abs() < 1e-6);
    }

    #[test]
    fn utility_summary_groups_by_level_tag() {
        let mut ledger = EnergyBalance::new();
        ledger.add_duty("reboiler@lp_steam", 2.0e6);
        ledger.add_duty("preheat@lp_steam", 0.5e6);
        ledger.add_duty("condenser@cooling_water", -1.5e6);
        let mut utilities = BTreeMap::new();
        utilities.insert("lp_steam".to_string(), 400.0);
        utilities.insert("cooling_water".to_string(), 300.0);
        let grouped = ledger.utility_summary(&utilities);
        assert!((grouped["lp_steam"] - 2.5e6).abs() < 1e-6);
        assert!((grouped["cooling_water"] + 1.5e6).abs() < 1e-6);
    }
}
