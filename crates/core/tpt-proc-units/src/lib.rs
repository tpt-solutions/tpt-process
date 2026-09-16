//! Shared unit-operation scaffolding.
//!
//! This crate defines *how a unit behaves* — the [`UnitBehavior`] contract
//! mapping inlet streams to outlet streams — plus the two units whose
//! physics need no thermodynamics: [`Mixer`] and [`Splitter`]. Richer units
//! (heat exchangers, pumps, reactors, …) implement the same trait in their
//! own `tpt-proc-*` crates so the flowsheet solver can execute any mixture
//! of them through a [`UnitRegistry`].
//!
//! The unit contract is deliberately stateless: `solve` is a pure function
//! of the inlets and the unit's own parameters. State lives in the
//! flowsheet's streams, which keeps sequential-modular execution trivially
//! reproducible.
//!
//! # Example
//!
//! ```
//! use tpt_proc_core::{Composition, FlowRate, MaterialStream};
//! use tpt_proc_units::{Mixer, UnitBehavior};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mixer = Mixer;
//! let a = MaterialStream::new(1, "a")
//!     .with_state(300.0, 101_325.0)?
//!     .with_flow(FlowRate::Molar(60.0))?
//!     .with_composition(Composition::from_mole_fractions(&[1.0, 0.0])?);
//! let b = MaterialStream::new(2, "b")
//!     .with_state(350.0, 101_325.0)?
//!     .with_flow(FlowRate::Molar(40.0))?
//!     .with_composition(Composition::from_mole_fractions(&[0.0, 1.0])?);
//!
//! let outlets = mixer.solve(&[a, b])?;
//! assert_eq!(outlets.len(), 1);
//! // 60 + 40 mol/s conserved on the molar basis.
//! assert!((outlets[0].total_flow()? - 100.0).abs() < 1e-9);
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

use tpt_proc_core::{CoreError, MaterialStream, UnitId};

mod error;
mod mixer;
mod splitter;

pub use error::UnitError;
pub use mixer::Mixer;
pub use splitter::Splitter;

/// Behavior of one unit operation: pure mapping from inlets to outlets.
pub trait UnitBehavior: Send + Sync {
    /// Kind name (diagnostics, PFD export).
    fn kind(&self) -> &'static str;

    /// Number of inlet ports.
    fn num_inlets(&self) -> usize;

    /// Number of outlet ports.
    fn num_outlets(&self) -> usize;

    /// Computes the outlet streams from the inlets.
    ///
    /// # Errors
    /// [`UnitError::WrongInletCount`] when `inlets.len() != num_inlets()`;
    /// [`UnitError::MissingStreamData`] when an inlet lacks state the model
    /// needs; model-specific errors otherwise.
    fn solve(&self, inlets: &[MaterialStream]) -> Result<Vec<MaterialStream>, UnitError>;
}

/// Maps a [`CoreError`] onto the unit error type.
impl From<CoreError> for UnitError {
    fn from(e: CoreError) -> Self {
        Self::MissingStreamData(e.to_string())
    }
}

/// Registry mapping unit ids to behaviors, used by the flowsheet solver.
///
/// Ordered internally (`BTreeMap`) so iteration is deterministic.
#[derive(Clone, Default)]
pub struct UnitRegistry {
    behaviors: BTreeMap<UnitId, Arc<dyn UnitBehavior>>,
}

impl UnitRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers (or replaces) the behavior for a unit id.
    pub fn register(&mut self, unit: UnitId, behavior: Arc<dyn UnitBehavior>) {
        self.behaviors.insert(unit, behavior);
    }

    /// The registered behavior for a unit id.
    pub fn get(&self, unit: UnitId) -> Option<Arc<dyn UnitBehavior>> {
        self.behaviors.get(&unit).cloned()
    }

    /// Number of registered units.
    #[must_use]
    pub fn len(&self) -> usize {
        self.behaviors.len()
    }

    /// True if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.behaviors.is_empty()
    }

    /// Executes one unit against its inlets.
    ///
    /// # Errors
    /// [`UnitError::UnknownUnit`] when the id is unregistered;
    /// otherwise whatever the behavior's [`UnitBehavior::solve`] returns.
    pub fn execute(
        &self,
        unit: UnitId,
        inlets: &[MaterialStream],
    ) -> Result<Vec<MaterialStream>, UnitError> {
        let behavior = self
            .behaviors
            .get(&unit)
            .ok_or(UnitError::UnknownUnit(unit))?;
        behavior.solve(inlets)
    }
}

