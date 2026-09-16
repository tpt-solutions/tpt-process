//! Phase classification and property-method selection types.

/// Thermodynamic phase for property evaluation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Vapor phase (largest compressibility root).
    Vapor,
    /// Liquid phase (smallest compressibility root).
    Liquid,
    /// Supercritical fluid (single root regime).
    Supercritical,
    /// Solid phase (rarely modeled; rejection temperature).
    Solid,
}

impl Phase {
    /// True for [`Phase::Vapor`].
    #[must_use]
    pub const fn is_vapor(self) -> bool {
        matches!(self, Self::Vapor)
    }

    /// True for [`Phase::Liquid`].
    #[must_use]
    pub const fn is_liquid(self) -> bool {
        matches!(self, Self::Liquid)
    }

    /// Converts the stream-level phase state of `tpt-proc-core` into the
    /// phase used for property evaluation (a two-phase state cannot be a
    /// single evaluation phase; use the flash solver instead).
    #[must_use]
    pub const fn from_stream_state(state: tpt_proc_core::PhaseState) -> Option<Self> {
        match state {
            tpt_proc_core::PhaseState::Liquid => Some(Self::Liquid),
            tpt_proc_core::PhaseState::Vapor => Some(Self::Vapor),
            tpt_proc_core::PhaseState::Supercritical => Some(Self::Supercritical),
            tpt_proc_core::PhaseState::Solid => Some(Self::Solid),
            tpt_proc_core::PhaseState::TwoPhase { .. } => None,
        }
    }
}

/// Which real root of a cubic EOS to select at a (T, P) state point.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PhaseSelection {
    /// Largest real root (vapor-like).
    Vapor,
    /// Smallest real root (liquid-like).
    Liquid,
    /// Root with the lowest Gibbs energy (physically stable phase);
    /// ties resolved toward vapor.
    Stable,
}

/// Property-method selection attached to a [`crate::PropertyPackage`].
#[derive(Clone, Debug, PartialEq, Default)]
pub enum PropertyMethods {
    /// Ideal-gas / ideal-solution behavior everywhere.
    #[default]
    IdealGas,
    /// Cubic equation of state for vapor/liquid fugacities.
    CubicEos,
    /// Activity-coefficient model for the liquid, ideal vapor.
    ActivityCoefficient,
    /// Activity model + Henry components for gas solubility.
    HenrysLaw,
}
