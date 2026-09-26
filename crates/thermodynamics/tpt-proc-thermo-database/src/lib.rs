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
//! fits within a few percent over 250–1000 K. For licensed final design,
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
        // --- Additions (2026-09 review): see PROVENANCE.md for sources. ---
        // Constant-Cp entries are frozen at 298.15 K ideal-gas values and
        // flagged for a DIPPR-127 refit; scalar properties follow the
        // standard DIPPR/NIST tabulations cited in PROVENANCE.md.
        Component::new(
            "helium",
            "7440-59-7",
            4.0026e-3,
            5.1953,
            2.276e5,
            57.4e-6,
            -0.390,
            4.222,
            CpCorrelation::constant(20.786),
        ),
        Component::new(
            "argon",
            "7440-37-1",
            39.948e-3,
            150.687,
            4.863e6,
            74.6e-6,
            0.0,
            87.302,
            CpCorrelation::constant(20.786),
        ),
        Component::new(
            "sulfur-dioxide",
            "7446-09-5",
            64.066e-3,
            430.75,
            7.884e6,
            122.0e-6,
            0.2454,
            263.13,
            CpCorrelation::constant(39.87),
        ),
        Component::new(
            "acetylene",
            "74-86-2",
            26.038e-3,
            308.3,
            6.138e6,
            112.0e-6,
            0.187,
            189.36,
            CpCorrelation::constant(43.99),
        ),
        Component::new(
            "1,3-butadiene",
            "106-99-0",
            54.091e-3,
            425.0,
            4.32e6,
            220.0e-6,
            0.195,
            268.74,
            CpCorrelation::constant(79.54),
        ),
        Component::new(
            "isobutane",
            "75-28-5",
            58.123e-3,
            407.81,
            3.648e6,
            262.7e-6,
            0.182,
            261.43,
            CpCorrelation::constant(96.65),
        ),
        Component::new(
            "cyclohexane",
            "110-82-7",
            84.161e-3,
            553.78,
            4.081e6,
            308.0e-6,
            0.212,
            353.87,
            CpCorrelation::constant(106.27),
        ),
        Component::new(
            "n-nonane",
            "111-84-2",
            128.259e-3,
            594.6,
            2.289e6,
            548.0e-6,
            0.277,
            423.94,
            CpCorrelation::constant(211.7),
        ),
        Component::new(
            "n-decane",
            "124-18-5",
            142.286e-3,
            617.7,
            2.11e6,
            624.0e-6,
            0.4923,
            447.28,
            CpCorrelation::constant(233.7),
        ),
        Component::new(
            "acetic-acid",
            "64-19-7",
            60.052e-3,
            591.95,
            5.786e6,
            171.0e-6,
            0.4665,
            391.05,
            CpCorrelation::constant(66.25),
        ),
        Component::new(
            "ethylene-glycol",
            "107-21-1",
            62.068e-3,
            645.0,
            7.53e6,
            186.0e-6,
            0.527,
            470.45,
            CpCorrelation::constant(87.9),
        ),
        Component::new(
            "ethylene-oxide",
            "75-21-8",
            44.053e-3,
            469.15,
            7.194e6,
            139.6e-6,
            0.201,
            283.85,
            CpCorrelation::constant(48.2),
        ),
        Component::new(
            "dimethyl-ether",
            "115-10-6",
            46.069e-3,
            400.1,
            5.37e6,
            172.0e-6,
            0.302,
            248.35,
            CpCorrelation::constant(65.6),
        ),
        Component::new(
            "methyl-tert-butyl-ether",
            "1634-04-4",
            88.150e-3,
            497.1,
            3.43e6,
            329.0e-6,
            0.266,
            328.35,
            CpCorrelation::constant(120.4),
        ),
        Component::new(
            "acetaldehyde",
            "75-07-0",
            44.053e-3,
            466.0,
            5.55e6,
            154.0e-6,
            0.2907,
            293.25,
            CpCorrelation::constant(55.4),
        ),
        Component::new(
            "1-butanol",
            "71-36-3",
            74.123e-3,
            563.05,
            4.423e6,
            274.0e-6,
            0.59,
            390.85,
            CpCorrelation::constant(109.9),
        ),
        Component::new(
            "isopropanol",
            "67-63-0",
            60.096e-3,
            508.31,
            4.764e6,
            220.0e-6,
            0.667,
            355.45,
            CpCorrelation::constant(89.3),
        ),
        Component::new(
            "ethylbenzene",
            "100-41-4",
            106.167e-3,
            617.15,
            3.609e6,
            374.0e-6,
            0.304,
            409.35,
            CpCorrelation::constant(127.9),
        ),
        Component::new(
            "m-xylene",
            "108-38-3",
            106.167e-3,
            617.05,
            3.541e6,
            376.0e-6,
            0.331,
            412.25,
            CpCorrelation::constant(127.5),
        ),
        Component::new(
            "p-xylene",
            "106-42-3",
            106.167e-3,
            616.2,
            3.511e6,
            379.0e-6,
            0.322,
            411.35,
            CpCorrelation::constant(126.9),
        ),
        Component::new(
            "o-xylene",
            "95-47-6",
            106.167e-3,
            630.3,
            3.732e6,
            369.0e-6,
            0.312,
            417.35,
            CpCorrelation::constant(133.3),
        ),
        Component::new(
            "acetonitrile",
            "75-05-8",
            41.053e-3,
            545.5,
            4.83e6,
            173.5e-6,
            0.338,
            354.75,
            CpCorrelation::constant(52.2),
        ),
        Component::new(
            "tetrahydrofuran",
            "109-99-9",
            72.107e-3,
            540.15,
            5.19e6,
            224.0e-6,
            0.281,
            339.15,
            CpCorrelation::constant(79.5),
        ),
        Component::new(
            "naphthalene",
            "91-20-3",
            128.174e-3,
            748.4,
            4.05e6,
            413.0e-6,
            0.302,
            491.14,
            CpCorrelation::constant(132.2),
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

#[cfg(test)]
mod database_gate {
    use super::*;

    /// Accuracy gate: canonical, human-readable rendering of every built-in
    /// entry (scalars plus Cp coefficients). `test-data/golden/thermodynamics/
    /// database-baseline.txt` is committed; CI compares it against this
    /// rendering, so any added or changed database entry shows up as a review
    /// request instead of landing silently. See `PROVENANCE.md` for the
    /// per-entry source documentation the review checks against.
    fn canonical_database_text() -> String {
        let db = ChemicalDatabase::builtin();
        let mut out = String::from(
        "# tpt-process built-in chemical database baseline.\n\
         # Regenerate with: TPT_UPDATE_DB_BASELINE=1 cargo test -p tpt-proc-thermo-database -- --ignored update_database_baseline\n\
         # Every diff must be reviewed against PROVENANCE.md (source + verification per entry).\n\
         # name | CAS | MW kg/mol | Tc K | Pc Pa | Vc m3/mol | omega | Tb K | Cp/R coefficients a b c d e\n",
    );
        for name in db.component_names() {
            let c = &db.components[&name];
            let cp = &c.ideal_gas_cp;
            out.push_str(&format!(
            "{} | {} | {:.4e} | {:.4} | {:.4e} | {:.4e} | {:.5} | {:.4} | {:.6} {:.6} {:.6} {:.6} {:.6}\n",
            c.name,
            c.cas_number,
            c.molecular_weight,
            c.critical_temperature,
            c.critical_pressure,
            c.critical_volume,
            c.acentric_factor,
            c.normal_boiling_point,
            cp.a,
            cp.b,
            cp.c,
            cp.d,
            cp.e,
        ));
        }
        out
    }

    fn baseline_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../test-data/golden/thermodynamics/database-baseline.txt")
    }

    #[test]
    fn database_matches_reviewed_baseline() {
        let current = canonical_database_text();
        let path = baseline_path();
        let baseline = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "cannot read {}: {e}; regenerate with TPT_UPDATE_DB_BASELINE=1 \
             cargo test -p tpt-proc-thermo-database -- --ignored update_database_baseline",
                path.display()
            )
        });
        assert_eq!(
            baseline, current,
            "the built-in chemical database changed: review the diff against \
         PROVENANCE.md (two-source verification per property), then \
         regenerate the baseline consciously with TPT_UPDATE_DB_BASELINE=1"
        );
    }

    #[test]
    #[ignore = "writes the baseline file; run explicitly with TPT_UPDATE_DB_BASELINE=1"]
    fn update_database_baseline() {
        if std::env::var("TPT_UPDATE_DB_BASELINE").is_err() {
            panic!(
                "refusing to overwrite the baseline without \
             TPT_UPDATE_DB_BASELINE=1 (review PROVENANCE.md first)"
            );
        }
        let path = baseline_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, canonical_database_text()).unwrap();
        println!("baseline written to {}", path.display());
    }

    #[test]
    fn database_entries_are_physically_plausible() {
        let db = ChemicalDatabase::builtin();
        assert!(
            db.len() >= 50,
            "expected the expanded database, got {}",
            db.len()
        );
        for name in db.component_names() {
            let c = &db.components[&name];
            assert!(
                c.molecular_weight > 0.0 && c.molecular_weight < 1.0,
                "{name}: MW"
            );
            assert!(
                c.critical_temperature > 1.0 && c.critical_temperature < 2_500.0,
                "{name}: Tc"
            );
            assert!(
                c.critical_pressure > 1.0e4 && c.critical_pressure < 5.0e8,
                "{name}: Pc"
            );
            assert!(
                c.critical_volume > 1.0e-6 && c.critical_volume < 2.0e-3,
                "{name}: Vc"
            );
            assert!((-1.0..2.0).contains(&c.acentric_factor), "{name}: omega");
            assert!(
                c.normal_boiling_point > 0.0
                    && c.normal_boiling_point < c.critical_temperature * 1.05,
                "{name}: Tb vs Tc"
            );
            assert!(c.ideal_gas_cp.a.is_finite(), "{name}: Cp a");
        }
    }
}