impl std::fmt::Debug for UnitRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnitRegistry")
            .field("behaviors", &self.behaviors.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_proc_core::{Composition, FlowRate};

    fn inlet(id: u64, t: f64, flow: f64, x: [f64; 2]) -> MaterialStream {
        MaterialStream::new(id, format!("in{id}"))
            .with_state(t, 101_325.0)
            .unwrap()
            .with_flow(FlowRate::Molar(flow))
            .unwrap()
            .with_composition(Composition::from_mole_fractions(&x).unwrap())
    }

    #[test]
    fn mixer_conserves_flow_and_mixes_composition() {
        let mixer = Mixer;
        let a = inlet(1, 300.0, 60.0, [1.0, 0.0]);
        let b = inlet(2, 350.0, 40.0, [0.0, 1.0]);
        let out = mixer.solve(&[a, b]).unwrap();
        assert_eq!(out.len(), 1);
        assert!((out[0].total_flow().unwrap() - 100.0).abs() < 1e-9);
        let c = out[0].composition().unwrap();
        assert!((c.get(0).unwrap() - 0.6).abs() < 1e-9);
        // Flow-weighted outlet temperature: 0.6·300 + 0.4·350 = 320.
        assert!((out[0].temperature() - 320.0).abs() < 1e-9);
        // Outlet pressure is the minimum inlet pressure.
        assert_eq!(out[0].pressure(), 101_325.0);
    }

    #[test]
    fn mixer_rejects_wrong_inlet_count() {
        let mixer = Mixer;
        let a = inlet(1, 300.0, 1.0, [1.0, 0.0]);
        assert!(matches!(
            mixer.solve(&[]),
            Err(UnitError::WrongInletCount {
                expected: 2,
                got: 0
            })
        ));
        let _ = a;
    }

    #[test]
    fn mixer_requires_composition_on_all_inlets() {
        let mixer = Mixer;
        let bare = MaterialStream::new(1, "x").with_state(300.0, 1e5).unwrap();
        let b = inlet(2, 300.0, 1.0, [1.0, 0.0]);
        assert!(matches!(
            mixer.solve(&[bare, b]),
            Err(UnitError::MissingStreamData(_))
        ));
    }

    #[test]
    fn splitter_splits_and_normalizes_ratios() {
        let splitter = Splitter::new(vec![0.7, 0.4, 0.1]); // sums to 1.2 → normalized
        let feed = inlet(1, 320.0, 100.0, [0.5, 0.5]);
        let out = splitter.solve(&[feed]).unwrap();
        assert_eq!(out.len(), 3);
        let flows: Vec<f64> = out.iter().map(|s| s.total_flow().unwrap()).collect();
        assert!((flows[0] - 58.333333333333336).abs() < 1e-9);
        assert!((flows[1] - 33.333333333333336).abs() < 1e-9);
        assert!((flows[2] - 8.333333333333334).abs() < 1e-9);
        let total: f64 = flows.iter().sum();
        assert!((total - 100.0).abs() < 1e-9);
    }

    #[test]
    fn registry_dispatch() {
        let mut registry = UnitRegistry::new();
        registry.register(tpt_proc_core::UnitId(1), Arc::new(Mixer));
        registry.register(
            tpt_proc_core::UnitId(2),
            Arc::new(Splitter::new(vec![0.5, 0.5])),
        );

        let a = inlet(1, 300.0, 1.0, [1.0, 0.0]);
        let b = inlet(2, 300.0, 1.0, [0.0, 1.0]);
        let out = registry
            .execute(tpt_proc_core::UnitId(1), &[a.clone(), b])
            .unwrap();
        assert_eq!(out.len(), 1);

        assert!(matches!(
            registry.execute(tpt_proc_core::UnitId(9), &[a]),
            Err(UnitError::UnknownUnit(_))
        ));

        // Splitter via the registry: feed the mixed outlet.
        let mixed = MaterialStream::new(3, "m")
            .with_state(300.0, 1e5)
            .unwrap()
            .with_flow(FlowRate::Molar(2.0))
            .unwrap()
            .with_composition(Composition::from_mole_fractions(&[0.5, 0.5]).unwrap());
        let split = registry
            .execute(tpt_proc_core::UnitId(2), &[mixed])
            .unwrap();
        assert_eq!(split.len(), 2);
        assert_eq!(
            split[0].total_flow().unwrap(),
            split[1].total_flow().unwrap()
        );
    }
}
