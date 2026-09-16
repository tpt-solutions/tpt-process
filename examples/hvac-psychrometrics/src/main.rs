//! Psychrometrics of moist air: property lookups and an air-mixing
//! calculation for an HVAC duty estimate.

use tpt_proc_hvac::Psychrometrics;

fn main() {
    let air = Psychrometrics::new(101_325.0);

    // Outdoor 35 °C / 60% RH; room 24 °C / 50% RH.
    let (t_out, rh_out) = (308.15_f64, 0.6);
    let (t_room, rh_room) = (297.15_f64, 0.5);

    let w_out = air.humidity_ratio_from_rh(t_out, rh_out);
    let w_room = air.humidity_ratio_from_rh(t_room, rh_room);

    println!("Outdoor: {:.1} °C, W = {:.5} kg/kg", t_out - 273.15, w_out);
    println!(
        "  dew point {:.1} °C, wet bulb {:.1} °C",
        air.dew_point(t_out, w_out) - 273.15,
        air.wet_bulb(t_out, w_out) - 273.15
    );
    println!("Room: {:.1} °C, W = {:.5} kg/kg", t_room - 273.15, w_room);

    // Mix 1 kg/s outdoor air with 3 kg/s recirculated room air.
    let (t_mix, w_mix) = air.mixing(t_out, w_out, 1.0, t_room, w_room, 3.0);
    let h_mix = air.enthalpy(t_mix, w_mix);
    println!("\nMixed stream (1:3 outdoor:room):");
    println!(
        "  {:.1} °C, W = {:.5} kg/kg, h = {:.1} kJ/kg",
        t_mix - 273.15,
        w_mix,
        h_mix / 1e3
    );

    // Cooling coil duty to bring the mix to 13 °C dew-point-saturated
    // supply: Δh on the dry-air basis.
    let w_supply = air.humidity_ratio_from_rh(286.15, 0.95);
    let h_supply = air.enthalpy(286.15, w_supply);
    println!(
        "\nCoil duty for 4 kg/s dry air to a 13 °C / 95% RH supply: {:.1} kW",
        4.0 * (h_mix - h_supply) / 1e3
    );
}
