//! Catalysis: internal effectiveness factor, Thiele modulus, and catalyst
//! deactivation models.
//!
//! # Example
//!
//! ```
//! use tpt_proc_catalysis::{
//!     deactivation_factor, effectiveness_factor, DeactivationModel, PelletGeometry,
//! };
//!
//! // First-order reaction in a sphere at Thiele modulus 3.
//! let eta = effectiveness_factor(3.0, PelletGeometry::Sphere, 1.0);
//! assert!((0.6..0.75).contains(&eta)); // diffusion-limited but active
//!
//! // Exponential deactivation after 1 year of a 5%/month decay.
//! let activity = deactivation_factor(0.05, 12.0, DeactivationModel::Exponential);
//! assert!(activity < 0.6);
//! ```

#![forbid(unsafe_code)]

/// Pellet geometry for the internal effectiveness factor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PelletGeometry {
    /// Infinite slab: shape factor n = 1.
    Slab,
    /// Long cylinder: n = 2.
    Cylinder,
    /// Sphere: n = 3.
    Sphere,
}

impl PelletGeometry {
    /// Characteristic shape factor for the asymptotic effectiveness
    /// η ≈ n/φ at large φ.
    #[must_use]
    pub const fn shape_factor(&self) -> f64 {
        match self {
            Self::Slab => 1.0,
            Self::Cylinder => 2.0,
            Self::Sphere => 3.0,
        }
    }
}

/// Thiele modulus for a first-order reaction in a porous pellet:
/// φ = R·√(k·ρ_p·S_a/D_eff).
#[must_use]
pub fn thiele_modulus(
    pellet_radius_m: f64,
    rate_constant: f64,
    effective_diffusivity_m2_s: f64,
) -> f64 {
    if effective_diffusivity_m2_s <= 0.0 {
        return f64::INFINITY;
    }
    pellet_radius_m * (rate_constant / effective_diffusivity_m2_s).sqrt()
}

/// Internal effectiveness factor for a first-order reaction:
///
/// - slab: η = tanh(φ)/φ
/// - sphere: η = (3/φ²)·(φ·coth φ − 1)
/// - cylinder: η = (2/φ)·I₁(2φ)/I₀(2φ) approximated by the slab/sphere
///   interpolation η = tanh(φ)/φ blended with the sphere form.
///
/// The general large-φ limit η → shape_factor/φ is honored for all
/// geometries.
#[must_use]
pub fn effectiveness_factor(thiele_modulus: f64, geometry: PelletGeometry, order: f64) -> f64 {
    let phi = thiele_modulus;
    if !phi.is_finite() || phi <= 0.0 {
        return 1.0;
    }
    if (order - 1.0).abs() > 1e-9 {
        // Non-first-order: generalized modulus approximation via the
        // first-order curve on the modified modulus.
        let eta1 = effectiveness_factor(phi, geometry, 1.0);
        return eta1.powf((2.0 - order).max(0.5));
    }
    match geometry {
        PelletGeometry::Slab => phi.tanh() / phi,
        PelletGeometry::Sphere => {
            if phi < 1e-3 {
                1.0 - phi * phi / 15.0
            } else {
                3.0 * (phi / phi.tanh() - 1.0) / (phi * phi)
            }
        }
        PelletGeometry::Cylinder => {
            // Modified Bessel ratio approximation: blend slab (n=1) and
            // sphere (n=3) asymptotics to the cylinder limit 2/φ.
            let slab = phi.tanh() / phi;
            let sphere = if phi < 1e-3 {
                1.0
            } else {
                3.0 * (phi / phi.tanh() - 1.0) / (phi * phi)
            };
            (slab + sphere) / 2.0
        }
    }
}

/// Catalyst deactivation models.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeactivationModel {
    /// a = exp(−k_d·t).
    Exponential,
    /// Linear decay clamped at zero: a = max(1 − k_d·t, 0).
    Linear,
    /// Empirical power law: a = (1 + k_d·t)^(−1/2).
    PowerLaw,
}

/// Catalyst activity a(t) ∈ (0, 1].
#[must_use]
pub fn deactivation_factor(k_d: f64, time_s: f64, model: DeactivationModel) -> f64 {
    if k_d <= 0.0 || !k_d.is_finite() {
        return 1.0;
    }
    let t = time_s.max(0.0);
    let a = match model {
        DeactivationModel::Exponential => (-k_d * t).exp(),
        DeactivationModel::Linear => (1.0 - k_d * t).max(0.0),
        DeactivationModel::PowerLaw => (1.0 + k_d * t).powf(-0.5),
    };
    a.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_modulus_is_unity() {
        // φ → 0: no internal limitation.
        assert!((effectiveness_factor(1e-3, PelletGeometry::Sphere, 1.0) - 1.0).abs() < 1e-5);
        assert!(
            (effectiveness_factor(0.1, PelletGeometry::Slab, 1.0) - 0.1f64.tanh() / 0.1).abs()
                < 1e-12
        );
    }

    #[test]
    fn sphere_asymptote_is_three_over_phi() {
        // Finite-φ correction is −3/φ²; at φ = 30 that is ~3% relative.
        let eta = effectiveness_factor(30.0, PelletGeometry::Sphere, 1.0);
        assert!(
            (eta - 3.0 / 30.0).abs() < 5e-3,
            "η = {eta}, asymptote = {}",
            3.0 / 30.0
        );
    }

    #[test]
    fn effectiveness_decreases_with_diffusion_limitation() {
        let weak = effectiveness_factor(0.5, PelletGeometry::Sphere, 1.0);
        let strong = effectiveness_factor(5.0, PelletGeometry::Sphere, 1.0);
        assert!(weak > strong);
        assert!(strong < 1.0);
    }

    #[test]
    fn deactivation_models_behavior() {
        let e = deactivation_factor(0.05, 12.0, DeactivationModel::Exponential);
        assert!((e - (-(0.05 * 12.0_f64)).exp()).abs() < 1e-12);
        // Linear reaches exactly zero at t = 1/k.
        assert_eq!(
            deactivation_factor(0.1, 10.0, DeactivationModel::Linear),
            0.0
        );
        // Power law decays slowest of the three at long time.
        let p = deactivation_factor(0.05, 12.0, DeactivationModel::PowerLaw);
        assert!(p > e);
        // Zero decay constant = fresh catalyst.
        assert_eq!(
            deactivation_factor(0.0, 1e6, DeactivationModel::Exponential),
            1.0
        );
    }

    #[test]
    fn thiele_modulus_definition() {
        // φ = R·√(k/D_eff) = 0.005·√(1/1e-6) = 5.
        let phi = thiele_modulus(0.005, 1.0, 1e-6);
        assert!((phi - 5.0).abs() < 1e-9);
    }
}
