//! Reaction kinetics: Arrhenius, power-law, Langmuir-Hinshelwood, and
//! Michaelis-Menten rate laws.
//!
//! Concentrations in mol/m³, time in s, rates in mol/(m³·s).
//!
//! # Example
//!
//! ```
//! use tpt_proc_reaction::{RateLaw, Reaction};
//! use std::collections::BTreeMap;
//!
//! // A → B, first order in A, Ea = 60 kJ/mol.
//! let mut stoich = BTreeMap::new();
//! stoich.insert(0, -1.0);
//! stoich.insert(1, 1.0);
//! let mut orders = BTreeMap::new();
//! orders.insert(0, 1.0);
//! let reaction = Reaction::new(
//!     "A → B",
//!     stoich,
//!     RateLaw::Arrhenius {
//!         pre_exponential: 1.0e6,
//!         activation_energy: 60.0e3,
//!         orders,
//!     },
//!     -80.0e3, // J/mol, exothermic
//! );
//!
//! let c = [1.0, 0.0]; // mol/m³
//! let rate = reaction.rate(&c, 400.0);
//! assert!(rate > 0.0); // positive rate law; A declines via stoichiometry
//! ```

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// Universal gas constant, J/(mol·K).
const R: f64 = 8.314462618;

/// Rate law of a reaction.
#[derive(Clone, Debug, PartialEq)]
pub enum RateLaw {
    /// Arrhenius with power-law orders: r = A·exp(−Ea/RT)·Π cᵢ^orderᵢ.
    Arrhenius {
        /// Pre-exponential factor (units depend on the orders).
        pre_exponential: f64,
        /// Activation energy, J/mol.
        activation_energy: f64,
        /// Reaction orders by component index.
        orders: BTreeMap<usize, f64>,
    },
    /// Temperature-independent power law: r = k·Π cᵢ^orderᵢ.
    PowerLaw {
        /// Rate constant at the reference condition.
        k: f64,
        /// Reaction orders by component index.
        orders: BTreeMap<usize, f64>,
    },
    /// Langmuir-Hinshelwood: r = k·Π cᵢ/(1 + Σ Kⱼcⱼ) — single-site
    /// inhibition form with an optional driving-force exponent.
    LangmuirHinshelwood {
        /// Intrinsic rate constant.
        k: f64,
        /// Adsorption constants by driving component, 1/(mol/m³).
        adsorption: BTreeMap<usize, f64>,
        /// Rate-driving components and orders (numerator).
        driving_orders: BTreeMap<usize, f64>,
    },
    /// Michaelis-Menten enzyme kinetics: r = Vmax·c_S/(Km + c_S).
    MichaelisMenten {
        /// Maximum rate, mol/(m³·s).
        v_max: f64,
        /// Michaelis constant, mol/m³.
        k_m: f64,
        /// Index of the substrate component.
        substrate: usize,
    },
}

impl RateLaw {
    /// Rate constant k(T) for laws with a temperature dependence
    /// (1 for temperature-independent forms).
    #[must_use]
    pub fn rate_constant(&self, temperature: f64) -> f64 {
        match self {
            Self::Arrhenius {
                pre_exponential,
                activation_energy,
                ..
            } => pre_exponential * (-activation_energy / (R * temperature)).exp(),
            Self::PowerLaw { k, .. } | Self::LangmuirHinshelwood { k, .. } => *k,
            Self::MichaelisMenten { .. } => 1.0,
        }
    }

    /// Net rate of the reaction (positive for products), mol/(m³·s),
    /// from the component concentrations `c` at `temperature`.
    #[must_use]
    pub fn rate(&self, c: &[f64], temperature: f64) -> f64 {
        let positive = |v: f64| v.max(0.0);
        match self {
            Self::Arrhenius {
                pre_exponential,
                activation_energy,
                orders,
            } => {
                let mut rate = pre_exponential * (-activation_energy / (R * temperature)).exp();
                for (&i, &order) in orders {
                    rate *= positive(c.get(i).copied().unwrap_or(0.0)).powf(order);
                }
                rate
            }
            Self::PowerLaw { k, orders } => {
                let mut rate = *k;
                for (&i, &order) in orders {
                    rate *= positive(c.get(i).copied().unwrap_or(0.0)).powf(order);
                }
                rate
            }
            Self::LangmuirHinshelwood {
                k,
                adsorption,
                driving_orders,
            } => {
                let mut numerator = *k;
                let mut denominator = 1.0;
                for (&i, &k_ads) in adsorption {
                    denominator += k_ads * positive(c.get(i).copied().unwrap_or(0.0));
                }
                for (&i, &order) in driving_orders {
                    numerator *= positive(c.get(i).copied().unwrap_or(0.0)).powf(order);
                }
                numerator / denominator
            }
            Self::MichaelisMenten {
                v_max,
                k_m,
                substrate,
            } => {
                let s = positive(c.get(*substrate).copied().unwrap_or(0.0));
                v_max * s / (k_m + s)
            }
        }
    }
}

