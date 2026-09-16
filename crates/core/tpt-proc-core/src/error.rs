//! Errors shared across the core crate.

use core::fmt;

/// Convenience alias for core results.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Errors raised when building or validating core domain objects.
#[derive(Clone, PartialEq, Debug)]
pub enum CoreError {
    /// A composition fraction was negative, NaN, or the vector was empty.
    InvalidComposition(String),
    /// A referenced id (stream, unit, port) does not exist.
    MissingReference(String),
    /// A structural invariant of the flowsheet was violated.
    InvalidFlowsheet(String),
    /// A physical quantity was outside its valid domain.
    InvalidQuantity(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidComposition(m) => write!(f, "invalid composition: {m}"),
            Self::MissingReference(m) => write!(f, "missing reference: {m}"),
            Self::InvalidFlowsheet(m) => write!(f, "invalid flowsheet: {m}"),
            Self::InvalidQuantity(m) => write!(f, "invalid quantity: {m}"),
        }
    }
}

impl std::error::Error for CoreError {}
