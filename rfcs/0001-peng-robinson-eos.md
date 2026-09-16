# RFC 0001 — Peng-Robinson Equation of State

- **Status:** Accepted
- **Start date:** 2026-01
- **Crate:** `tpt-proc-thermo-eos`

## Summary

Adopt the Peng-Robinson (PR) cubic equation of state as the default
pressure–volume–temperature and fugacity model for non-ideal vapor/liquid
phases in `tpt-process`, with the classical van der Waals one-fluid mixing
rule as the default and a path to Huron-Vidal / Wong-Sandler mixing.

## Motivation

Process simulation needs a general-purpose EOS that handles the light gases
and hydrocarbons of typical flowsheets with industry-accepted accuracy.
PR is the de-facto industrial standard for hydrocarbon systems, is analytically
solvable (cubic in Z), is cheap enough for microsecond flash iterations in
real-time control, and its constants derive from critical properties and the
acentric factor alone — no proprietary databank required.

## Detailed design

### The equation

```
P = R·T / (v − b) − a·α / (v² + 2·b·v − b²)
```

Pure-component parameters:

```
a_i(T) = 0.45724 · R²·Tc² / Pc · α_i(T)
b_i    = 0.07780 · R·Tc / Pc
α_i(T) = [1 + κ_i·(1 − √(Tr))]²
κ_i    = 0.37464 + 1.54226·ω − 0.26992·ω²
```

### Mixing rules

Default — classical van der Waals one-fluid:

```
a_mix = Σᵢ Σⱼ xᵢ·xⱼ·√(aᵢ·aⱼ)·(1 − kᵢⱼ)      b_mix = Σ xᵢ·bᵢ
```

Binary interaction parameters `kᵢⱼ` come from `tpt-proc-thermo-database`
(default 0 when unknown, symmetric, kᵢᵢ = 0). Huron-Vidal and Wong-Sandler
are provided for polar systems by coupling to `tpt-proc-thermo-activity`
models; they translate the excess-Gibbs model into EOS `a` at each state
point.

### Root selection

Compressibility is obtained from the cubic `Z³ + c₂·Z² + c₁·Z + c₀ = 0`
solved via a numerically robust trigonometric method (analytic roots,
no iteration). For multiphase regions, return all real roots; the flash
layer in `tpt-proc-thermo-phase` selects the root with the lowest Gibbs
energy (minimum |fugacity integral|), not "largest Z = vapor".

### Derived properties

- Fugacity coefficients (component and mixture), per phase:

```
ln φᵢ = (bᵢ/b)·(Z−1) − ln(Z−B)
        − A/(2√2·B) · [2·Σⱼ xⱼ·aᵢⱼ/a − bᵢ/b] · ln[(Z + (1+√2)·B)/(Z + (1−√2)·B)]
```

- Enthalpy/entropy departure functions from the standard PR integrals.
- Pure-component vapor pressure by fugacity-equality root finding
  (used for Wilson K-value initialization and database validation).

### API surface

```rust
pub struct PengRobinson { /* components, k_ij matrix */ }
impl PengRobinson {
    pub fn pressure(&self, comp: &Composition, t: f64, v: f64) -> f64;
    pub fn compressibility_factors(&self, comp: &Composition, t: f64, p: f64)
        -> Vec<f64>;                     // 1 or 3 real roots
    pub fn fugacity_coefficients(&self, comp: &Composition, t: f64, p: f64,
        root: RootSelection) -> Vec<f64>;
    pub fn enthalpy_departure(&self, /* ... */) -> f64;
    pub fn vapor_pressure(&self, component: &Component, t: f64) -> Result<f64>;
}
```

The EOS implements the `PropertyPackage` trait from `tpt-proc-core`, so the
flash solver and flowsheet engine consume PR and SRK interchangeably.

SRK ships in the same crate with identical structure (different constants
`Ωa = 0.42747`, `Ωb = 0.08664`, `m(ω)` from Soave) to allow like-for-like
comparison.

## Verification & validation

- Saturation pressure of water at 373.15 K within 5% of 101 325 Pa;
  same tolerance class for methane, propane, benzene at reference
  temperatures (NIST WebBook reference data).
- Critical-point consistency: `Zc(PR) ≈ 0.307` for pure components.
- Ideal-gas limit: at low density, `Z → 1` and departures → 0.
- Golden files: `test-data/golden/thermodynamics/pr-vapor-pressure.json`.

## Alternatives considered

- **SRK only:** worse liquid densities for hydrocarbons; keep both.
- **PR78 / volume-translated PR:** better liquid density, deferred — needs
  volume translation parameters per component; add later behind an option.
- **Multi-parameter EOS (BWR/GERG):** high accuracy, heavy data and license
  questions (GERG-2008 documents are paywalled); out of scope for the MIT
  chain.
- **CPA (cubic-plus-association):** needed for water/alcohol non-ideality
  at high fidelity; deferred to a future RFC; NRTL/activity models cover
  the immediate need.

## Unresolved questions

- Default source of `kᵢⱼ` when the database lacks a pair: 0 (current) vs
  a group-contribution estimate (e.g., via UNIFAC-derived LLE/VLE data)?
  Revisit after Phase 3.
- Solid/precipitate phases are out of scope for this RFC.
