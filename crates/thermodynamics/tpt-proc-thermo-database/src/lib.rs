//! The built-in open chemical database.
//!
//! The point of `tpt-process` is that the property data should be a public
//! good, not a license line-item. This crate ships constants for the
//! chemicals that cover the bulk of teaching and pre-design work:
//! critical properties, acentric factors, boiling points, and ideal-gas
//! heat capacities, plus a table of Peng-Robinson binary interaction
//! parameters for common pairs.
//!
//! # Data provenance and accuracy
//!
//! Constants are compiled from open literature tabulations (DIPPR-style
//! correlations, NIST WebBook summary values, and Smith–Van Ness–Abbott
//! Appendix C heat capacities). They are **design-estimate grade**:
//! critical constants are typically within experimental uncertainty, Cp
//! fits within a few percent over 250–1000 K. ForLicenced final design,
//! validate against experiment or replace entries via
//! [`ChemicalDatabase::register`]. Every entry is overridable; nothing is
//! hard-coded in the physics crates.
//!
//! # Example
//!
//! ```
//! use tpt_proc_thermo_database::ChemicalDatabase;
//! use tpt_proc_thermo_eos::CubicEos;
//!
//! let db = ChemicalDatabase::builtin();
//! let water = db.get_component("water").unwrap();
//! assert_eq!(water.cas_number, "7732-18-5");
//!
//! // Build a PR EOS directly from the database.
//! let components = db.components_for(&["water", "methanol"]).unwrap();
//! let eos = CubicEos::peng_robinson(components)
//!     .with_binary_interaction(0, 1, db.binary_interaction("water", "methanol"));
//! assert_eq!(eos.components().len(), 2);
//! ```

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use tpt_proc_core::{CoreError, Result};
use tpt_proc_thermo_core::{Component, CpCorrelation};

/// Reference collection of built-in components keyed by canonical name.
#[derive(Clone, Debug, Default)]
pub struct ChemicalDatabase {
    components: BTreeMap<String, Component>,
    /// Binary interaction parameters keyed by "name-a|name-b" (sorted).
    binary: BTreeMap<(String, String), f64>,
}

impl ChemicalDatabase {
    /// The built-in database (compiled in; no I/O).
    #[must_use]
    pub fn builtin() -> Self {
        let mut db = Self::default();
        for component in builtin_components() {
            db.components.insert(component.name.clone(), component);
        }
        for ((a, b), k) in builtin_binary_interactions() {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            db.binary.insert((lo.to_string(), hi.to_string()), k);
        }
        db
    }

    /// An empty database for user-supplied data.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Registers (or replaces) a component under its own name.
    pub fn register(&mut self, component: Component) {
        let name = component.name.to_lowercase();
        self.components.insert(name, component);
    }

    /// Registers a binary interaction parameter (symmetric).
    pub fn register_binary_interaction(&mut self, name_a: &str, name_b: &str, k_ij: f64) {
        let (lo, hi) = ordered_pair(name_a, name_b);
        self.binary.insert((lo, hi), k_ij);
    }

    /// Looks a component up by name (case-insensitive) or CAS number.
    #[must_use]
    pub fn get_component(&self, name_or_cas: &str) -> Option<&Component> {
        let key = name_or_cas.to_lowercase();
        self.components.get(&key).or_else(|| {
            self.components
                .values()
                .find(|c| c.cas_number == name_or_cas)
        })
    }

    /// All registered component names, sorted.
    #[must_use]
    pub fn component_names(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }

    /// Number of registered components.
    #[must_use]
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// True if the database has no components.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// The binary interaction parameter kᵢⱼ for a pair (0.0 when unknown).
    #[must_use]
    pub fn binary_interaction(&self, name_a: &str, name_b: &str) -> f64 {
        let (lo, hi) = ordered_pair(name_a, name_b);
        self.binary.get(&(lo, hi)).copied().unwrap_or(0.0)
    }

    /// Builds the component list for a named set, in order.
    ///
    /// # Errors
    /// [`CoreError::MissingReference`] naming the first unknown component.
    pub fn components_for(&self, names: &[&str]) -> Result<Vec<Component>> {
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            let component = self.get_component(name).ok_or_else(|| {
                CoreError::MissingReference(format!("component {name:?} is not in the database"))
            })?;
            out.push(component.clone());
        }
        Ok(out)
    }
}

