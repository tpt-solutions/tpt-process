# example-pump-pipeline

Fluid machinery day one: (1) find the duty point of a centrifugal pump
lifting water through a 200 m pipeline — where the quadratic pump curve
meets the static-plus-friction system curve — with efficiency, shaft
power, and NPSH; (2) size the control valve on that line with a Cv check
and compare linear vs. equal-percentage characteristics; (3) evaluate a
methane compressor stage (1 → 4 bar) for isentropic work, discharge
temperature, shaft power, and polytropic head. One binary, three sections.

## Run

From the repository root:

```sh
cargo run --release -p example-pump-pipeline
```

## What it shows

- Deriving the friction coefficient of the line from
  `PipeFlow::pressure_drop_at_flow` (`tpt-proc-fluid`) and wrapping it in
  a `SystemCurve { static_head, k }`.
- The operating point by bisection: `Pump::operating_point(&system, q_max)`
  over a `PumpCurve`, plus `efficiency_at_flow`, `power_at_flow`, and
  `Pump::npsh_available`.
- Valve sizing (`tpt-proc-valves`): `ControlValve::required_cv_liquid`,
  `liquid_flow_m3s`, `cv_at` for `Linear` and `EqualPercentage`
  characteristics, and the `cv_to_kv` conversion.
- Compression (`tpt-proc-compressors`): `CompressorService::evaluate(rs)`
  giving `head_isentropic`, `work_actual`, `discharge_temperature`, and
  `pressure_ratio`; `power(mdot, rs)`; `polytropic_head(rs, eta_p)`.

## Crates used

- [tpt-proc-fluid](../../crates/fluid-flow/tpt-proc-fluid)
- [tpt-proc-pumps](../../crates/fluid-flow/tpt-proc-pumps)
- [tpt-proc-valves](../../crates/fluid-flow/tpt-proc-valves)
- [tpt-proc-compressors](../../crates/fluid-flow/tpt-proc-compressors)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
