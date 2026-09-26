# Validation Results

Solver outputs are checked against golden reference files in
`test-data/golden/` (thermodynamics, fluid-flow, heat-transfer,
separations, reactors, applications). Every file states its tolerances;
where a physical reference source is cited — NIST WebBook and DIPPR
correlations for vapor pressure, ASHRAE-style psychrometric
correlations — it is stated in the file itself. Most cases are
analytical identities (closed-form solutions the solver must reproduce
exactly), which is deliberate: a number that can be derived exactly
should not be "validated" against another approximation.

## Thermodynamics

| Case | Expected | Tolerance |
|---|---|---|
| PR vapor pressure: water at 373.15 K | 101 325 Pa | 5% |
| PR vapor pressure: benzene at 353.25 K, toluene at 383.75 K, propane at 231.04 K, methane at 111.67 K | 101 325 Pa | 5% |
| Pure bubble/dew point at 1 atm: benzene / toluene | 353.25 K / 383.75 K | 3 K |
| Binary flash, 50/50 water/methanol at 350 K, 1 atm | β = 0.7889; x = [0.9698, 0.0302]; y = [0.3743, 0.6257] | 1e-6 |

The vapor-pressure file cites its source: NIST WebBook / DIPPR
correlations. NRTL activity coefficients are guarded by range checks
plus Gibbs-Duhem residuals < 1e-4 as the primary consistency check.

## Fluid flow

| Case | Expected | Tolerance |
|---|---|---|
| Laminar Darcy-Weisbach vs Hagen-Poiseuille | exact agreement | 1e-9 |
| Turbulent friction at Re = 1e5, smooth | f = 0.018 (Colebrook-White) | 0.002 |
| Pump vs system curve crossing (quadratic curves) | Q = 0.0353553 m³/s, H = 20 m | 1e-10 |
| Two-loop Hardy-Cross network | continuity < 1e-4, loop sums < 1e-8, all three solvers (Hardy Cross, Newton-Raphson, Linear Theory) agree < 2% | per check |

## Heat transfer

| Case | Expected | Tolerance |
|---|---|---|
| Counter-current LMTD (ΔT = 50 K, 10 K) | 24.8546 K | 1e-6 |
| ε-NTU at C_r = 1 (NTU = 0.1) | ε = NTU/(1+NTU) = 0.0909091 | 1e-9 |
| Pinch analysis at ΔT_min = 10 K | Q_H,min = 0 kW, Q_C,min = 2925 kW (threshold problem, no pinch) | first-law identity |

## Separations

| Case | Expected | Tolerance |
|---|---|---|
| FUG shortcut (α = 2.4, 95/95 split, R = 1.5) | N_min = 6.7294, R_min = 1.1857, N = 15.4 | 1e-6 |
| McCabe-Thiele on the same binary | 14 stages, feed at stage 7 | ±1 stage |
| Kremser absorption (N = 1, A = 2; N = 3, A = 1) | fraction remaining 1/3 and 0.25 (closed form) | 1e-12 |

The FUG and McCabe-Thiele files describe the same separation, so the
shortcut and graphical methods are cross-checked against each other.

## Reactors

| Case | Expected | Tolerance |
|---|---|---|
| First-order CSTR, kτ = 1 | X = kτ/(1+kτ) = 0.5 | 1e-6 |
| First-order PFR, kτ = 1 | X = 1 − e^(−kτ) = 0.63212 | 1e-4 |
| Arrhenius temperature sensitivity, E_a = 54.5 kJ/mol | rate doubles 300 → 310 K | 5% |

## Applications

| Case | Expected | Tolerance |
|---|---|---|
| Psychrometrics at 1 atm (ASHRAE-style correlations): 20 °C, 50% RH | W = 0.00726 kg/kg, T_dp = 282.4 K | 2e-4 / 1.5 K |
| Water treatment train: conventional + RO on model raw water | product < 1 NTU, < 42.5 mg/L TDS, < 1 CFU/mL | limit check |

## Policy

The verification & validation rules — analytical limits per
correlation, stated sources and tolerances for every golden
comparison, convergence tests that report non-convergence honestly —
are defined in [CONTRIBUTING.md](https://github.com/tpt-solutions/tpt-process/blob/master/CONTRIBUTING.md).
Browse the raw data in [`test-data/golden/`](https://github.com/tpt-solutions/tpt-process/tree/master/test-data/golden).
