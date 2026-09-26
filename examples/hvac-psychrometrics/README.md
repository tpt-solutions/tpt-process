# example-hvac-psychrometrics

Psychrometrics of moist air at a fixed barometric pressure: reading the
state of outdoor and room air (humidity ratio, dew point, wet bulb),
mixing two airstreams, and sizing the cooling-coil duty needed to deliver
the supply air condition of a small all-air system. This is the daily
calculation behind air-handling-unit selection.

## Run

From the repository root:

```sh
cargo run --release -p example-hvac-psychrometrics
```

## What it shows

- Creating a `Psychrometrics::new(101_325.0)` calculator at sea-level
  pressure.
- State points from temperature and relative humidity:
  `humidity_ratio_from_rh`, with `dew_point` and `wet_bulb` lookups.
- Adiabatic-ish stream mixing of 1 kg/s outdoor air with 3 kg/s room air
  via `Psychrometrics::mixing(...)`.
- Moist-air enthalpy `Psychrometrics::enthalpy(t, w)` and the coil duty
  (kJ/kg to kW) required to reach a 13 °C / 95% RH supply state.

## Crates used

- [tpt-proc-hvac](../../crates/applications/tpt-proc-hvac)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
