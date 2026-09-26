# Refining

`tpt-proc-refining` characterizes refinery streams: API gravity,
Watson characterization factor, and boiling-point cut blending over a
simplified crude assay.

## Gravity and character

```rust
use tpt_proc_refining::{api_to_specific_gravity, watson_k_factor};

// A 40° API light stream.
let sg = api_to_specific_gravity(40.0);
assert!((sg - 0.8251).abs() < 1e-3);

// Watson characterization factor from MeABP (avg boiling point).
let k = watson_k_factor(650.0, sg); // °R basis
assert!((10.0..=13.0).contains(&k));
```

`api_to_specific_gravity` and `specific_gravity_to_api` are the
standard 141.5/(API + 131.5) pair (water is 10° API). `watson_k_factor`
takes the mean average boiling point in degrees Rankine:
K = Tb_R^(1/3)/sg — K ≈ 12.5+ is paraffinic, 11–12 naphthenic, ≈ 10.5
aromatic. (The familiar 1.216 constant belongs to the Kelvin form.)

## Crude assays

A `CrudeAssay` is a named list of `Cut`s — boiling ranges in °F with a
mass fraction that should sum to ~1:

- `total_yield()` — normalization check.
- `weighted_api(&[f64])` — mass-weighted crude gravity from per-cut
  gravities (missing entries default to 35° API).
- `yield_below(max_final_bp_f)` — pooled yield of everything at or
  below a final boiling point, e.g. the naphtha/gasoline pool.

`light_crude_assay()` ships an illustrative ~40° API light-crude shape
(butanes-plus through vacuum resid, seven cuts) used by the tests: its
naphtha pool (cuts ending at or below 380 °F) is exactly 0.33 by mass.
