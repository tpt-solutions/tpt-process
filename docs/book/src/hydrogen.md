# Hydrogen Production

`tpt-proc-hydrogen` is pre-design energy accounting for hydrogen
routes: water electrolysis, steam methane reforming (SMR),
autothermal reforming (ATR), and the water-gas shift. It answers
screening questions — kWh per kg, CO₂ per kg, plant size — not detailed
reactor design.

## Electrolysis

`Electrolyzer::new` picks a technology's typical efficiency:
`Alkaline` (η = 0.67), `Pem` (0.65), `SolidOxide` (0.90, high
temperature with heat input).

```rust
use tpt_proc_hydrogen::{Electrolyzer, ElectrolyzerType};

let pem = Electrolyzer::new(ElectrolyzerType::Pem);
let sec = pem.specific_energy_consumption(); // kWh/kg H2
assert!((47.0..=60.0).contains(&sec));

// A 100 MW plant produces roughly 2 t/h.
let rate = pem.production_rate(100.0e6); // kg/s
assert!(rate > 0.5 && rate < 0.7);
```

SEC = LHV/η/3.6 with the exported `HYDROGEN_LHV_MJ_KG` = 120
(`HYDROGEN_HHV_MJ_KG` = 141.8). `water_consumption(margin)` returns the
stoichiometric 8.936 kg water per kg H₂ inflated by a purification
margin.

## Reformer routes

`Smr { thermal_efficiency }` and `Atr { overall_efficiency }` convert a
fuel-energy input to a hydrogen rate and report intensities per kg H₂:

- `fuel_per_kg_h2()` — natural gas input on an LHV basis.
- `Smr::co2_per_kg_h2()` — 5.5 kg/kg process CO₂ (CH₄ + 2H₂O → CO₂ +
  4H₂) plus 0.055 kg/MJ for the fired fuel: ≈ 8–9 kg CO₂ per kg H₂
  unabated at typical efficiencies (the tests bound 7.5–11).
- `Atr::oxygen_per_kg_h2()` — the partial-oxidation oxygen demand
  (~0.35/η kg O₂/kg H₂).

```rust
use tpt_proc_hydrogen::Smr;

let smr = Smr { thermal_efficiency: 0.80 };
let co2 = smr.co2_per_kg_h2(); // ≈ 8.3 kg CO2 per kg H2
```

`water_gas_shift_conversion(temperature_k)` gives the per-stage CO
conversion approach for shift sizing: ~0.97 at low temperature, falling
to a clamped floor of 0.55 as temperature rises.
