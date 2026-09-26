# example-project-economics

Pre-feasibility economics of a 150 kt/a plant. The capital cost comes
from six-tenths-rule scaling of a 100 kt/a reference plant; the operating
cost combines a fixed block with a variable term per tonne. The annual
margin becomes a 15-year cash-flow series evaluated three standard ways —
net present value at a 10% hurdle rate, internal rate of return, and
discounted payback — with an NPV sensitivity at 8% and 12%, and the
project is summarized by its levelized cost of product against the 220
$/t selling price.

## Run

From the repository root:

```sh
cargo run --release -p example-project-economics
```

## What it shows

- Capacity scaling: `six_tenths_capex(capex_ref, capacity_ref, capacity,
  exponent)`.
- Operating cost: `annual_opex(fixed, variable_per_unit, throughput)`.
- Cash-flow construction with `cashflows(investment, annual_margin, years)`.
- Investment metrics: `npv(rate, &flows)`, `irr(&flows)` (bisection),
  and `discounted_payback(rate, &flows)`.
- `levelized_cost(capex, opex, production, rate, life)` via the capital
  recovery factor.

## Crates used

- [tpt-proc-economics](../../crates/data/tpt-proc-economics)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
