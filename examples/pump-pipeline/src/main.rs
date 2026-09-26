//! A pumping pipeline (pump curve vs. system curve), a control-valve Cv
//! check, and a compressor stage — one fluid-flow story in three sections.

use tpt_proc_compressors::CompressorService;
use tpt_proc_fluid::PipeFlow;
use tpt_proc_pumps::{Pump, PumpCurve, SystemCurve};
use tpt_proc_valves::{cv_to_kv, ControlValve, ValveCharacteristic};

fn main() {
    // ---- Section 1: pump operating point on a pipeline ----
    // 200 m of 0.15 m commercial-steel line lifting water 12 m; quadratic
    // pump curve H = 35 − 120·Q − 900·Q² (H in m, Q in m³/s).
    let line = PipeFlow::new(0.15, 200.0, 4.6e-5);
    let (density, g) = (998.0, 9.80665);
    let q_nom = 0.04; // m³/s
    let friction_k = line.pressure_drop_at_flow(q_nom) / (density * g) / q_nom.powi(2);
    let system = SystemCurve {
        static_head: 12.0,
        k: friction_k,
    };
    let pump = Pump {
        curve: PumpCurve {
            shutoff_head: 35.0,
            a: -120.0,
            b: -900.0,
        },
        speed_rpm: 1750.0,
        bep_flow: 0.045,
        peak_efficiency: 0.74,
    };
    let (q, h) = pump
        .operating_point(&system, 0.2)
        .expect("pump crosses the system curve");
    let efficiency = pump.efficiency_at_flow(q);
    let shaft_power = pump.power_at_flow(q, density);
    println!("Pipeline: 200 m x 0.15 m, static lift 12 m (system k = {friction_k:.0} m/(m³/s)²)");
    println!("Operating point: Q = {q:.5} m³/s at H = {h:.2} m");
    println!(
        "  efficiency η = {efficiency:.3}, shaft power P = {:.1} kW",
        shaft_power / 1e3
    );
    let npsh = Pump::npsh_available(101_325.0, 2340.0, 2.0, 0.5, density);
    println!("  NPSH available = {npsh:.2} m");
    assert!(h > system.static_head && q > 0.0);

    // ---- Section 2: control-valve Cv check ----
    // Throttle 0.04 m³/s of water through 10 psi (68.9 kPa).
    let (q_valve, dp_valve) = (0.04, 68_947.6);
    let cv_required = ControlValve::required_cv_liquid(q_valve, dp_valve, 1.0);
    let valve = ControlValve::new(250.0, ValveCharacteristic::Linear);
    let delivered = valve.liquid_flow_m3s(0.8, dp_valve, 1.0, 0.0);
    println!("\nValve: {q_valve:.3} m³/s at ΔP = 10 psi needs Cv = {cv_required:.0}");
    println!(
        "  Cv 250 linear at 80% open: Cv = {:.0}, Q = {:.5} m³/s (Kv = {:.0})",
        valve.cv_at(0.8),
        delivered,
        cv_to_kv(valve.cv_at(0.8))
    );
    let eq_pct = ControlValve::new(250.0, ValveCharacteristic::EqualPercentage);
    println!(
        "  Cv 250 equal-percentage at 80% open: Cv = {:.0} (installed-range gain grows with opening)",
        eq_pct.cv_at(0.8)
    );
    assert!(delivered > 0.0);

    // ---- Section 3: compressor stage (methane, 1 -> 4 bar) ----
    let methane_rs = 8.314_462_618 / 0.016_043; // J/(kg·K)
    let service = CompressorService {
        suction_pressure: 101_325.0,
        discharge_pressure: 405_300.0,
        suction_temperature: 300.0,
        kappa: 1.31,
        isentropic_efficiency: 0.75,
        compressibility: 0.97,
    };
    let result = service.evaluate(methane_rs);
    println!(
        "\nCompressor: methane, r = {:.2}, η_s = 0.75",
        result.pressure_ratio
    );
    println!(
        "  isentropic head = {:.1} kJ/kg, actual work = {:.1} kJ/kg",
        result.head_isentropic / 1e3,
        result.work_actual / 1e3
    );
    println!(
        "  discharge temperature = {:.1} K",
        result.discharge_temperature
    );
    println!(
        "  shaft power at 2 kg/s = {:.0} kW (polytropic head at η_p = 0.72: {:.1} kJ/kg)",
        service.power(2.0, methane_rs) / 1e3,
        service.polytropic_head(methane_rs, 0.72) / 1e3
    );
    assert!(result.discharge_temperature > service.suction_temperature);
}
