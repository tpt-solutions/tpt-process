# Process Economics

`tpt-proc-economics` is the screening toolkit: capital-cost scaling,
operating cost, and cash-flow analysis (NPV, discounted payback, IRR,
levelized cost). All money values are in whatever currency unit you
pass — the functions are unit-agnostic.

## Capital and operating cost

`six_tenths_capex(c1, q1, q2, exponent)` scales a reference capital
cost with capacity: C₂ = C₁·(Q₂/Q₁)^n. Doubling capacity at n = 0.6
costs 1.52×. `annual_opex(fixed, variable_per_unit, throughput)` adds
the proportional term.

## Cash flows

`cashflows(investment, annual_margin, years)` builds the standard
series (outflow at year 0, constant margin after). `npv(rate, &flows)`
discounts it; `irr(&flows)` bisects the NPV curve for the rate where it
vanishes (`None` when no sign change exists);
`discounted_payback(rate, &flows)` interpolates the recovery year
(`None` if never recovered).

```rust
use tpt_proc_economics::{cashflows, irr, npv};

// -100 now, +130 next year → IRR = 30%.
let r = irr(&[-100.0, 130.0]).unwrap();
assert!((r - 0.30).abs() < 1e-9);

// 100 M invested, 25 M/a for 5 years misses a 10% hurdle.
let flows = cashflows(-100.0e6, 25.0e6, 5);
assert!(npv(0.10, &flows) < 0.0);
```

## Levelized cost

`levelized_cost(capex, annual_opex, annual_production, rate, life)`
annualizes capital through the capital recovery factor
CRF = r/(1 − (1+r)^(−N)) and divides by production. A 500 M plant with
50 M/a opex, 2 Mt/a output, 10% over 20 years lands at ~54 per tonne
(CRF ≈ 0.1175).

A standard check in the test suite: a par bond (−1000, then 5 × 100
coupons and 1000 principal) has NPV = 0 at a 10% discount.
