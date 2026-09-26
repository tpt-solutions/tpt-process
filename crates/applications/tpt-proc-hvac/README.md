# tpt-proc-hvac

> Psychrometrics for moist air: humidity, enthalpy, dew point, wet bulb, mixing.

[![Crates.io](https://img.shields.io/crates/v/tpt-proc-hvac.svg)](https://crates.io/crates/tpt-proc-hvac)
[![Documentation](https://docs.rs/tpt-proc-hvac/badge.svg)](https://docs.rs/tpt-proc-hvac)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

The air-side HVAC crate of the [tpt-process](../../../README.md)
process-engineering suite: psychrometric state computations for moist air at
near-atmospheric pressures, ASHRAE-style correlations, all-SI API.

## Overview

[`Psychrometrics`] is a small stateless engine constructed at a fixed total
(barometric) pressure. From a dry-bulb temperature and a humidity ratio (or a
relative humidity) it computes the quantities an air-side design needs:

- **Saturation pressure** from the ASHRAE/Magnus-style correlation
  (valid 273–373 K within ~0.1%), used by every other property.
- **Humidity ratio** W = 0.62198·φ·p_sat/(p − φ·p_sat) from relative
  humidity, and the inverse [`Psychrometrics::relative_humidity`].
- **Specific enthalpy** per kg of dry air on the familiar 0 °C reference,
  h = 1.006·T[°C] + W·(2501 + 1.86·T[°C]) kJ/kg.
- **Specific volume** of the moist mixture per kg dry air,
  v = R_air·T·(1 + 1.6078·W)/p.
- **Dew point** by bisection-inverting the saturation correlation, and the
  **thermodynamic wet-bulb** temperature via the adiabatic saturation
  approximation (enthalpy matching with the liquid make-up term).
- **Adiabatic mixing** of two moist-air streams on a dry-air mass basis.

Temperatures are in kelvin at the API boundary; pressures in Pa, enthalpies
in J/kg dry air, humidity ratios in kg water/kg dry air. Physical constants
([`STANDARD_ATMOSPHERE`], [`CP_DRY_AIR`], [`CP_VAPOR`],
[`H_VAPORIZATION`]) are exported. Dry air and water vapor are treated as
ideal gases; the crate is `#![forbid(unsafe_code)]`.

## Features

- [`Psychrometrics`] — the psychrometric engine at a fixed total pressure
  (`Psychrometrics::new` is `const`).
- [`Psychrometrics::saturation_pressure`] — water saturation pressure, Pa
  (ASHRAE/Magnus correlation, 273–373 K).
- [`Psychrometrics::humidity_ratio_from_rh`] /
  [`Psychrometrics::relative_humidity`] — W ↔ φ conversions.
- [`Psychrometrics::enthalpy`] — moist-air specific enthalpy, J/kg dry air.
- [`Psychrometrics::specific_volume`] — m³/kg dry air.
- [`Psychrometrics::dew_point`] — dew-point temperature by correlation
  inversion (bisection).
- [`Psychrometrics::wet_bulb`] — thermodynamic wet-bulb via adiabatic
  saturation.
- [`Psychrometrics::mixing`] — adiabatic mixing of two moist-air streams,
  returning the mixed (dry-bulb, humidity ratio).
- [`STANDARD_ATMOSPHERE`], [`CP_DRY_AIR`], [`CP_VAPOR`],
  [`H_VAPORIZATION`] — exported physical constants.

## Installation

```toml
[dependencies]
tpt-proc-hvac = "0.1"
```

## Usage

```rust
use tpt_proc_hvac::Psychrometrics;

let ps = Psychrometrics::new(101_325.0);
// Saturation pressure of water at 20 °C ≈ 2.34 kPa.
let p_sat = ps.saturation_pressure(293.15);
assert!((p_sat - 2339.0).abs() < 30.0);

// 20 °C, 50% RH → humidity ratio ≈ 0.00726 kg/kg.
let w = ps.humidity_ratio_from_rh(293.15, 0.5);
assert!((w - 0.00726).abs() < 2e-4);

// Moist-air enthalpy ≈ 38.6 kJ/kg dry air.
let h = ps.enthalpy(293.15, w);
assert!((h - 38_600.0).abs() < 500.0);
```

## API tour

| Item | Purpose |
|---|---|
| [`Psychrometrics`] | State computations at a fixed total pressure (`pressure` field, Pa). |
| `saturation_pressure` | Water p_sat, Pa, ASHRAE/Magnus correlation. |
| `humidity_ratio_from_rh` / `relative_humidity` | Humidity ratio ↔ relative humidity. |
| `enthalpy` | Moist-air enthalpy, J/kg dry air (0 °C reference). |
| `specific_volume` | Specific volume, m³/kg dry air. |
| `dew_point` / `wet_bulb` | Dew-point and thermodynamic wet-bulb temperatures, K. |
| `mixing` | Adiabatic mix of two moist-air streams. |
| [`STANDARD_ATMOSPHERE`] etc. | 101 325 Pa, cp of air/vapor, h_fg at 0 °C. |

## Scope and limitations

- Ideal-gas moist air at **near-atmospheric pressures** only; no
  enhancement factors or real-gas corrections for high pressure.
- The saturation correlation is valid 273–373 K (within ~0.1%); outside that
  range accuracy degrades and no warning is given.
- `wet_bulb` is the thermodynamic (adiabatic saturation) wet-bulb by
  enthalpy matching — not an instrument/sling psychrometer correlation.
- `mixing` performs a mass/energy balance only: it does **not** check the
  mixed state for supersaturation or fog formation.
- No equipment models: no coils, cooling towers, humidifiers, fans, or duct
  sizing — air-side property math only.
- SI units only (K, Pa, J, kg); no IP-unit conversions.

## Related crates

- [`tpt-proc-core`](../../core/tpt-proc-core) — the stream and unit types of
  the suite (declared companion dependency).
- [`example-hvac-psychrometrics`](../../../examples/hvac-psychrometrics) —
  the worked example binary built on this crate.

## Minimum Supported Rust Version

Rust 1.84 (workspace MSRV).

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE)) or MIT ([LICENSE-MIT](../../../LICENSE-MIT)) at your option.
