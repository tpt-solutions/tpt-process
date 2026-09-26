# tpt-proc-economics

> Capital cost scaling, cash-flow analysis, NPV/IRR, and levelized cost.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-economics.svg)](https://crates.io/crates/tpt-proc-economics)
[![Documentation](https://docs.rs/tpt-proc-economics/badge.svg)](https://docs.rs/tpt-proc-economics)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The profitability layer of the [tpt-process](../../../README.md) process-engineering
suite: the small set of engineering-economy functions that turn a flowsheet's capacity
and utility numbers into capex estimates, cash flows, and decision metrics.

## Overview

Process design ends with the question every project must answer: is it worth
building? This crate implements the standard screening toolkit from engineering
economy (as taught in Towler & Sinnot's *Chemical Engineering Design* and
Peters, Timmerhaus & West's *Plant Design and Economics*) as pure, dependency-free
functions:

- **Capacity scaling.** [`six_tenths_capex`] applies the six-tenths rule,
  `C2 = C1·(Q2/Q1)^n`, the classic exponential capex scaling used to cost a plant from
  a reference design; the exponent is an argument, not a constant.
- **Operating cost.** [`annual_opex`] is the fixed-plus-variable model,
  `fixed + variable_per_unit × throughput`.
- **Cash-flow measures.** [`npv`] discounts a year-indexed series; [`irr`] solves the
  internal rate of return by bisection on the NPV curve (returning `None` when no sign
  change exists in the searched range); [`discounted_payback`] interpolates the time
  at which the discounted cumulative flow crosses zero. [`cashflows`] builds the
  simple series — an initial investment followed by a constant annual margin — that
  screening studies use.
- **Levelized cost.** [`levelized_cost`] annualizes capex through the capital
  recovery factor (CRF), adds annual opex, and divides by annual production — the
  metric of choice for comparing energy and commodity processes.

Everything is `f64` in, `f64` out, with degenerate inputs (zero capacity, zero life,
no sign change) handled explicitly rather than silently. The crate tests double as
worked examples: a par bond prices to par, a −100/+130 two-period project returns a
30% IRR, and a 500 M plant at 2 Mt/a levelizes near 54 per tonne.

## Features

- [`six_tenths_capex`] — exponential capex scaling, `C2 = C1·(Q2/Q1)^exponent`.
- [`annual_opex`] — fixed plus throughput-proportional operating cost.
- [`cashflows`] — builds an investment-plus-constant-margin cash-flow series.
- [`npv`] — net present value of a year-0-first series at a fractional discount rate.
- [`irr`] — internal rate of return by bisection; `Option` when no sign change exists.
- [`discounted_payback`] — discounted payback time with fractional-year interpolation; `None` if never recovered.
- [`levelized_cost`] — levelized cost of product via the capital recovery factor (CRF).
- Degenerate-input handling (non-positive capacity/life → `NaN`; no IRR sign change → `None`); `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
tpt-proc-economics = "0.1"
```

## Usage

```rust
use tpt_proc_economics::{cashflows, npv, six_tenths_capex};

// Six-tenths rule: doubling capacity raises capex by ~2^0.6 ≈ 1.52.
let capex2 = six_tenths_capex(100.0e6, 1.0, 2.0, 0.6);
assert!((capex2 - 100.0e6 * 2.0_f64.powf(0.6)).abs() < 1e-3);

// A project investing 100 M and returning 25 M/a for 5 years
// (25·3.79 ≈ 94.8 M present value) misses a 10% hurdle.
let flows = cashflows(-100.0e6, 25.0e6, 5);
let value = npv(0.10, &flows);
assert!(value < 0.0);
```

## API tour

| Item | Purpose |
|---|---|
| [`six_tenths_capex`] | Scale capex with capacity by the six-tenths rule. |
| [`annual_opex`] | Fixed + variable annual operating cost. |
| [`cashflows`] | Simple investment-then-margin cash-flow series. |
| [`npv`] | Discount a cash-flow series to present value. |
| [`irr`] | Solve the internal rate of return (bisection). |
| [`discounted_payback`] | Time to recover the investment, discounted. |
| [`levelized_cost`] | (Annualized capex + opex) / annual production via CRF. |

## Scope and limitations

- Screening-grade models only: no escalation/inflation, taxes, depreciation
  schedules, working capital, financing structure, or location factors; the six-tenths
  rule is a single-exponent scaling, not a detailed module estimate.
- Annual periods, end-of-year convention, and a constant annual margin in
  [`cashflows`]; irregular or mid-year flows must be assembled manually into the slice
  you pass to [`npv`].
- [`irr`] searches the fixed rate range −0.9 to 10 and finds one root; non-conventional
  cash-flow sign patterns (multiple IRRs) are not detected.
- No currency handling (values are unit-agnostic numbers) and no uncertainty or
  sensitivity analysis helpers.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types (compile-time dependency).
- [`tpt-proc-database`](../tpt-proc-database) — a natural store for the resulting cost metrics.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
