//! Bridge from `tpt-process` streams and reactor designs into
//! `tpt-materials` compositions (`tpt-mat-core`).
//!
//! Process streams carry *species* (water, metal sulfates, hydroxides);
//! material science works on *elemental* compositions. This crate
//! provides the species→element mass-balance translation and the
//! battery-precursor process design from the master plan (§6 of the
//! spec).
//!
//! # Example
//!
//! ```
//! use tpt_mat_core::{Composition, CompositionBasis};
//! use tpt_proc_materials::{elemental_composition, formula_weight_fractions, PrecursorDesign};
//!
//! // NMC hydroxide precursor M(OH)2 with the 811 ratio.
//! let design = PrecursorDesign::new(8.0, 1.0, 1.0).unwrap();
//! let composition: Composition = design.target_composition().unwrap();
//! assert_eq!(composition.basis(), CompositionBasis::Weight);
//!
//! // A mixture of species maps to elements by exact mass balance.
//! let nickel_sulfate = formula_weight_fractions(&[("Ni", 1), ("S", 4)]).unwrap();
//! let oxygen = formula_weight_fractions(&[("O", 2)]).unwrap();
//! let mix = vec![(nickel_sulfate, 0.5), (oxygen, 0.5)];
//! let elements = elemental_composition(&mix).unwrap();
//! assert!((elements["Ni"] + elements["S"] + elements["O"] - 1.0).abs() < 1e-9);
//! ```

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use tpt_mat_core::{Composition, CompositionBasis};

