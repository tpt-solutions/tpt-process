# Water Treatment

`tpt-proc-water` simulates a treatment train: coagulation →
flocculation → sedimentation → filtration → ion exchange → reverse
osmosis → disinfection. Each process transforms a `WaterQuality` state
(turbidity NTU, TDS mg/L, hardness as CaCO₃ mg/L, coliform CFU/mL);
the train applies its processes in declaration order.

```rust
use tpt_proc_water::{Process, Train, WaterQuality};

let raw = WaterQuality::new(25.0, 800.0, 250.0, 10_000.0);
let train = Train::new(vec![
    Process::Coagulation { dose_mg_l: 30.0 },
    Process::Filtration { removal_fraction: 0.95 },
    Process::ReverseOsmosis { rejection: 0.98, recovery: 0.75 },
    Process::Disinfection { ct_value: 40.0, removal_rate: 4.0 },
]);
let out = train.simulate(&raw);
```

## The process models

- `Coagulation { dose_mg_l }` — sweep-floc removal of up to ~70% of
  turbidity per typical dose, with partial coliform removal.
- `Flocculation { retention_time_min }` — additional removal that
  saturates with ~30 min contact time.
- `Sedimentation { removal_fraction }` / `Filtration { removal_fraction }`
  — fractional removals; filtration also takes 90% of the filtered
  coliform load.
- `IonExchange { capacity_mg_l }` — softening up to a finite exchange
  capacity; TDS drops by ~0.85 mg per mg hardness exchanged (Na swaps
  for Ca/Mg).
- `ReverseOsmosis { rejection, recovery }` — permeate TDS = feed·(1−r);
  hardness and coliform ride the salt rejection.
- `Disinfection { ct_value, removal_rate }` — CT concept:
  log removal = k·CT, so a CT of 2 with k = 2 gives 4-log removal
  (10⁶ → 10² CFU/mL).

`apply(process, &quality)` is public, so single units can be evaluated
in isolation (useful for the golden tests).

## Verification

`test-data/golden/applications/water-treatment-train.json` runs a
conventional train plus RO on model raw water (28 NTU, 850 mg/L TDS,
50 000 CFU/mL) and requires product limits of < 1 NTU, < 42.5 mg/L TDS
(5% of feed), and < 1 CFU/mL. Disinfection log-removal is additionally
checked exactly (4-log: 10⁶ → 10² CFU/mL).
