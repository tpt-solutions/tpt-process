# example-crystallizer

Crystallization on two fronts. Yield: cooling a 1000 kg KNO3 solution
from 80 °C (solubility 0.51 kg/kg solvent) to 20 °C (0.15 kg/kg) drops
238 kg of crystals — computed from the solubility curve on a
solvent basis. Size: an MSMPR (mixed suspension, mixed product removal)
crystallizer treated with the classic population-balance results — an
empirical nucleation law B0 = kB·G²·MT and the dominant crystal size
L_D = 3.67·G·τ, including the linear coarsening when residence time
doubles.

## Run

From the repository root:

```sh
cargo run --release -p example-crystallizer
```

## What it shows

- Cooling yield from the solubility curve: `cooling_yield(solution_kg,
  c_initial, c_final)` and `crystal_mass_kg(solvent_kg, c_i, c_f)`.
- The mother-liquor bookkeeping that closes the mass balance.
- `Crystallizer::new(volume_m3)` with `msmpr_nucleation(G, kB, MT)`
  (crystals per m³ per second).
- Dominant crystal size `msmpr_dominant_size(G, tau)` and its linear
  scaling with residence time.

## Crates used

- [tpt-proc-crystallization](../../crates/separations/tpt-proc-crystallization)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
