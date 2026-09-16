# Verification & Validation

Engineering software needs evidence:

- Analytical limits (Hagen-Poiseuille, ideal-gas EOS limit, analytic CSTR/
  PFR conversions, Gilliland X = 1).
- Golden files under `test-data/golden/` with stated tolerances and
  sources (DIPPR, NIST, ASHRAE).
- Thermodynamic consistency checks: Gibbs-Duhem residuals < 1e-4, mass
  balance closure to machine precision, first-law identities in pinch.
- Cross-method agreement: three network solvers, FUG vs McCabe-Thiele.
