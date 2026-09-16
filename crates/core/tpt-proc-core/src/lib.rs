//! Core domain types for chemical process engineering.
//!
//! `tpt-proc-core` defines the data model every other `tpt-proc-*` crate
//! builds on: material streams, compositions, phase states, unit-operation
//! specifications, connections, and flowsheets. It also hosts the
//! [`PropertyPackage`] trait — the contract thermodynamic packages implement
//! so the flash solver and flowsheet engine can consume any model
//! interchangeably.
//!
//! # Conventions
//!
//! - **Units are SI** unless a name states otherwise: temperature in K,
//!   pressure in Pa, energy in J, amount in mol, length in m, time in s.
//! - **Determinism:** compositions are index-aligned vectors ordered by the
//!   owning property package's component list; maps are ordered
//!   (`BTreeMap`), so identical inputs give identical results.
//! - **No panics:** library code returns [`Result`]; construction of
//!   physically invalid data (e.g. a negative mole fraction) is rejected at
//!   the boundary.
//!
//! # Example
//!
//! ```
//! use tpt_proc_core::{Composition, FlowRate, MaterialStream, PhaseState};
//!
//! # fn main() -> Result<(), tpt_proc_core::CoreError> {
//! let composition = Composition::from_mole_fractions(&[0.7, 0.3])?;
//! let stream = MaterialStream::new(1, "feed")
//!     .with_state(350.0, 101_325.0)?
//!     .with_flow(FlowRate::Molar(100.0))?
//!     .with_composition(composition)
//!     .with_phase(PhaseState::Liquid);
//!
//! assert!((stream.total_flow()? - 100.0).abs() < 1e-12);
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

pub mod composition;
pub mod error;
pub mod flowsheet;
pub mod ids;
pub mod property;
pub mod stream;
pub mod units;

pub use composition::{Composition, CompositionBasis};
pub use error::{CoreError, Result};
pub use flowsheet::{Connection, Flowsheet};
pub use ids::{ComponentId, FlowsheetId, PortId, StreamId, UnitId};
pub use property::PropertyPackage;
pub use stream::{FlowRate, MaterialStream, PhaseState, StreamProperties};
pub use units::{
    AbsorberConfig, ColumnConfig, CompressorConfig, CrystallizerConfig, HeatExchangerConfig,
    PumpConfig, ReactorConfig, UnitOperation, ValveConfig,
};
