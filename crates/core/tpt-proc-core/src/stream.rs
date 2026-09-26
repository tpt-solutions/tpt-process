//! Material streams and their thermodynamic state.

use crate::composition::Composition;
use crate::error::{CoreError, Result};

/// Flow specification of a material stream.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FlowRate {
    /// Mass flow, kg/s.
    Mass(f64),
    /// Molar flow, mol/s.
    Molar(f64),
    /// Volumetric flow, m³/s.
    Volumetric(f64),
}

impl FlowRate {
    /// The numeric value in the enum's SI unit.
    #[must_use]
    pub const fn value(self) -> f64 {
        match self {
            Self::Mass(v) | Self::Molar(v) | Self::Volumetric(v) => v,
        }
    }
}

/// Phase classification of a stream.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PhaseState {
    /// Single liquid phase.
    Liquid,
    /// Single vapor phase.
    Vapor,
    /// Coexisting vapor and liquid with the given molar vapor fraction β.
    TwoPhase {
        /// Molar vapor fraction in [0, 1].
        vapor_fraction: f64,
    },
    /// Temperature/pressure beyond the critical point.
    Supercritical,
    /// Solid phase.
    Solid,
}

impl PhaseState {
    /// True for [`PhaseState::TwoPhase`].
    #[must_use]
    pub const fn is_two_phase(self) -> bool {
        matches!(self, Self::TwoPhase { .. })
    }
}

/// Transport and thermodynamic properties attached to a stream.
///
/// All fields are SI. Unset fields are `NaN` so accidental use of a
/// property that was never computed is loud, not silent. When serialized
/// (with the `serde` feature) unset fields are written as `null` and read
/// back as `NaN`, so JSON round-trips never carry a bare `NaN`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StreamProperties {
    /// Specific enthalpy, J/mol.
    #[cfg_attr(feature = "serde", serde(with = "maybe_finite"))]
    pub enthalpy: f64,
    /// Specific entropy, J/(mol·K).
    #[cfg_attr(feature = "serde", serde(with = "maybe_finite"))]
    pub entropy: f64,
    /// Density, kg/m³.
    #[cfg_attr(feature = "serde", serde(with = "maybe_finite"))]
    pub density: f64,
    /// Dynamic viscosity, Pa·s.
    #[cfg_attr(feature = "serde", serde(with = "maybe_finite"))]
    pub viscosity: f64,
    /// Thermal conductivity, W/(m·K).
    #[cfg_attr(feature = "serde", serde(with = "maybe_finite"))]
    pub thermal_conductivity: f64,
    /// Heat capacity, J/(mol·K).
    #[cfg_attr(feature = "serde", serde(with = "maybe_finite"))]
    pub heat_capacity: f64,
    /// Compressibility factor Z = P·v/(R·T).
    #[cfg_attr(feature = "serde", serde(with = "maybe_finite"))]
    pub compressibility: f64,
}

/// Serialization helpers mapping the NaN "unset" sentinel to `null`.
#[cfg(feature = "serde")]
mod maybe_finite {
    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S: Serializer>(v: &f64, s: S) -> Result<S::Ok, S::Error> {
        if v.is_finite() {
            s.serialize_some(v)
        } else {
            s.serialize_none()
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
        Ok(Option::<f64>::deserialize(d)?.unwrap_or(f64::NAN))
    }
}

impl Default for StreamProperties {
    fn default() -> Self {
        Self {
            enthalpy: f64::NAN,
            entropy: f64::NAN,
            density: f64::NAN,
            viscosity: f64::NAN,
            thermal_conductivity: f64::NAN,
            heat_capacity: f64::NAN,
            compressibility: f64::NAN,
        }
    }
}

impl StreamProperties {
    /// True if every property has been computed (no NaN).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.enthalpy.is_finite()
            && self.entropy.is_finite()
            && self.density.is_finite()
            && self.viscosity.is_finite()
            && self.thermal_conductivity.is_finite()
            && self.heat_capacity.is_finite()
            && self.compressibility.is_finite()
    }
}

/// A material stream: the edge of every process flow diagram.
///
/// Temperature is in K, pressure in Pa, and the composition is aligned with
/// the property package's component list.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MaterialStream {
    id: crate::ids::StreamId,
    name: String,
    #[cfg_attr(feature = "serde", serde(with = "maybe_finite"))]
    temperature: f64,
    #[cfg_attr(feature = "serde", serde(with = "maybe_finite"))]
    pressure: f64,
    flow_rate: Option<FlowRate>,
    composition: Option<Composition>,
    phase: Option<PhaseState>,
    properties: StreamProperties,
}

