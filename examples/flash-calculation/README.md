# example-flash-calculation

An isothermal (PT) flash is the workhorse separator calculation of process
simulation: a feed of known composition is brought to a given temperature
and pressure, and the solver must find the vapor fraction and the vapor and
liquid compositions at vapor-liquid equilibrium. This example flashes a
50/50 water/methanol feed at 350 K and 1 atm using a Peng-Robinson cubic
equation of state built from the suite's built-in chemical database, prints
the vapor fraction and both outlet compositions, and then flashes pure
water at its normal boiling point to demonstrate the degenerate (single
phase) flag.

## Run

From the repository root:

```sh
cargo run --release -p example-flash-calculation
```

## What it shows

- Building a property package from the built-in database:
  `ChemicalDatabase::builtin()` and `components_for(&["water", "methanol"])`.
- Attaching an equation of state: `CubicEos::peng_robinson(...)` and
  `PropertyPackage::with_eos(...)`.
- Constructing a feed with `Composition::from_mole_fractions(&[0.5, 0.5])`.
- Solving the PT flash with `FlashSolver::pt_flash(&feed, 350.0, 101_325.0)`
  and reading `vapor_fraction`, `liquid_composition`, `vapor_composition`,
  and `iterations` off the result.
- Detecting the degenerate pure-component case via the `degenerate` flag.
- Inspecting a pure component's critical constants
  (`critical_temperature`, `critical_pressure`, `critical_compressibility`).

## Crates used

- [tpt-proc-core](../../crates/core/tpt-proc-core)
- [tpt-proc-thermo-core](../../crates/thermodynamics/tpt-proc-thermo-core)
- [tpt-proc-thermo-database](../../crates/thermodynamics/tpt-proc-thermo-database)
- [tpt-proc-thermo-eos](../../crates/thermodynamics/tpt-proc-thermo-eos)
- [tpt-proc-thermo-phase](../../crates/thermodynamics/tpt-proc-thermo-phase)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
