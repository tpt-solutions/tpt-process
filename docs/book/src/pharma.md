# Pharmaceutical Batch Processes

`tpt-proc-pharma` covers three batch-manufacturing calculations: cycle
accounting, dissolution kinetics, and stability/shelf-life estimation.

## Batch cycle accounting

`BatchCycle` accumulates named steps and reports the total duration and
utilization against a plant time budget.

```rust
use tpt_proc_pharma::BatchCycle;

let mut cycle = BatchCycle::new(48.0);
let total = cycle
    .add_step("charge", 2.0)
    .add_step("react", 24.0)
    .add_step("discharge", 4.0)
    .total_duration();               // 30 h
let utilization = cycle.utilization(48.0); // 30/48
```

## Dissolution

`dissolution_fraction(k, t)` integrates the Noyes–Whitney first-order
law: f = 1 − exp(−k·t), with k in h⁻¹. A tablet with k = 0.35 h⁻¹
reaches ~95% dissolved at 8.5 h.

## Stability and shelf life

`potency_after_storage(k_ref, reference_k, temperature_k, days, ea)`
applies Arrhenius acceleration of the first-order degradation rate
around a reference condition — potency falls faster warm, slower cold.
`shelf_life_days(k_ref)` returns the time to the USP 95%-potency limit
under reference conditions: t₉₅ = ln(1/0.95)/k, so k = 1e-4/day gives
~513 days (> 2 years).

```rust
use tpt_proc_pharma::{potency_after_storage, shelf_life_days};

// One year in a 40 °C warehouse vs 25 °C reference, Ea = 75 kJ/mol.
let remaining = potency_after_storage(0.001, 298.15, 313.15, 365.0, 75_000.0);
let t95 = shelf_life_days(1.0e-4); // ≈ 513 days
```

These are screening models for schedule and stability budgets; formal
ICH stability studies remain the regulatory authority.
