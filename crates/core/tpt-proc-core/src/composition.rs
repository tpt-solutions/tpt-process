//! Mixture compositions.
//!
//! A [`Composition`] is a dense vector of fractions aligned with the
//! component list of the property package the stream is evaluated against
//! (index 0 = package component 0, and so on). Index alignment keeps the
//! numerics allocation-free and deterministic; the package supplies
//! component identity.

use crate::error::{CoreError, Result};

/// Basis of the fractions stored in a [`Composition`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CompositionBasis {
    /// Mole fractions (default; required by most solvers).
    #[default]
    Mole,
    /// Mass fractions.
    Mass,
    /// Volumetric fractions (ideal mixing).
    Volume,
}

/// Composition of a mixture as fractions over the package component list.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Composition {
    fractions: Vec<f64>,
    basis: CompositionBasis,
}

impl Composition {
    /// Builds a composition from a slice of fractions.
    ///
    /// The fractions must be non-negative, finite, and sum to 1 within a
    /// tolerance of 1e-6; otherwise [`CoreError::InvalidComposition`] is
    /// returned. Use [`Composition::from_parts`] to normalize an
    /// unnormalized set of amounts.
    pub fn from_mole_fractions(fractions: &[f64]) -> Result<Self> {
        Self::validate(fractions)?;
        Ok(Self {
            fractions: fractions.to_vec(),
            basis: CompositionBasis::Mole,
        })
    }

    /// Builds a composition on an explicit basis with validation.
    pub fn from_parts(fractions: &[f64], basis: CompositionBasis) -> Result<Self> {
        Self::validate(fractions)?;
        Ok(Self {
            fractions: fractions.to_vec(),
            basis,
        })
    }

    /// Builds a composition from component amounts, normalizing them to
    /// fractions. At least one amount must be positive.
    pub fn from_amounts(amounts: &[f64], basis: CompositionBasis) -> Result<Self> {
        if amounts.is_empty() {
            return Err(CoreError::InvalidComposition("empty composition".into()));
        }
        let total: f64 = amounts
            .iter()
            .map(|a| {
                if a.is_finite() && *a >= 0.0 {
                    *a
                } else {
                    f64::NAN
                }
            })
            .sum();
        if !total.is_finite() || total <= 0.0 {
            return Err(CoreError::InvalidComposition(
                "amounts must be non-negative, finite, and sum to a positive value".into(),
            ));
        }
        let fractions: Vec<f64> = amounts.iter().map(|a| a / total).collect();
        Ok(Self { fractions, basis })
    }

    /// A single-component composition with mole fraction 1.
    pub fn pure(num_components: usize, index: usize) -> Result<Self> {
        if index >= num_components {
            return Err(CoreError::InvalidComposition(format!(
                "component index {index} out of range (0..{num_components})"
            )));
        }
        let mut fractions = vec![0.0; num_components];
        fractions[index] = 1.0;
        Ok(Self {
            fractions,
            basis: CompositionBasis::Mole,
        })
    }

    fn validate(fractions: &[f64]) -> Result<()> {
        if fractions.is_empty() {
            return Err(CoreError::InvalidComposition("empty composition".into()));
        }
        let sum: f64 = fractions.iter().sum();
        if fractions.iter().any(|f| !f.is_finite() || *f < 0.0) {
            return Err(CoreError::InvalidComposition(
                "fractions must be finite and non-negative".into(),
            ));
        }
        if (sum - 1.0).abs() > 1e-6 {
            return Err(CoreError::InvalidComposition(format!(
                "fractions must sum to 1 (got {sum:.9}); use from_amounts to normalize"
            )));
        }
        Ok(())
    }

    /// Number of components.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fractions.len()
    }

    /// True if there are no components.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fractions.is_empty()
    }

    /// Fraction of component `index`.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<f64> {
        self.fractions.get(index).copied()
    }

    /// The fraction vector.
    #[must_use]
    pub fn as_slice(&self) -> &[f64] {
        &self.fractions
    }

    /// The basis of the stored fractions.
    #[must_use]
    pub const fn basis(&self) -> CompositionBasis {
        self.basis
    }

    /// Sum of the fractions (1 within floating-point error for a valid
    /// composition).
    #[must_use]
    pub fn sum(&self) -> f64 {
        self.fractions.iter().sum()
    }

    /// Returns the fractions renormalized to sum exactly to 1.
    #[must_use]
    pub fn normalized(&self) -> Self {
        let total = self.sum();
        let fractions = if total > 0.0 && total.is_finite() {
            self.fractions.iter().map(|f| f / total).collect()
        } else {
            self.fractions.clone()
        };
        Self {
            fractions,
            basis: self.basis,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_composition_roundtrip() {
        let c = Composition::from_mole_fractions(&[0.2, 0.3, 0.5]).unwrap();
        assert_eq!(c.len(), 3);
        assert_eq!(c.get(1), Some(0.3));
        assert!((c.sum() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn rejects_negative_fractions() {
        assert!(Composition::from_mole_fractions(&[-0.1, 1.1]).is_err());
    }

    #[test]
    fn rejects_unnormalized_fractions() {
        assert!(Composition::from_mole_fractions(&[0.5, 0.6]).is_err());
    }

    #[test]
    fn rejects_nan() {
        assert!(Composition::from_mole_fractions(&[f64::NAN, 1.0]).is_err());
    }

    #[test]
    fn from_amounts_normalizes() {
        let c = Composition::from_amounts(&[1.0, 3.0], CompositionBasis::Mole).unwrap();
        assert!((c.get(0).unwrap() - 0.25).abs() < 1e-12);
        assert!((c.get(1).unwrap() - 0.75).abs() < 1e-12);
    }

    #[test]
    fn pure_composition() {
        let c = Composition::pure(3, 1).unwrap();
        assert_eq!(c.as_slice(), &[0.0, 1.0, 0.0]);
        assert!(Composition::pure(3, 3).is_err());
    }
}