/// A chemical formula: (element symbol, stoichiometric count) pairs.
pub type Formula<'a> = &'a [(&'a str, u32)];

/// Molar masses, kg/kmol (g/mol) for the elements used by process
/// species. Values are IUPAC standard atomic weights, rounded to 4
/// decimals.
#[must_use]
pub fn molar_mass(element: &str) -> Option<f64> {
    Some(match element {
        "H" => 1.008,
        "C" => 12.011,
        "N" => 14.007,
        "O" => 15.999,
        "Na" => 22.990,
        "Mg" => 24.305,
        "Al" => 26.982,
        "S" => 32.06,
        "Cl" => 35.45,
        "K" => 39.098,
        "Ca" => 40.078,
        "Ti" => 47.867,
        "Cr" => 51.996,
        "Mn" => 54.938,
        "Fe" => 55.845,
        "Co" => 58.933,
        "Ni" => 58.693,
        "Cu" => 63.546,
        "Zn" => 65.38,
        "Li" => 6.94,
        _ => return None,
    })
}

/// Builds a formula and its molar mass, kg/kmol.
///
/// # Errors
/// Returns `Err` with the unknown element symbol when any element is
/// outside the built-in table.
pub fn formula(pairs: Formula) -> Result<(BTreeMap<String, f64>, f64), String> {
    let mut map = BTreeMap::new();
    let mut mass = 0.0;
    for &(element, count) in pairs {
        let m = molar_mass(element).ok_or_else(|| format!("unknown element {element:?}"))?;
        *map.entry(element.to_string()).or_insert(0.0) += f64::from(count);
        mass += m * f64::from(count);
    }
    if mass <= 0.0 {
        return Err("empty formula".into());
    }
    Ok((map, mass))
}

/// Element weight fractions of a single formula.
///
/// # Errors
/// Propagates [`formula`] errors.
pub fn formula_weight_fractions(pairs: Formula) -> Result<BTreeMap<String, f64>, String> {
    let (map, mass) = formula(pairs)?;
    let mut fractions = BTreeMap::new();
    for (element, count) in &map {
        fractions.insert(
            element.clone(),
            count * molar_mass(element).expect("validated above") / mass,
        );
    }
    Ok(fractions)
}

/// Element weight fractions of a *mixture* of species: exact mass balance
/// `elements = Σ species_fraction × species_element_fractions`.
///
/// The species fractions must sum to ~1 (tolerance 1e-6).
///
/// # Errors
/// Returns `Err` when the mixture fractions do not sum to 1.
pub fn elemental_composition(
    mixture: &[(BTreeMap<String, f64>, f64)],
) -> Result<BTreeMap<String, f64>, String> {
    let total: f64 = mixture.iter().map(|(_, frac)| frac).sum();
    if (total - 1.0).abs() > 1e-6 {
        return Err(format!("species fractions must sum to 1 (got {total})"));
    }
    let mut elements: BTreeMap<String, f64> = BTreeMap::new();
    for (species, fraction) in mixture {
        for (element, wf) in species {
            *elements.entry(element.clone()).or_insert(0.0) += fraction * wf;
        }
    }
    Ok(elements)
}

/// Converts element weight fractions into a `tpt-mat-core` `Composition`
/// on the weight basis.
///
/// # Errors
/// Returns `Err` when the map is empty; the sum is normalized inside
/// `tpt-mat-core`.
pub fn to_material_composition(
    elements: &BTreeMap<String, f64>,
) -> Result<Composition, tpt_mat_core::CompositionError> {
    Composition::new(CompositionBasis::Weight, elements.clone(), 1e-6)
}

/// Mixed-metal hydroxide precursor M(OH)₂ design (the NMC co-precipitation
/// step feeding battery cathode production).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrecursorDesign {
    /// Ni share of the metal, mol fraction (normalized internally).
    pub nickel: f64,
    /// Mn share, mol fraction.
    pub manganese: f64,
    /// Co share, mol fraction.
    pub cobalt: f64,
}

impl PrecursorDesign {
    /// Creates a precursor design from the metal molar ratios (any scale;
    /// normalized to sum 1).
    ///
    /// # Errors
    /// Returns `Err` when the ratios are non-finite or sum to zero.
    pub fn new(nickel: f64, manganese: f64, cobalt: f64) -> Result<Self, String> {
        let sum = nickel + manganese + cobalt;
        if !sum.is_finite() || sum <= 0.0 {
            return Err("metal ratios must be finite and sum > 0".into());
        }
        Ok(Self {
            nickel: nickel / sum,
            manganese: manganese / sum,
            cobalt: cobalt / sum,
        })
    }

    /// Molar mass of M(OH)₂ for the metal ratio, kg/kmol.
    #[must_use]
    pub fn molar_mass(&self) -> f64 {
        let metal = self.nickel * molar_mass("Ni").expect("Ni")
            + self.manganese * molar_mass("Mn").expect("Mn")
            + self.cobalt * molar_mass("Co").expect("Co");
        metal + 2.0 * molar_mass("O").expect("O") + 2.0 * molar_mass("H").expect("H")
    }

    /// Element weight fractions of M(OH)₂.
    ///
    /// # Errors
    /// Never in practice (all metals are in the table); typed for
    /// symmetry with [`elemental_composition`].
    pub fn target_composition(&self) -> Result<Composition, tpt_mat_core::CompositionError> {
        let mass = self.molar_mass();
        let mut elements = BTreeMap::new();
        elements.insert(
            "Ni".to_string(),
            self.nickel * molar_mass("Ni").expect("Ni") / mass,
        );
        elements.insert(
            "Mn".to_string(),
            self.manganese * molar_mass("Mn").expect("Mn") / mass,
        );
        elements.insert(
            "Co".to_string(),
            self.cobalt * molar_mass("Co").expect("Co") / mass,
        );
        elements.insert("O".to_string(), 2.0 * molar_mass("O").expect("O") / mass);
        elements.insert("H".to_string(), 2.0 * molar_mass("H").expect("H") / mass);
        to_material_composition(&elements)
    }

    /// Mixed-sulfate feed composition (NiSO₄/MnSO₄/CoSO₄ in water) matching
    /// the metal ratio, as element weight fractions. `water_per_salt_mass`
    /// is the kg of water per kg of salt in the dilute feed (typical 1–3).
    ///
    /// # Errors
    /// Propagates formula errors; returns `Err` for a negative water
    /// ratio.
    pub fn feed_composition(
        &self,
        water_per_salt_mass: f64,
    ) -> Result<BTreeMap<String, f64>, String> {
        if water_per_salt_mass < 0.0 {
            return Err("water ratio must be non-negative".into());
        }
        let niso4 = formula_weight_fractions(&[("Ni", 1), ("S", 1), ("O", 4)])?;
        let mnso4 = formula_weight_fractions(&[("Mn", 1), ("S", 1), ("O", 4)])?;
        let coso4 = formula_weight_fractions(&[("Co", 1), ("S", 1), ("O", 4)])?;
        let water = formula_weight_fractions(&[("H", 2), ("O", 1)])?;

        // Per 1 mol of total metal, the salt masses are the mol fractions
        // times the salt molar masses.
        let salt_mass = self.nickel * formula(&[("Ni", 1), ("S", 1), ("O", 4)])?.1
            + self.manganese * formula(&[("Mn", 1), ("S", 1), ("O", 4)])?.1
            + self.cobalt * formula(&[("Co", 1), ("S", 1), ("O", 4)])?.1;
        let water_mass = salt_mass * water_per_salt_mass;
        let total = salt_mass + water_mass;

        let mut elements: BTreeMap<String, f64> = BTreeMap::new();
        for (metal_share, salt) in [
            (self.nickel, &niso4),
            (self.manganese, &mnso4),
            (self.cobalt, &coso4),
        ] {
            for (element, wf) in salt {
                *elements.entry(element.clone()).or_insert(0.0) +=
                    metal_share * salt_mass * wf / total;
            }
        }
        for (element, wf) in &water {
            *elements.entry(element.clone()).or_insert(0.0) += water_mass * wf / total;
        }
        Ok(elements)
    }

    /// Suggested co-precipitation CSTR volume, m³, for a target
    /// production rate: classic mixed-sulfate precipitation at
    /// `residence_time_h` (typically 10–12 h) and a slurry solids density
    /// `solids_kg_m3` (typically 50–100 kg/m³).
    #[must_use]
    pub fn cstr_volume_m3(
        &self,
        production_kg_h: f64,
        residence_time_h: f64,
        solids_kg_m3: f64,
    ) -> f64 {
        if solids_kg_m3 <= 0.0 {
            return f64::NAN;
        }
        production_kg_h.max(0.0) * residence_time_h.max(0.0) / solids_kg_m3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formula_mass_and_fractions() {
        let (map, mass) = formula(&[("H", 2), ("O", 1)]).unwrap();
        assert!((mass - 18.015).abs() < 1e-9);
        assert!((map["H"] - 2.0).abs() < 1e-12);
        let wf = formula_weight_fractions(&[("H", 2), ("O", 1)]).unwrap();
        assert!((wf["H"] - 2.016 / 18.015).abs() < 1e-9);
        assert!(formula(&[("Xx", 1)]).is_err());
    }

    #[test]
    fn elemental_balance_of_a_binary_mixture() {
        let water = formula_weight_fractions(&[("H", 2), ("O", 1)]).unwrap();
        let co2 = formula_weight_fractions(&[("C", 1), ("O", 2)]).unwrap();
        let elements = elemental_composition(&[(water, 0.5), (co2, 0.5)]).unwrap();
        let sum: f64 = elements.values().sum();
        assert!((sum - 1.0).abs() < 1e-9);
        assert!((elements["C"] - 0.5 * 12.011 / 44.009).abs() < 1e-9);
        assert!(elements.contains_key("H"));
    }

    #[test]
    fn rejects_unnormalized_mixture() {
        let water = formula_weight_fractions(&[("H", 2), ("O", 1)]).unwrap();
        assert!(elemental_composition(&[(water, 0.5)]).is_err());
    }

    #[test]
    fn precursor_design_normalizes_and_composes() {
        let design = PrecursorDesign::new(8.0, 1.0, 1.0).unwrap();
        assert!((design.nickel - 0.8).abs() < 1e-12);
        let composition = design.target_composition().unwrap();
        let sum: f64 = composition.iter().map(|(_, f)| f).sum();
        assert!((sum - 1.0).abs() < 1e-6);
        // Metals are ~65% of the hydroxide mass; O+H is the rest.
        let metal = |e: &str| composition.fraction(e).unwrap_or(0.0);
        let metals = metal("Ni") + metal("Mn") + metal("Co");
        assert!(metals > 0.60 && metals < 0.70, "metals = {metals}");
        assert!(PrecursorDesign::new(0.0, 0.0, 0.0).is_err());
    }

    #[test]
    fn feed_composition_closes_to_unity() {
        let design = PrecursorDesign::new(8.0, 1.0, 1.0).unwrap();
        let feed = design.feed_composition(2.0).unwrap();
        let sum: f64 = feed.values().sum();
        assert!((sum - 1.0).abs() < 1e-9, "sum = {sum}");
        // Sulfur present, lithium absent.
        assert!(feed["S"] > 0.0);
        assert!(!feed.contains_key("Li"));
    }

    #[test]
    fn cstr_volume_scales_with_throughput() {
        let design = PrecursorDesign::new(6.0, 2.0, 2.0).unwrap();
        let v = design.cstr_volume_m3(1000.0, 12.0, 80.0);
        assert!((v - 150.0).abs() < 1e-9);
        assert!(design.cstr_volume_m3(1000.0, 12.0, 0.0).is_nan());
    }
}