/// A chemical reaction: stoichiometry, rate law, and heat of reaction.
#[derive(Clone, Debug, PartialEq)]
pub struct Reaction {
    /// Identifier (diagnostics).
    pub name: String,
    /// Stoichiometric coefficients by component index (negative reactants,
    /// positive products).
    pub stoichiometry: BTreeMap<usize, f64>,
    /// Rate law.
    pub law: RateLaw,
    /// Heat of reaction ΔH_R, J/mol (negative = exothermic).
    pub heat_of_reaction: f64,
}

impl Reaction {
    /// Creates a reaction.
    #[must_use]
    pub fn new(
        name: &str,
        stoichiometry: BTreeMap<usize, f64>,
        law: RateLaw,
        heat_of_reaction: f64,
    ) -> Self {
        Self {
            name: name.to_string(),
            stoichiometry,
            law,
            heat_of_reaction,
        }
    }

    /// Net rate of the reaction, mol/(m³·s).
    #[must_use]
    pub fn rate(&self, c: &[f64], temperature: f64) -> f64 {
        self.law.rate(c, temperature)
    }

    /// Rate of change of every component concentration: dcᵢ/dt = νᵢ·r.
    /// `out` must have at least as many entries as the largest component
    /// index in the stoichiometry.
    pub fn dcdt(&self, c: &[f64], temperature: f64, out: &mut [f64]) {
        let r = self.rate(c, temperature);
        for (&i, &nu) in &self.stoichiometry {
            if let Some(slot) = out.get_mut(i) {
                *slot += nu * r;
            }
        }
    }

    /// Equilibrium constant by van 't Hoff from ΔH_R and a reference
    /// value: K(T) = K_ref·exp(−ΔH_R/R·(1/T − 1/T_ref)).
    #[must_use]
    pub fn equilibrium_constant(&self, temperature: f64, k_ref: f64, t_ref: f64) -> f64 {
        k_ref * (-(self.heat_of_reaction / R) * (1.0 / temperature - 1.0 / t_ref)).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_order() -> Reaction {
        let mut stoich = BTreeMap::new();
        stoich.insert(0, -1.0);
        stoich.insert(1, 1.0);
        let mut orders = BTreeMap::new();
        orders.insert(0, 1.0);
        Reaction::new(
            "A → B",
            stoich,
            RateLaw::Arrhenius {
                pre_exponential: 1.0e6,
                activation_energy: 60.0e3,
                orders,
            },
            -80.0e3,
        )
    }

    #[test]
    fn arrhenius_doubles_every_ten_degrees_rule_of_thumb() {
        // A reaction with Ea ≈ 54.5 kJ/mol doubles between 300 and 310 K.
        let mut orders = BTreeMap::new();
        orders.insert(0, 1.0);
        let law = RateLaw::Arrhenius {
            pre_exponential: 1.0e7,
            activation_energy: 54_500.0,
            orders,
        };
        let k1 = law.rate_constant(300.0);
        let k2 = law.rate_constant(310.0);
        assert!((k2 / k1 - 2.0).abs() < 0.05, "ratio = {}", k2 / k1);
    }

    #[test]
    fn first_order_rate_is_linear_in_concentration() {
        let rxn = first_order();
        let r1 = rxn.rate(&[1.0, 0.0], 350.0);
        let r2 = rxn.rate(&[2.0, 0.0], 350.0);
        assert!(r1 > 0.0);
        assert!((r2 / r1 - 2.0).abs() < 1e-12);
    }

    #[test]
    fn dcdt_satisfies_stoichiometry() {
        let rxn = first_order();
        let mut out = [0.0_f64; 2];
        rxn.dcdt(&[2.0, 1.0], 350.0, &mut out);
        let r = rxn.rate(&[2.0, 1.0], 350.0);
        assert!((out[0] - (-r)).abs() < 1e-12);
        assert!((out[1] - r).abs() < 1e-12);
    }

    #[test]
    fn michaelis_menten_limits() {
        let law = RateLaw::MichaelisMenten {
            v_max: 5.0,
            k_m: 1.0,
            substrate: 0,
        };
        // c_S = Km → half-maximal rate.
        let half = law.rate(&[1.0], 300.0);
        assert!((half - 2.5).abs() < 1e-12);
        // Saturating substrate approaches Vmax.
        let saturated = law.rate(&[1.0e6], 300.0);
        assert!((saturated - 5.0).abs() < 1e-3);
    }

    #[test]
    fn langmuir_hinshelwood_inhibited_by_product() {
        let mut driving = BTreeMap::new();
        driving.insert(0, 1.0);
        let mut adsorption = BTreeMap::new();
        adsorption.insert(0, 1.0);
        adsorption.insert(1, 10.0); // product strongly adsorbs
        let law = RateLaw::LangmuirHinshelwood {
            k: 1.0,
            adsorption,
            driving_orders: driving,
        };
        let clean = law.rate(&[1.0, 0.0], 400.0);
        let inhibited = law.rate(&[1.0, 1.0], 400.0);
        assert!(inhibited < clean, "product must inhibit");
    }

    #[test]
    fn van_t_hoff_exotherm_reduces_k_with_temperature() {
        let rxn = first_order(); // ΔH_R = −80 kJ/mol
        let k_low = rxn.equilibrium_constant(300.0, 1.0e3, 298.15);
        let k_high = rxn.equilibrium_constant(400.0, 1.0e3, 298.15);
        assert!(k_high < k_low, "exothermic K falls with T");
    }
}