impl MaterialStream {
    /// Creates a stream with no state yet; use the builder methods to
    /// complete it.
    #[must_use]
    pub fn new(id: impl Into<crate::ids::StreamId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            temperature: f64::NAN,
            pressure: f64::NAN,
            flow_rate: None,
            composition: None,
            phase: None,
            properties: StreamProperties::default(),
        }
    }

    /// Sets temperature (K) and pressure (Pa), validating the domain.
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidQuantity`] if temperature ≤ 0 or
    /// pressure ≤ 0 (absolute).
    pub fn with_state(mut self, temperature: f64, pressure: f64) -> Result<Self> {
        if !temperature.is_finite() || temperature <= 0.0 {
            return Err(CoreError::InvalidQuantity(format!(
                "temperature must be finite and > 0 K (got {temperature})"
            )));
        }
        if !pressure.is_finite() || pressure <= 0.0 {
            return Err(CoreError::InvalidQuantity(format!(
                "pressure must be finite and > 0 Pa (got {pressure})"
            )));
        }
        self.temperature = temperature;
        self.pressure = pressure;
        Ok(self)
    }

    /// Sets the flow rate (any basis).
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidQuantity`] for negative or non-finite
    /// flows.
    pub fn with_flow(mut self, flow: FlowRate) -> Result<Self> {
        if !flow.value().is_finite() || flow.value() < 0.0 {
            return Err(CoreError::InvalidQuantity(format!(
                "flow must be finite and >= 0 (got {})",
                flow.value()
            )));
        }
        self.flow_rate = Some(flow);
        Ok(self)
    }

    /// Sets the composition.
    pub fn with_composition(mut self, composition: Composition) -> Self {
        self.composition = Some(composition);
        self
    }

    /// Sets the phase state.
    pub fn with_phase(mut self, phase: PhaseState) -> Self {
        self.phase = Some(phase);
        self
    }

    /// Replaces the attached properties block.
    pub fn with_properties(mut self, properties: StreamProperties) -> Self {
        self.properties = properties;
        self
    }

    /// Relabels the stream (solvers rename computed outlets to the id of
    /// the connection they feed).
    #[must_use]
    pub fn with_id(mut self, id: crate::ids::StreamId) -> Self {
        self.id = id;
        self
    }

    /// Stream identifier.
    #[must_use]
    pub const fn id(&self) -> crate::ids::StreamId {
        self.id
    }

    /// Human-readable stream name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Temperature in K (NaN until set).
    #[must_use]
    pub const fn temperature(&self) -> f64 {
        self.temperature
    }

    /// Pressure in Pa (NaN until set).
    #[must_use]
    pub const fn pressure(&self) -> f64 {
        self.pressure
    }

    /// Flow rate, if set.
    pub const fn flow_rate(&self) -> Option<FlowRate> {
        self.flow_rate
    }

    /// Numeric flow value; errors if unset.
    ///
    /// # Errors
    /// [`CoreError::MissingReference`] if no flow was assigned.
    pub fn total_flow(&self) -> Result<f64> {
        self.flow_rate
            .map(FlowRate::value)
            .ok_or_else(|| CoreError::MissingReference(format!("stream {} has no flow", self.id)))
    }

    /// Composition, if set.
    pub const fn composition(&self) -> Option<&Composition> {
        self.composition.as_ref()
    }

    /// Phase state, if set.
    pub const fn phase(&self) -> Option<PhaseState> {
        self.phase
    }

    /// Attached properties block.
    #[must_use]
    pub const fn properties(&self) -> &StreamProperties {
        &self.properties
    }

    /// Mutable access to the properties block (for solvers to fill in).
    pub fn properties_mut(&mut self) -> &mut StreamProperties {
        &mut self.properties
    }

    /// True if temperature, pressure, and composition are all set.
    #[must_use]
    pub const fn is_fully_specified(&self) -> bool {
        self.temperature.is_finite() && self.pressure.is_finite() && self.composition.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Result<MaterialStream> {
        let composition = Composition::from_mole_fractions(&[0.7, 0.3])?;
        Ok(MaterialStream::new(1, "feed")
            .with_state(350.0, 101_325.0)?
            .with_flow(FlowRate::Molar(100.0))?
            .with_composition(composition)
            .with_phase(PhaseState::Liquid))
    }

    #[test]
    fn builder_roundtrip() {
        let s = sample().unwrap();
        assert_eq!(s.id(), crate::ids::StreamId(1));
        assert_eq!(s.temperature(), 350.0);
        assert_eq!(s.pressure(), 101_325.0);
        assert_eq!(s.total_flow().unwrap(), 100.0);
        assert!(s.is_fully_specified());
    }

    #[test]
    fn rejects_unphysical_state() {
        assert!(MaterialStream::new(1, "s").with_state(-5.0, 1e5).is_err());
        assert!(MaterialStream::new(1, "s").with_state(300.0, 0.0).is_err());
    }

    #[test]
    fn rejects_negative_flow() {
        assert!(MaterialStream::new(1, "s")
            .with_flow(FlowRate::Mass(-1.0))
            .is_err());
    }

    #[test]
    fn unset_flow_is_an_error_not_a_default() {
        let s = MaterialStream::new(1, "s").with_state(300.0, 1e5).unwrap();
        assert!(s.total_flow().is_err());
    }

    #[test]
    fn unset_properties_are_nan() {
        let s = sample().unwrap();
        assert!(!s.properties().is_complete());
        assert!(s.properties().enthalpy.is_nan());
    }
}
