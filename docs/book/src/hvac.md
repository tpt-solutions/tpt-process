# HVAC & Psychrometrics

`tpt-proc-hvac` computes moist-air states at a fixed total (barometric)
pressure: humidity ratios, enthalpy per kg of dry air, specific volume,
dew point, wet bulb, and adiabatic mixing. Dry air and water vapor are
treated as ideal gases with ASHRAE-style correlations; temperatures are
K internally and the API takes and returns SI (Pa, J/kg, kg/kg).

```rust
use tpt_proc_hvac::Psychrometrics;

let ps = Psychrometrics::new(101_325.0);

// Saturation pressure of water at 20 °C ≈ 2.34 kPa.
let p_sat = ps.saturation_pressure(293.15);

// 20 °C, 50% RH → humidity ratio ≈ 0.00726 kg/kg.
let w = ps.humidity_ratio_from_rh(293.15, 0.5);

// Moist-air enthalpy ≈ 38.6 kJ/kg dry air.
let h = ps.enthalpy(293.15, w);
```

## The correlation set

- `saturation_pressure(t)` — ASHRAE/Magnus-style form, valid 273–373 K
  within ~0.1%.
- `humidity_ratio_from_rh` / `relative_humidity` — exact inverses via
  W = 0.62198·φ·p_sat/(p − φ·p_sat).
- `enthalpy(t, w)` — h = 1.006·T[°C] + W·(2501 + 1.86·T[°C]) kJ/kg dry
  air, the standard 0 °C reference.
- `dew_point` — bisection on the inverted saturation correlation.
- `wet_bulb` — thermodynamic wet bulb by the adiabatic-saturation
  approximation (always between the dew point and the dry bulb).
- `mixing` — two-stream adiabatic mix on a dry-air mass basis; equal
  flows give linear averages of dry-bulb temperature and humidity ratio.

Physical constants are exported for downstream work: `CP_DRY_AIR`
(1006 J/(kg·K)), `CP_VAPOR` (1860), `H_VAPORIZATION` (2.501 MJ/kg at
0 °C), and `STANDARD_ATMOSPHERE` (101 325 Pa).

## Verification

Golden data in `test-data/golden/applications/psychrometrics-cooling.json`
pins the 20 °C / 50% RH state: W = 0.00726 kg/kg (±2e-4) and dew point
282.4 K (±1.5 K). See [Validation Results](validation.md).
