# RFC 0002 — Flash Calculations

- **Status:** Accepted
- **Start date:** 2026-02
- **Crate:** `tpt-proc-thermo-phase`

## Summary

Define the flash-calculation architecture: specification types (PT, PH, PS,
PVF), the two-phase PT flash as successive substitution on K-values with a
Rachford-Rice inner loop, bubble/dew point calculations, phase-stability
testing (Gibbs tangent-plane criterion, simplified), and single-component
degenerate handling.

## Motivation

Every unit operation in the simulator ultimately reduces to one or more
flash calculations: distillation stages, separators, valve let-downs,
heat exchangers with phase change. The flash layer must be fast (microsecond
class for real-time MPC use), robust at the edges of the two-phase region,
deterministic, and never panic — a non-converged flash must surface as an
error the flowsheet solver can recover from.

## Detailed design

### Specifications

```rust
pub enum FlashSpec {
    PT { temperature: f64, pressure: f64 },
    PH { pressure: f64, enthalpy: f64 },
    PS { pressure: f64, entropy: f64 },
    PVF { pressure: f64, vapor_fraction: f64 },
    TVF { temperature: f64, vapor_fraction: f64 },
}
```

The core algorithm is PT; other specs wrap it in an outer 1-D solve
(brent-style bisection on T or P with the wrapped PT flash as the residual).

### PT flash algorithm

1. **K-value initialization** — Wilson correlation from the database /
   property package: `Kᵢ = (Pcᵢ/P)·exp(5.373·(1 + ωᵢ)·(1 − Tcᵢ/T))`.
2. **Phase stability** — simplified tangent-plane test with the Wilson
   K-values as the trial phase. If stable single-phase, return with
   `vapor_fraction = 0 or 1` (guarded, not exact — single phase either side).
3. **Rachford-Rice inner loop** — Newton-Raphson on

   `g(β) = Σᵢ zᵢ·(Kᵢ−1)/(1+β·(Kᵢ−1)) = 0`

   bracketed to `[β_lo, β_hi]` derived from the sign structure of
   `(Kᵢ−1)`, then accelerated; converges in a handful of iterations
   because `g` is monotone on the bracket.
4. **Composition update** — `xᵢ = zᵢ/(1+β(Kᵢ−1))`, `yᵢ = Kᵢ·xᵢ`.
5. **K-value update** — full property package (fugacity coefficients from
   the EOS/activity model): `Kᵢ = φᵢᴸ/φᵢⱽ` (EOS path) or
   `γᵢ·Psatᵢ·Poynting/(φᵢⱽ·P)` (activity path). Successive substitution
   with damping near the critical region; fall back to a bounded
   Broyden accelerator when substitution stalls.
6. **Convergence** — max |ln Kᵢ⁽ⁿ⁺¹⁾ − ln Kᵢ⁽ⁿ⁾| < 1e-10, iter < 100.
   Return `FlashError::NotConverged` otherwise.

### Degenerate cases (first-class, tested)

- Single component at saturation: β is not unique — return the bracketed
  midpoint and a `degenerate: true` flag rather than a non-convergence.
- All Kᵢ > 1 or all Kᵢ < 1: no two-phase solution exists → single phase.
- Composition containing a zero mole fraction component.
- T or P beyond component criticals → supercritical classification.

### Bubble/dew point

Temperature solve at fixed P (and pressure solve at fixed T) with
`Σ Kᵢ·xᵢ = 1` (bubble) and `Σ yᵢ/Kᵢ = 1` (dew) residuals; Newton with
numerical derivative and 1e-10 tolerance.

## API surface

```rust
pub struct FlashSolver { /* property package */ }
impl FlashSolver {
    pub fn pt_flash(&self, feed: &Composition, t: f64, p: f64)
        -> Result<FlashResult, FlashError>;
    pub fn ph_flash(&self, feed: &Composition, p: f64, h: f64)
        -> Result<FlashResult, FlashError>;
    pub fn bubble_point_t(&self, liquid: &Composition, p: f64) -> Result<f64>;
    pub fn dew_point_t(&self, vapor: &Composition, p: f64) -> Result<f64>;
    pub fn is_two_phase(&self, feed: &Composition, t: f64, p: f64) -> bool;
}
```

## Verification & validation

- Pure water at 373.15 K / 101 325 Pa → β ≈ 0.5 (degenerate flag).
- Binary water/methanol VLE against golden data
  (`flash-binary-vle.json`); Rachford-Rice convergence property test over
  randomized valid K-value sets.
- Bubble/dew of benzene/toluene mixtures vs ideal-K and NRTL paths.
- Mass-balance closure: `zᵢ = (1−β)·xᵢ + β·yᵢ` to machine precision.

## Alternatives considered

- **Gibbs-energy minimization directly (global methods):** the correct
  general answer (handles 3+ phases) but 10–100× slower; successive
  substitution with stability testing is the pragmatic industrial default.
  A Gibbs-minimization module can layer on top later.
- **Inside-out algorithm (Boston-Britt):** superior for wide-boiling and
  near-critical systems; planned as an alternative `FlashStrategy`, same
  API.

## Unresolved questions

- Three-phase VLL flash (e.g., water/hydrocarbon decant) — separate RFC.
- Parallelism across flash calls (flowsheet-level) vs inside one flash.