fn ordered_pair(a: &str, b: &str) -> (String, String) {
    let (a, b) = (a.to_lowercase(), b.to_lowercase());
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// The built-in component table.
///
/// Cp coefficients follow the DIPPR-127 form
/// `Cp/R = A + B·T + C·T² + D·T³ + E/T²` (T in K), fitted to
/// Smith–Van Ness–Abbott Appendix C / NIST tabulations; values are
/// approximate (see crate docs).
#[allow(clippy::too_many_lines)]
fn builtin_components() -> Vec<Component> {
    fn cp(a: f64, b: f64, c: f64, d: f64, e: f64) -> CpCorrelation {
        CpCorrelation::dippr127(a, b, c, d, e)
    }
    vec![
        Component::new(
            "water",
            "7732-18-5",
            18.015e-3,
            647.096,
            22.064e6,
            55.9e-6,
            0.344,
            373.124,
            cp(3.470, 1.450e-3, 0.0, 0.0, 0.121e5),
        ),
        Component::new(
            "methanol",
            "67-56-1",
            32.042e-3,
            512.64,
            8.096e6,
            118.0e-6,
            0.566,
            337.85,
            cp(2.211, 12.216e-3, -3.450e-6, 0.0, 0.0),
        ),
        Component::new(
            "ethanol",
            "64-17-5",
            46.069e-3,
            513.92,
            6.148e6,
            168.0e-6,
            0.644,
            351.45,
            cp(3.518, 20.001e-3, -6.002e-6, 0.0, 0.0),
        ),
        Component::new(
            "acetone",
            "67-64-1",
            58.080e-3,
            508.1,
            4.700e6,
            209.0e-6,
            0.307,
            329.23,
            cp(2.800, 24.000e-3, -7.500e-6, 0.0, 0.0),
        ),
        Component::new(
            "benzene",
            "71-43-2",
            78.114e-3,
            562.05,
            4.895e6,
            259.0e-6,
            0.210,
            353.25,
            cp(-0.206, 39.064e-3, -13.301e-6, 0.0, 0.0),
        ),
        Component::new(
            "toluene",
            "108-88-3",
            92.141e-3,
            591.75,
            4.106e6,
            316.0e-6,
            0.264,
            383.75,
            cp(0.290, 47.035e-3, -16.045e-6, 0.0, 0.0),
        ),
        Component::new(
            "styrene",
            "100-42-5",
            104.152e-3,
            636.0,
            3.840e6,
            354.0e-6,
            0.297,
            418.30,
            cp(1.000, 48.000e-3, -15.000e-6, 0.0, 0.0),
        ),
        Component::new(
            "phenol",
            "108-95-2",
            94.113e-3,
            694.25,
            6.130e6,
            229.0e-6,
            0.443,
            454.99,
            cp(2.000, 40.000e-3, -13.000e-6, 0.0, 0.0),
        ),
        Component::new(
            "methane",
            "74-82-8",
            16.043e-3,
            190.56,
            4.599e6,
            98.6e-6,
            0.0115,
            111.67,
            cp(1.702, 9.081e-3, -2.164e-6, 0.0, 0.0),
        ),
        Component::new(
            "ethane",
            "74-84-0",
            30.070e-3,
            305.32,
            4.872e6,
            145.5e-6,
            0.0995,
            184.55,
            cp(1.131, 19.225e-3, -5.561e-6, 0.0, 0.0),
        ),
        Component::new(
            "propane",
            "74-98-6",
            44.097e-3,
            369.83,
            4.248e6,
            203.0e-6,
            0.1523,
            231.04,
            cp(1.213, 28.785e-3, -8.824e-6, 0.0, 0.0),
        ),
        Component::new(
            "n-butane",
            "106-97-8",
            58.123e-3,
            425.12,
            3.797e6,
            255.0e-6,
            0.1995,
            272.65,
            cp(1.935, 36.915e-3, -11.402e-6, 0.0, 0.0),
        ),
        Component::new(
            "n-pentane",
            "109-66-0",
            72.150e-3,
            469.7,
            3.370e6,
            311.0e-6,
            0.2510,
            309.21,
            cp(2.464, 45.351e-3, -14.111e-6, 0.0, 0.0),
        ),
        Component::new(
            "n-hexane",
            "110-54-3",
            86.178e-3,
            507.6,
            3.025e6,
            368.0e-6,
            0.3013,
            341.87,
            cp(3.025, 53.229e-3, -16.652e-6, 0.0, 0.0),
        ),
        Component::new(
            "n-heptane",
            "142-82-5",
            100.204e-3,
            540.2,
            2.740e6,
            426.0e-6,
            0.3495,
            371.53,
            cp(3.570, 60.817e-3, -19.051e-6, 0.0, 0.0),
        ),
        Component::new(
            "n-octane",
            "111-65-9",
            114.232e-3,
            568.7,
            2.490e6,
            486.0e-6,
            0.3996,
            398.77,
            cp(4.108, 68.249e-3, -21.397e-6, 0.0, 0.0),
        ),
        Component::new(
            "ethylene",
            "74-85-1",
            28.054e-3,
            282.34,
            5.041e6,
            131.0e-6,
            0.0866,
            169.38,
            cp(1.424, 14.394e-3, -4.392e-6, 0.0, 0.0),
        ),
        Component::new(
            "propylene",
            "115-07-1",
            42.081e-3,
            364.9,
            4.600e6,
            181.0e-6,
            0.1477,
            225.46,
            cp(1.637, 22.670e-3, -6.895e-6, 0.0, 0.0),
        ),
        Component::new(
            "hydrogen",
            "1333-74-0",
            2.016e-3,
            33.19,
            1.313e6,
            64.1e-6,
            -0.216,
            20.28,
            cp(3.249, 0.422e-3, 0.0, 0.0, 0.083e5),
        ),
        Component::new(
            "nitrogen",
            "7727-37-9",
            28.013e-3,
            126.20,
            3.394e6,
            89.8e-6,
            0.0372,
            77.36,
            cp(3.280, 0.593e-3, 0.0, 0.0, 0.040e5),
        ),
        Component::new(
            "oxygen",
            "7782-44-7",
            31.999e-3,
            154.58,
            5.043e6,
            73.4e-6,
            0.0222,
            90.19,
            cp(3.639, 0.506e-3, 0.0, 0.0, -0.227e5),
        ),
        Component::new(
            "carbon-monoxide",
            "630-08-0",
            28.010e-3,
            132.85,
            3.494e6,
            93.4e-6,
            0.045,
            81.64,
            cp(3.376, 0.557e-3, 0.0, 0.0, -0.031e5),
        ),
        Component::new(
            "carbon-dioxide",
            "124-38-9",
            44.010e-3,
            304.13,
            7.377e6,
            94.0e-6,
            0.2239,
            194.65,
            cp(5.457, 1.045e-3, 0.0, 0.0, -1.157e5),
        ),
        Component::new(
            "hydrogen-sulfide",
            "7783-06-4",
            34.081e-3,
            373.53,
            8.963e6,
            98.5e-6,
            0.0942,
            213.60,
            cp(3.931, 1.490e-3, 0.0, 0.0, 0.232e5),
        ),
        Component::new(
            "ammonia",
            "7664-41-7",
            17.031e-3,
            405.65,
            11.35e6,
            72.5e-6,
            0.2526,
            239.82,
            cp(3.578, 3.020e-3, 0.0, 0.0, 0.186e5),
        ),
        Component::new(
            "chloroform",
            "67-66-3",
            119.378e-3,
            536.4,
            5.470e6,
            239.0e-6,
            0.222,
            334.33,
            cp(2.000, 21.000e-3, -7.000e-6, 0.0, 0.0),
        ),
        Component::new(
            "diethyl-ether",
            "60-29-7",
            74.123e-3,
            466.7,
            3.640e6,
            280.0e-6,
            0.281,
            307.60,
            cp(3.000, 39.000e-3, -12.500e-6, 0.0, 0.0),
        ),
    ]
}

/// Built-in Peng-Robinson binary interaction parameters.
///
/// Typical hydrocarbon/non-hydrocarbon literature values (Poling et al.,
/// *The Properties of Gases and Liquids*); unlisted pairs default to 0.
fn builtin_binary_interactions() -> Vec<((&'static str, &'static str), f64)> {
    vec![
        (("carbon-dioxide", "methane"), 0.095),
        (("carbon-dioxide", "ethane"), 0.137),
        (("carbon-dioxide", "propane"), 0.126),
        (("carbon-dioxide", "n-butane"), 0.133),
        (("carbon-dioxide", "n-pentane"), 0.125),
        (("carbon-dioxide", "n-hexane"), 0.123),
        (("carbon-dioxide", "n-heptane"), 0.120),
        (("carbon-dioxide", "n-octane"), 0.114),
        (("carbon-dioxide", "ethylene"), 0.055),
        (("carbon-dioxide", "water"), 0.096),
        (("nitrogen", "methane"), 0.031),
        (("nitrogen", "ethane"), 0.052),
        (("nitrogen", "propane"), 0.080),
        (("nitrogen", "n-butane"), 0.090),
        (("hydrogen-sulfide", "methane"), 0.070),
        (("hydrogen-sulfide", "propane"), 0.088),
        (("hydrogen-sulfide", "water"), 0.186),
        (("water", "methane"), 0.500),
        (("water", "ethane"), 0.490),
        (("water", "propane"), 0.480),
        (("water", "n-butane"), 0.470),
        (("water", "benzene"), 0.210),
        (("water", "toluene"), 0.200),
        (("water", "methanol"), 0.0),
        (("water", "ethanol"), -0.085),
        (("water", "ammonia"), 0.100),
        (("methanol", "benzene"), 0.028),
        (("methanol", "toluene"), 0.018),
        (("methanol", "methane"), 0.025),
        (("ethanol", "benzene"), 0.010),
        (("ethanol", "toluene"), 0.005),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_loads_and_lookups() {
        let db = ChemicalDatabase::builtin();
        assert!(db.len() >= 25);
        let water = db.get_component("WATER").expect("case-insensitive");
        assert_eq!(water.cas_number, "7732-18-5");
        // CAS lookup also works.
        assert!(db.get_component("7732-18-5").is_some());
        assert!(db.get_component("unobtanium").is_none());
    }

    #[test]
    fn components_for_preserves_order_and_reports_unknown() {
        let db = ChemicalDatabase::builtin();
        let comps = db.components_for(&["benzene", "toluene"]).unwrap();
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].name, "benzene");
        assert!(db.components_for(&["benzene", "kryptonite"]).is_err());
    }

    #[test]
    fn binary_interactions_are_symmetric_and_default_zero() {
        let db = ChemicalDatabase::builtin();
        let k1 = db.binary_interaction("water", "methane");
        let k2 = db.binary_interaction("methane", "water");
        assert_eq!(k1, k2);
        assert!((k1 - 0.500).abs() < 1e-12);
        // Listed pairs may be stored under either ordering...
        assert!((db.binary_interaction("methanol", "benzene") - 0.028).abs() < 1e-12);
        // ...while unlisted pairs default to zero.
        assert_eq!(db.binary_interaction("propane", "benzene"), 0.0);
    }

    #[test]
    fn user_can_override_entries() {
        let mut db = ChemicalDatabase::empty();
        db.register(Component::new(
            "my-fluid",
            "00000-00-0",
            100.0e-3,
            600.0,
            3.0e6,
            300.0e-6,
            0.3,
            350.0,
            CpCorrelation::constant(150.0),
        ));
        assert_eq!(
            db.get_component("my-fluid").unwrap().molecular_weight,
            100.0e-3
        );
        db.register_binary_interaction("my-fluid", "water", 0.11);
        assert!((db.binary_interaction("water", "my-fluid") - 0.11).abs() < 1e-12);
    }

    #[test]
    fn all_builtin_components_validate() {
        let db = ChemicalDatabase::builtin();
        for name in db.component_names() {
            let component = db.get_component(&name).expect("listed");
            component
                .validate()
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
}
