//! Mixing-rule selection.
//!
//! The classical van der Waals one-fluid rule is implemented. Huron-Vidal
//! and Wong-Sandler translate an excess-Gibbs activity model into the EOS
//! attraction parameter; they require the activity crate and are wired up
//! when `tpt-proc-thermo-activity` lands (RFC 0001).

/// Mixing rule for the EOS attraction parameter.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum MixingRule {
    /// Classical one-fluid: a = ΣΣ xᵢxⱼ√(aᵢaⱼ)(1−kᵢⱼ), b = Σ xᵢbᵢ.
    #[default]
    VanDerWaals,
    /// Huron-Vidal: a from the zero-pressure limit of G^E/RT.
    HuronVidal,
    /// Wong-Sandler: a from G^E held constant between low/high pressure.
    WongSandler,
}
