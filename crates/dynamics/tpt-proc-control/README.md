# tpt-proc-control

> Ideal-form PID control with anti-windup, filtered derivative, and classical tuning rules.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-control.svg)](https://crates.io/crates/tpt-proc-control)
[![Documentation](https://docs.rs/tpt-proc-control/badge.svg)](https://docs.rs/tpt-proc-control)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A self-contained PID controller and FOPDT-based tuning toolbox for the
[tpt-process](../../../README.md) process-engineering suite: the algorithmic layer that
turns a measured variable and a setpoint into a controller output, sample by sample.

## Overview

The feedback workhorse of process control is the ideal (non-interacting) PID law.
[`Pid`] implements it in discrete form with the two refinements every industrial
implementation needs:

- **Derivative on measurement only.** The derivative term acts on the process
  variable, not the error, avoiding the derivative kick a setpoint step would
  otherwise produce. A first-order filter (N = 10) keeps the differentiated
  measurement from amplifying noise.
- **Clamping anti-windup.** While the output sits at an [`PidConfig::output_limits`]
  clamp in the direction of the error, the integral is frozen, so the controller
  recovers promptly when the error reverses instead of unwinding a saturated
  integral.

For tuning, [`ProcessReactionCurve`] holds a FOPDT identification — process gain K,
time constant τ, dead time θ, exactly the triple that
[`tpt-proc-dynamics`](../tpt-proc-dynamics) models — and applies the two classical
reaction-curve rules: [`ProcessReactionCurve::ziegler_nichols`] open-loop settings and
[`ProcessReactionCurve::cohen_coon`] (the better choice at large θ/τ), each returning
a ready-to-use [`PidConfig`]. These are the rules of Ziegler & Nichols (1942) and
Cohen & Coon (1953) as tabulated in standard references such as Luyben's *Process
Modeling, Simulation and Control* and Ogunnaike & Ray's *Process Dynamics,
Optimization, and Control*.

The crate has no plant model inside it; closed-loop studies combine it with
`tpt-proc-dynamics` (see the crate tests for a worked Ziegler–Nichols loop on a FOPDT
plant).

## Features

- [`Pid`] — discrete ideal-form PID controller with integral, filtered derivative, and output clamp.
- [`Pid::compute`] — one sample per call: `(setpoint, pv, dt)` → output, with derivative-on-measurement and clamping anti-windup.
- [`Pid::reset`] — clears the integral and derivative history.
- [`PidConfig`] — gain Kc, integral time Ti, derivative time Td, optional `output_limits` clamp.
- [`ProcessReactionCurve`] — a FOPDT identification (K, τ, θ) ready for tuning.
- [`ProcessReactionCurve::ziegler_nichols`] — Ziegler–Nichols open-loop PID settings.
- [`ProcessReactionCurve::cohen_coon`] — Cohen–Coon PID settings, better for large dead time.
- `#![forbid(unsafe_code)]`; dev-dependency only on `tpt-proc-dynamics` for closed-loop tests.

## Installation

```toml
[dependencies]
tpt-proc-control = "0.1"
```

## Usage

```rust
use tpt_proc_control::{Pid, PidConfig};

let mut pid = Pid::new(PidConfig {
    gain: 2.0,
    integral_time: 60.0,
    derivative_time: 10.0,
    output_limits: Some((0.0, 100.0)),
});

let output = pid.compute(300.0, 295.0, 1.0); // (setpoint, pv, dt)
assert!((0.0..=100.0).contains(&output));
```

## API tour

| Item | Purpose |
|---|---|
| [`PidConfig`] | Controller parameters: Kc, Ti, Td, output clamp. |
| [`Pid`] | Stateful ideal-form PID with filtered derivative and anti-windup. |
| `Pid::compute` | Produces the output for the current sample. |
| `Pid::reset` | Returns the controller to its initial state. |
| [`ProcessReactionCurve`] | FOPDT identification used as tuning input. |
| `ProcessReactionCurve::ziegler_nichols` | Open-loop ZN tuning → `PidConfig`. |
| `ProcessReactionCurve::cohen_coon` | Cohen–Coon tuning → `PidConfig`. |

## Scope and limitations

- Discrete sampled controller only — no continuous-time state-space synthesis, no
  cascade, ratio, feedforward, or multivariable (MPC) structures.
- Tuning covers open-loop reaction-curve rules; there is no closed-loop (ultimate
  gain/period) Ziegler–Nichols method, no IMC or Lambda tuning, and no relay
  auto-tuning.
- Anti-windup is conditional-integration (clamping) only; no back-calculation or
  tracking schemes.
- No gain scheduling, no bumpless transfer/mode handling, and no I/O scaling —
  engineering units in, engineering units out.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — shared stream and unit types (compile-time dependency).
- [`tpt-proc-dynamics`](../tpt-proc-dynamics) — FOPDT plant simulation; used in the dev-dependencies to verify tuned loops in closed loop, and the natural source of the `ProcessReactionCurve` identification.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
