# tpt-proc-optimization

> Golden-section, Nelder-Mead, and gradient-descent optimizers. Zero dependencies.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-optimization.svg)](https://crates.io/crates/tpt-proc-optimization)
[![Documentation](https://docs.rs/tpt-proc-optimization/badge.svg)](https://docs.rs/tpt-proc-optimization)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The numerical-optimization toolbox of the [tpt-process](../../../README.md)
process-engineering suite: three classic unconstrained minimizers as small,
dependency-free functions over `f64` closures.

## Overview

Process engineering work keeps needing a minimum: a pump operating point, a
fitted correlation coefficient, a recycle tear variable, a cost curve knee.
This crate ships the three workhorse algorithms, each as a single pure
function, with no dependencies at all.

- [`golden_section`] minimizes a **unimodal** scalar function on a bracket
  `[a, b]` using the golden ratio 1/φ ≈ 0.618 to shrink the interval; each
  evaluation reuses the previous interior point, and bounds may be passed in
  either order. Returns `(x*, f(x*))`.
- [`nelder_mead`] minimizes an n-dimensional function **without gradients**
  using a simplex that reflects (coefficient 1), expands (2), contracts
  (0.5), and shrinks toward the best vertex. The initial simplex offsets each
  coordinate by ~5% of its value (1e-2 when the coordinate is zero), and the
  loop stops when the best–worst value spread falls below `tolerance` or
  `max_iterations` is reached. Returns `(best point, f(best))`.
- [`gradient_descent`] minimizes a function given its **analytic gradient**
  and a fixed step; it iterates `x ← x − step·∇f(x)` until the update is
  numerically stationary or `iterations` is exhausted, and returns the final
  point.

All three are `#[must_use]`, closure-based, and `#![forbid(unsafe_code)]`.

## Features

- [`golden_section`] — 1-D bracketed minimization for unimodal functions;
  returns `(x*, f(x*))`.
- [`nelder_mead`] — derivative-free n-dimensional minimization with the
  standard reflect/expand/contract/shrink simplex moves.
- [`gradient_descent`] — fixed-step descent from a user-supplied gradient
  closure, in any dimension.
- Zero dependencies: the crate has an empty `[dependencies]` table and
  compiles anywhere the standard library does.
- Pure functions — no builders, no configuration structs, no state.
- Deterministic: no randomness in any initial simplex or iteration.
- Tested against known optima (quadratic brackets, Rosenbrock, gradient
  bowls).
- `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
tpt-proc-optimization = "0.1"
```

## Usage

```rust
use tpt_proc_optimization::{golden_section, nelder_mead};

// Minimize (x − 2)² on [0, 5].
let (x, fx) = golden_section(|x| (x - 2.0).powi(2), 0.0, 5.0, 1e-9);
assert!((x - 2.0).abs() < 1e-6);

// Minimize the 2-D Rosenbrock-style bowl.
let start = [3.0, 3.0];
let (point, fx) = nelder_mead(|p| p[0].powi(2) + 10.0 * p[1].powi(2), &start, 1e-10, 2000);
assert!(fx < 1e-8);
```

## API tour

| Item | Purpose |
|---|---|
| [`golden_section`] | 1-D unimodal minimization on a bracket; returns `(x*, f(x*))`. |
| [`nelder_mead`] | Derivative-free n-D minimization; returns `(point, f(point))`. |
| [`gradient_descent`] | Fixed-step descent with an analytic gradient; returns the final point. |

## Scope and limitations

- **Unconstrained minimization only**: no bounds, no linear/nonlinear
  constraints, no penalty or barrier methods; to maximize, minimize the
  negated objective.
- [`golden_section`] requires a unimodal objective and a bracket containing
  the minimum; it will stop at a local flat region otherwise.
- [`nelder_mead`] is a local method — no global search, no restarts, no
  multi-start — and its fixed initial simplex perturbation (~5% of each
  coordinate) makes it scale-sensitive to badly conditioned starts.
- [`gradient_descent`] uses a fixed step with no line search or momentum, so
  step size is the user's responsibility (too large diverges, too small
  crawls).
- `f64` only; the objective closures must be deterministic (NaN/inf behavior
  is not sanitized).

## Related crates

- No dependencies: `tpt-proc-optimization` is the only zero-dependency crate
  in the suite and can be used entirely standalone.
- [`tpt-proc-flowsheet`](../tpt-proc-flowsheet) — flowsheet convergence and
  tear-stream studies are the natural consumer of these solvers (not wired
  together yet).
- [`tpt-proc-economics`](../../data/tpt-proc-economics) — cash-flow/NPV
  objectives for design optimization are a typical use (also not wired yet).
- No suite crate currently consumes `tpt-proc-optimization`.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
