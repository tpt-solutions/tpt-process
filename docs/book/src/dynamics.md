# Dynamics & Control

Two crates cover unsteady behavior: `tpt-proc-dynamics` (integration
and process-response models) and `tpt-proc-control` (PID with
classical tuning rules).

## Simulation (`tpt-proc-dynamics`)

`rk4` integrates dx/dt = f(t, x) at fixed step and returns the final
`IntegratorState`:

```rust
use tpt_proc_dynamics::{rk4, IntegratorState};

// Exponential decay dx/dt = −x, from x = 1 over 1 s.
let state = IntegratorState { time: 0.0, variables: vec![1.0] };
let final_state = rk4(state, 1.0, 100, |_, x| vec![-x[0]]);
assert!((final_state.variables[0] - (-1.0_f64).exp()).abs() < 1e-9);
```

`Fopdt` is the first-order-plus-dead-time workhorse
τ·dy/dt + y = K·u(t − θ). `simulate(u, y0, duration, samples)` steps
the exact exponential update of the linear ODE with the dead time
handled by an input history buffer; `settling_time()` is 4τ (dead time
excluded). `step_response_metrics(&samples, initial, dt)` reads a
trajectory back: the final value and the time to 63.2% of the change —
recovered τ within 0.5 s on a τ = 8 s test.

## Control (`tpt-proc-control`)

`Pid` is an ideal-form controller with derivative on the measurement
only (no derivative kick), a first-order filtered derivative (N = 10),
and clamping anti-windup — the integral freezes while the output is
saturated in the direction of the error.

```rust
use tpt_proc_control::{Pid, PidConfig};

let mut pid = Pid::new(PidConfig {
    gain: 2.0,
    integral_time: 60.0,
    derivative_time: 10.0,
    output_limits: Some((0.0, 100.0)),
});

let output = pid.compute(300.0, 295.0, 1.0); // (setpoint, pv, dt)
```

## Tuning from a process-reaction curve

Identify a FOPDT model (or take it from `Fopdt`), then apply the
classical rules. Both return a ready `PidConfig`:

```rust
use tpt_proc_control::ProcessReactionCurve;

let curve = ProcessReactionCurve { gain: 0.8, time_constant: 20.0, dead_time: 2.0 };
let zn = curve.ziegler_nichols();
let cc = curve.cohen_coon(); // more robust for large θ/τ
```

The integration test drives a FOPDT plant (K = 0.8, τ = 20 s,
θ = 2 s) with Ziegler–Nichols settings to within 0.5 of setpoint over
2000 samples at dt = 0.5 — controller and plant crates are designed to
compose.
