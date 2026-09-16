//! Rate a counter-current heat exchanger with the ε-NTU method and size
//! an alternative service with the LMTD method.

use tpt_proc_heat_exchangers::{FlowConfiguration, HeatExchanger};
use tpt_proc_heat_transfer::{convection, FlowRegime, WallResistance};

fn main() {
    // A 1-2 shell-and-tube exchanger: 25 m², U = 900 W/(m²·K).
    let hx = HeatExchanger::new(25.0, 900.0, FlowConfiguration::CounterCurrent);

    // Service: hot stream (2 kW/K) in at 110 °C, coolant (4 kW/K) at 17 °C.
    let hot_in = 383.15;
    let cold_in = 290.0;
    let rated = hx.rate(2.0e3, 4.0e3, f64::MAX, hot_in, cold_in);

    println!("Rating (ε-NTU, counter-current):");
    println!("  duty:          {:.1} kW", rated.duty / 1e3);
    println!("  hot outlet:    {:.1} °C", rated.hot_outlet - 273.15);
    println!("  cold outlet:   {:.1} °C", rated.cold_outlet - 273.15);
    println!("  effectiveness: {:.3}", rated.effectiveness);

    // Tube-side water at 1 m/s: Dittus-Boelter film coefficient.
    let re = convection::reynolds_internal(998.0, 1.0, 0.02, 1.0e-3);
    let pr = convection::prandtl(4180.0, 1.0e-3, 0.6);
    let nu = convection::dittus_boelter(re, pr, FlowRegime::Heating);
    let h = convection::heat_transfer_coefficient(nu, 0.6, 0.02);
    println!("\nTube-side film coefficient at Re = {re:.0}: h = {h:.0} W/(m²·K)");

    // Series resistances → clean overall coefficient per unit area.
    let wall = WallResistance::new()
        .convective(h)
        .fouling(0.0002)
        .convective(2_000.0);
    let u_clean = HeatExchanger::overall_from_resistances(1.0, wall);
    println!("Overall U (per unit area, fouled): {u_clean:.0} W/(m²·K)");

    // Size a duty of 60 kW against fixed terminal temperatures.
    let area = hx.required_area(60.0e3, 373.15, 313.15, 290.0, 320.0);
    println!("Required area for 60 kW: {area:.1} m²");
}
