//! Small-plant economics: six-tenths CAPEX scaling to a 150 kt/a design,
//! annual OPEX, project cashflows, NPV/IRR/discounted payback, and the
//! levelized cost of product.

use tpt_proc_economics::{
    annual_opex, cashflows, discounted_payback, irr, levelized_cost, npv, six_tenths_capex,
};

fn main() {
    // CAPEX: a 100 kt/a reference plant cost 80 M; scale to 150 kt/a with
    // the six-tenths rule.
    let capex = six_tenths_capex(80.0e6, 100.0e3, 150.0e3, 0.6);
    println!("CAPEX (six-tenths, 100 -> 150 kt/a): {:.1} M$", capex / 1e6);

    // OPEX: 6 M/a fixed plus 55 $/t variable at nameplate throughput.
    let throughput = 150.0e3; // t/a
    let opex = annual_opex(6.0e6, 55.0, throughput);
    println!("OPEX (6 M fixed + 55 $/t): {:.2} M$/a", opex / 1e6);

    // Margin: product sells at 220 $/t.
    let revenue = 220.0 * throughput;
    let margin = revenue - opex;
    println!(
        "Revenue {:.2} M$/a -> annual margin {:.2} M$/a",
        revenue / 1e6,
        margin / 1e6
    );

    // Cashflows, 15-year life, 10% discount rate.
    let (rate, life) = (0.10, 15u32);
    let flows = cashflows(-capex, margin, life);
    let value = npv(rate, &flows);
    let rate_of_return = irr(&flows).expect("profitable project has an IRR");
    let payback = discounted_payback(rate, &flows).expect("project pays back");
    println!(
        "\nProject metrics at r = {:.0}%, {life} years:",
        100.0 * rate
    );
    println!("  NPV = {:.2} M$", value / 1e6);
    println!("  IRR = {:.1}%", 100.0 * rate_of_return);
    println!("  discounted payback = {:.2} years", payback);
    assert!(value > 0.0 && rate_of_return > rate);

    // Sensitivity: the same project at 8% and 12%.
    for r in [0.08, 0.12] {
        println!(
            "  NPV at r = {:.0}%: {:.2} M$",
            100.0 * r,
            npv(r, &flows) / 1e6
        );
    }

    // Levelized cost of product: annualized capex plus opex per tonne.
    let lc = levelized_cost(capex, opex, throughput, rate, life);
    println!(
        "\nLevelized cost of product: {:.1} $/t (vs 220 $/t price)",
        lc
    );
    assert!(lc < 220.0);
}
