//! Unit-level error type.

use std::fmt;
use tpt_proc_core::UnitId;

/// Errors raised while solving a unit operation.
#[derive(Clone, PartialEq, Debug)]
pub enum UnitError {
    /// The unit received the wrong number of inlet streams.
    WrongInletCount {
        /// Expected inlet count.
        expected: usize,
        /// Actual inlet count.
        got: usize,
    },
    /// An inlet (or its state) was missing data the model requires.
    MissingStreamData(String),
    /// The id is not present in the [`UnitRegistry`](crate::UnitRegistry).
    UnknownUnit(UnitId),
    /// The model does not support the requested configuration.
    Unsupported(String),
    /// An internal iteration failed to converge within limits.
    NotConverged {
        /// Iteration cap that was reached.
        iterations: u32,
    },
}

impl fmt::Display for UnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongInletCount { expected, got } => {
                write!(f, "wrong inlet count: expected {expected}, got {got}")
            }
            Self::MissingStreamData(m) => write!(f, "missing stream data: {m}"),
            Self::UnknownUnit(id) => write!(f, "unknown unit {id}"),
            Self::Unsupported(m) => write!(f, "unsupported configuration: {m}"),
            Self::NotConverged { iterations } => {
                write!(f, "unit did not converge in {iterations} iterations")
            }
        }
    }
}

impl std::error::Error for UnitError {}
