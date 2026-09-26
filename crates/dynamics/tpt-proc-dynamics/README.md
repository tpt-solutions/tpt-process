# tpt-proc-dynamics

> Fixed-step RK4 integration, FOPDT process models, and step-response analysis.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-dynamics.svg)](https://crates.io/crates/tpt-proc-dynamics)
[![Documentation](https://docs.rs/tpt-proc-dynamics/badge.svg)](https://docs.rs/tpt-proc-dynamics)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The dynamic-simulation building block of the [tpt-process](../../../README.md)
process-engineering suite: a small, dependency-light core you wrap around any plant
model when you need to move it forward in time and characterize how it responds to
input changes.

## Overview

Steady-state design tells you where a plant will sit; dynamics tells you how it gets
there. This crate provides the two primitives that most process dynamic studies reduce
to:

1. **An integrator.** [`rk4`] is the classical fourth-order Runge–Kutta method for
   `dx/dt = f(t, x)`, fixed-step and allocation-light, operating directly on a
   [`IntegratorState`] of `time` plus a state vector. It is the standard workhorse of
   process-dynamics coursework (e.g. Bequette, *Process Dynamics*; Luyben, *Process
   Modeling, Simulation and Control*) and is accurate to O(h⁴) per step.

2. **A process model and an identification step.** [`Fopdt`] is the
   first-order-plus-dead-time model `τ·dy/dt + y = K·u(t − θ)` — the lingua franca of
   controller tuning. Its [`Fopdt::simulate`] integrates the linear ODE *exactly*
   between samples (exponential update, no integration error) and handles dead time by
   input buffering. [`step_response_metrics`] runs the identification in the other
   direction: given a uniformly sampled trajectory it returns the final value and the
   time to 63.2% of the total change, i.e. the time-constant estimate a FOPDT fit
   would produce.

The crate is deliberately minimal: no events, no variable-step solvers, no Jacobians.
It does one thing — move linear-in-the-state models and FOPDT responses forward — and
pairs naturally with [`tpt-proc-control`](../tpt-proc-control) for closed-loop studies.

[`rk4`]: https://docs.rs/tpt-proc-dynamics/latest/tpt_proc_dynamics/fn.rk4.html
[`IntegratorState`]: https://docs.rs/tpt-proc-dynamics/latest/tpt_proc_dynamics/struct.IntegratorState.html
[`Fopdt`]: https://docs.rs/tpt-proc-dynamics/latest/tpt_proc_dynamics/struct.Fopdt.html
[`Fopdt::simulate`]: https://docs.rs/tpt-proc-dynamics/latest/tpt_proc_dynamics/struct.Fopdt.html#method.simulate
[`step_response_metrics`]: https://docs.rs/tpt-proc-dynamics/latest/tpt_proc_dynamics/fn.step_response_metrics.html

## Features

- [`rk4`] — fixed-step classical fourth-order Runge–Kutta integration of `dx/dt = f(t, x)`.
- [`IntegratorState`] — the carried integration state (`time` + state `variables`).
- [`Fopdt`] — first-order-plus-dead-time model with fields `gain`, `time_constant`, `dead_time`.
- [`Fopdt::simulate`] — response to a piecewise-constant input, with exact exponential update and dead-time buffering.
- [`Fopdt::settling_time`] — the 2% settling time (≈ 4·τ, dead time excluded).
- [`step_response_metrics`] — final value and 63.2%-of-change time-constant estimate from a sampled trajectory.
- [`StepResponse`] — the metrics struct returned by the identification.
- `#![forbid(unsafe_code)]` and no dependencies beyond `tpt-proc-core`.

[`Fopdt::settling_time`]: https://docs.rs/tpt-proc-dynamics/latest/tpt_proc_dynamics/struct.Fopdt.html#method.settling_time
[`StepResponse`]: https://docs.rs/tpt-proc-dynamics/latest/tpt_proc_dynamics/struct.StepResponse.html

## Installation

```toml
[dependencies]
tpt-proc-dynamics = "0.1"
```

## Usage

```rust
use tpt_proc_dynamics::{rk4, IntegratorState};

// Exponential decay dx/dt = −x, from x = 1 over 1 s.
let state = IntegratorState { time: 0.0, variables: vec![1.0] };
let final_state = rk4(state, 1.0, 100, |_, x| vec![-x[0]]);
assert!((final_state.variables[0] - (-1.0_f64).exp()).abs() < 1e-9);
```

## API tour

| Item | Purpose |
|---|---|
| [`IntegratorState`] | Time plus state vector passed to and returned from the integrator. |
| [`rk4`] | Integrates `dx/dt = f(t, x)` over `duration` in `steps` fixed steps. |
| [`Fopdt`] | FOPDT model `τ·dy/dt + y = K·u(t − θ)` with gain, time constant, dead time. |
| `Fopdt::simulate` | Output trajectory for a piecewise-constant input; dead time via input buffering. |
| `Fopdt::settling_time` | 2% settling time (≈ 4·τ). |
| [`StepResponse`] | Final value + time-constant estimate of a step response. |
| `step_response_metrics` | Extracts a `StepResponse` from a uniformly sampled trajectory. |

## Scope and limitations

- Fixed-step RK4 only — no adaptive step-size control, stiff-solver (BDF/Radau), or
  event/location handling; stiff kinetics need a different tool.
- No discontinuity handling beyond dead time: the integrator does not stop-and-restart
  at step changes or zero crossings.
- `Fopdt::simulate` quantizes dead time to whole sample intervals (input buffering) and
  assumes the pre-step input was 0.
- State is a plain `Vec<f64>`; there is no sparse or matrix state, no Jacobian
  computation, and no linearization helpers.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types (compile-time dependency).
- [`tpt-proc-control`](../tpt-proc-control) — consumes `Fopdt` in its tests to tune and verify PID controllers in closed loop.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
