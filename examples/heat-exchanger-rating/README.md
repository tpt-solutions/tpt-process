# example-heat-exchanger-rating

Rating and sizing a shell-and-tube heat exchanger. Given the area and
overall coefficient of a 1-2 counter-current exchanger, the rating
(ε-NTU) problem asks what duty and outlet temperatures are reached with
two given streams; the sizing (LMTD) problem asks how much area a new
duty requires. The example also computes a tube-side film coefficient
from first principles (Dittus-Boelter) and builds an overall coefficient
from series resistances including fouling.

## Run

From the repository root:

```sh
cargo run --release -p example-heat-exchanger-rating
```

## What it shows

- Rating a 25 m² counter-current exchanger with
  `HeatExchanger::new(area, u, FlowConfiguration::CounterCurrent)` and
  `HeatExchanger::rate(...)` (duty, outlet temperatures, effectiveness).
- A film coefficient from scratch: `convection::reynolds_internal`,
  `convection::prandtl`, `convection::dittus_boelter` (with
  `FlowRegime::Heating`), and `convection::heat_transfer_coefficient`.
- Series resistance addition — convective, fouling, convective — with
  `WallResistance` and `HeatExchanger::overall_from_resistances`.
- Sizing a 60 kW service against fixed terminal temperatures with
  `HeatExchanger::required_area(...)` (LMTD method).

## Crates used

- [tpt-proc-heat-exchangers](../../crates/heat-transfer/tpt-proc-heat-exchangers)
- [tpt-proc-heat-transfer](../../crates/heat-transfer/tpt-proc-heat-transfer)

## Suite

Part of the [tpt-process](../../README.md) suite, MIT OR Apache-2.0.
