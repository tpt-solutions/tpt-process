# example-membrane-separation

Two membrane services in one example. Gas: a 2 µm CO2-selective polymer
film (CO2 = 30 Barrer, CH4 = 5 Barrer) with solution-diffusion flux
J = P·Δp/ℓ across a 1.1 MPa partial-pressure drop, ideal CO2/CH4
selectivity, permeance in GPU, and the daily CO2 yield per m² of
membrane. Liquid: seawater reverse osmosis — van 't Hoff osmotic
pressure, water flux J_w = A·(ΔP − Δπ) with the driving force clamped at
zero below the osmotic pressure, and salt rejection R = 1 − cp/cf.

## Run

From the repository root:

```sh
cargo run --release -p example-membrane-separation
```

## What it shows

- `Membrane::new(thickness, permeability_barrer)` and the gas flux
  `Membrane::gas_flux(p_feed, p_perm, P_barrer)` in mol/(m²·s).
- `Membrane::selectivity(P_fast, P_slow)` and `permeance_gpu(P_barrer)`.
- RO thermodynamics: `Membrane::osmotic_pressure(c, i, T)` and liquid
  flux `Membrane::liquid_flux(A, delta_p, delta_pi)` (no flux below Δπ).
- Salt rejection via `Membrane::rejection(c_feed, c_permeate)`.

## Crates used

- [tpt-proc-membranes](../../crates/separations/tpt-proc-membranes)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
