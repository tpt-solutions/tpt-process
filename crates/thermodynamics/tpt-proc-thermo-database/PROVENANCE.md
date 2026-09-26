# Database provenance — tpt-proc-thermo-database

Every entry in the built-in chemical database documents where its numbers
came from and how they were verified. This file is the review reference for
the accuracy gate enforced by
`test-data/golden/thermodynamics/database-baseline.txt`: any change to the
database shows up as a baseline diff in CI, and the reviewer checks the
diff against the source/verification claims below before approving.

## Verification policy

- **Scalar properties** (molar mass, critical temperature/pressure/volume,
  acentric factor, normal boiling point) are transcribed from two
  independent tabulations and compared at review time:
  1. DIPPR 801 summary values as reproduced in Poling, Prausnitz &
     O'Connell, *The Properties of Gases and Liquids*, 5th ed. (2001);
  2. NIST WebBook summary values (or Perry's Chemical Engineers' Handbook
     where NIST lists no critical set).
  An entry is only marked *verified* when the two sources agree within the
  rounding shown (Tc ± 0.5 K, Pc ± 1 %, Vc ± 3 %, ω ± 0.005, Tb ± 0.3 K).
- **Ideal-gas heat capacities** appear in two grades:
  - `dippr127` — a fitted DIPPR-127 polynomial (`Cp/R = A + B·T + C·T² +
    D·T³ + E/T²`), expected within a few percent over 250–1000 K;
  - `constant` — a single value frozen at the 298.15 K ideal-gas Cp,
    marked **refit pending**. These are deliberately conservative
    placeholders: use them away from 298 K only for screening, and replace
    with a DIPPR-127 fit (via `ChemicalDatabase::register`) for real work.
- Binary interaction parameters are literature values for Peng-Robinson
  (same Poling tabulation); unlisted pairs default to 0.
- Everything remains **design-estimate grade**. For licensed final design,
  validate against experiment or override entries via
  [`ChemicalDatabase::register`].

## Original 27 components (2025–2026 initial build)

water, methanol, ethanol, acetone, benzene, toluene, styrene, phenol,
methane, ethane, propane, n-butane, n-pentane, n-hexane, n-heptane,
n-octane, ethylene, propylene, nitrogen, oxygen, hydrogen,
carbon-monoxide, carbon-dioxide, hydrogen-sulfide, ammonia, chloroform,
diethyl-ether.

- Source: DIPPR/NIST tabulations as above; ideal-gas Cp fitted to
  Smith–Van Ness–Abbott, *Introduction to Chemical Engineering
  Thermodynamics*, Appendix C tabulations and NIST Shomate-derived
  polynomials, refitted to the DIPPR-127 form over 250–1000 K.
- Verification: two-source transcription cross-check per property
  (policy above). Known soft spots: chloroform Vc and diethyl-ether Cp
  fit are single-source transcriptions pending re-verification.

## Additions (2026-09 review)

Scalar properties: DIPPR 801 (via Poling et al. 5th ed.) cross-checked
against NIST WebBook summary values at transcription time. Cp grade noted
per entry; `constant` values are 298.15 K ideal-gas tabulations (NIST) —
**refit pending**.

| Component | Scalars | Cp grade |
|---|---|---|
| helium | DIPPR/NIST, verified (ω = −0.390 is the DIPPR tabulation) | constant, refit pending |
| argon | DIPPR/NIST, verified (ω = 0) | constant, refit pending |
| sulfur-dioxide | DIPPR/NIST, verified | constant, refit pending |
| acetylene | DIPPR/NIST, verified | constant, refit pending |
| 1,3-butadiene | DIPPR/NIST, Pc/Vc single-source pending second source | constant, refit pending |
| isobutane | DIPPR/NIST, verified | constant, refit pending |
| cyclohexane | DIPPR/NIST, verified | constant, refit pending |
| n-nonane | DIPPR/NIST, verified | constant, refit pending |
| n-decane | DIPPR/NIST, verified | constant, refit pending |
| acetic-acid | DIPPR/NIST, verified (vapor-phase dimerization NOT modeled — PR treats it as a simple fluid) | constant, refit pending |
| ethylene-glycol | DIPPR/NIST, Pc single-source pending second source | constant, refit pending |
| ethylene-oxide | DIPPR/NIST, verified | constant, refit pending |
| dimethyl-ether | DIPPR/NIST, verified | constant, refit pending |
| methyl-tert-butyl-ether | DIPPR/NIST, Pc/Vc single-source pending second source | constant, refit pending |
| acetaldehyde | DIPPR/NIST, verified | constant, refit pending |
| 1-butanol | DIPPR/NIST, verified | constant, refit pending |
| isopropanol | DIPPR/NIST, verified | constant, refit pending |
| ethylbenzene | DIPPR/NIST, verified | constant, refit pending |
| m-xylene | DIPPR/NIST, verified | constant, refit pending |
| p-xylene | DIPPR/NIST, verified | constant, refit pending |
| o-xylene | DIPPR/NIST, verified | constant, refit pending |
| acetonitrile | DIPPR/NIST, verified | constant, refit pending |
| tetrahydrofuran | DIPPR/NIST, verified | constant, refit pending |
| naphthalene | DIPPR/NIST, verified (solid at 298 K; Tb/Tc still valid) | constant, refit pending |

## Adding or changing an entry

1. Transcribe scalars from two independent sources; record disagreement
   in this file rather than silently averaging.
2. Prefer a fitted DIPPR-127 Cp; if only a single-point value is
   available, use `CpCorrelation::constant` and add "refit pending" here.
3. Update the entry in `src/lib.rs` and the relevant table above.
4. Regenerate the baseline
   (`TPT_UPDATE_DB_BASELINE=1 cargo test -p tpt-proc-thermo-database --
   --ignored update_database_baseline`) and let the CI diff surface the
   change for review. Database changes are maintainer-entered: file an
   issue with sources rather than opening an unsolicited PR.
