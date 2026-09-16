//! Heat exchanger rating and sizing: LMTD (with correction factor),
//! NTU-effectiveness, duty calculations.
//!
//! Reference: Incropera's *Fundamentals of Heat and Mass Transfer*;
//! tolerances in tests are absolute against hand-computed values.
//!
//! # Example
//!
//! ```
//! use tpt_proc_heat_exchangers::{FlowConfiguration, HeatExchanger};
//!
//! let hx = HeatExchanger::new(25.0, 900.0, FlowConfiguration::CounterCurrent);
//!
//! // LMTD of counter-current 100→30 / 20→50 °C service.
//! let lmtd = hx.lmtd(373.15, 303.15, 293.15, 323.15);
//! let expected = 40.0 / 5.0f64.ln(); // K, terminal differences 50 & 10
//! assert!((lmtd - expected).abs() < 1e-6);
//!
//! // Rating: duty and outlet temperatures from UA and inlet states.
//! let rated = hx.rate(2.0e3, 4.0e3, 3.0e3, 330.0, 290.0);
//! assert!(rated.duty > 0.0);
//! assert!(rated.hot_outlet < 330.0 && rated.cold_outlet > 290.0);
//! ```

#![forbid(unsafe_code)]

use tpt_proc_heat_transfer::WallResistance;

/// Flow arrangement of the exchanger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowConfiguration {
    /// True counter-current.
    CounterCurrent,
    /// Co-current (parallel).
    CoCurrent,
    /// 1 shell pass, 2 or more tube passes (Bowman correlation for F).
    ShellAndTube1ShellPass,
}

/// Result of a rating calculation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RatingResult {
    /// Heat duty, W.
    pub duty: f64,
    /// Hot outlet temperature, K.
    pub hot_outlet: f64,
    /// Cold outlet temperature, K.
    pub cold_outlet: f64,
    /// Effectiveness ε ∈ [0, 1].
    pub effectiveness: f64,
}

/// A heat exchanger with fixed UA.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatExchanger {
    /// Heat-transfer area, m².
    pub area: f64,
    /// Clean overall coefficient, W/(m²·K).
    pub overall_u: f64,
    /// Flow arrangement.
    pub configuration: FlowConfiguration,
}

impl HeatExchanger {
    /// Creates an exchanger.
    #[must_use]
    pub const fn new(area: f64, overall_u: f64, configuration: FlowConfiguration) -> Self {
        Self {
            area,
            overall_u,
            configuration,
        }
    }

    /// Builds the overall coefficient from resistance layers (fouling
    /// inclusive): U = 1/(R_total · A).
    #[must_use]
    pub fn overall_from_resistances(area: f64, resistance: WallResistance) -> f64 {
        resistance.overall_u(area)
    }

    /// UA product, W/K.
    #[must_use]
    pub const fn ua(&self) -> f64 {
        self.area * self.overall_u
    }

    /// Log-mean temperature difference, K (identical terminal differences
    /// collapse to their common value).
    #[must_use]
    pub fn lmtd(&self, hot_in: f64, hot_out: f64, cold_in: f64, cold_out: f64) -> f64 {
        // Counter-current terminal differences; the LMTD magnitude is the
        // same for co-current with its own pairing — we take the general
        // definition on the caller's pairing.
        let dt1 = hot_in - cold_out;
        let dt2 = hot_out - cold_in;
        if (dt1 - dt2).abs() < 1e-9 {
            return dt1;
        }
        (dt1 - dt2) / (dt1 / dt2).ln()
    }

    /// LMTD correction factor F for the configuration (1 for
    /// counter-current; Bowman chart fit for 1-shell-pass).
    #[must_use]
    pub fn correction_factor(&self, hot_in: f64, hot_out: f64, cold_in: f64, cold_out: f64) -> f64 {
        match self.configuration {
            FlowConfiguration::CounterCurrent => 1.0,
            FlowConfiguration::CoCurrent => 1.0, // LMTD already co-current form
            FlowConfiguration::ShellAndTube1ShellPass => {
                // Bowman: R = (T1−T2)/(t2−t1), P = (t2−t1)/(T1−t1).
                let r = (hot_in - hot_out) / (cold_out - cold_in);
                let p = (cold_out - cold_in) / (hot_in - cold_in);
                if (r - 1.0).abs() < 1e-9 {
                    return 1.0;
                }
                let s = (r * r + 1.0).sqrt() / (r - 1.0);
                let w = ((1.0 - p * r) / (1.0 - p)).max(1e-9);
                let f = s * (w).ln()
                    / (2.0
                        * ((1.0 + r - p + (r * r + 1.0).sqrt())
                            / (1.0 + r - p - (r * r + 1.0).sqrt()))
                        .ln());
                f.clamp(0.0, 1.0)
            }
        }
    }

    /// Duty from the LMTD method: Q = U·A·LMTD·F, W.
    #[must_use]
    pub fn duty_from_lmtd(
        &self,
        hot_in: f64,
        hot_out: f64,
        cold_in: f64,
        cold_out: f64,
        correction: f64,
    ) -> f64 {
        self.ua() * self.lmtd(hot_in, hot_out, cold_in, cold_out) * correction
    }

    /// Effectiveness from the NTU method for counter-current flow:
    /// ε = (1 − e^(−NTU(1−Cr)))/(1 − Cr·e^(−NTU(1−Cr))).
    #[must_use]
    pub fn effectiveness(&self, c_hot: f64, c_cold: f64) -> f64 {
        let c_min = c_hot.min(c_cold);
        let c_max = c_hot.max(c_cold);
        let cr = c_min / c_max;
        let ntu = self.ua() / c_min;
        match self.configuration {
            FlowConfiguration::CounterCurrent => {
                if (cr - 1.0).abs() < 1e-9 {
                    ntu / (1.0 + ntu)
                } else {
                    let e = (-ntu * (1.0 - cr)).exp();
                    (1.0 - e) / (1.0 - cr * e)
                }
            }
            FlowConfiguration::CoCurrent => {
                let e = (-ntu * (1.0 + cr)).exp();
                (1.0 - e) / (1.0 + cr)
            }
            FlowConfiguration::ShellAndTube1ShellPass => {
                // Bowman 1-shell-pass effectiveness.
                let e = (-ntu).exp();
                2.0 / (1.0 + cr + (1.0 + cr * cr).sqrt() * (1.0 + e) / (1.0 - e))
            }
        }
    }

    /// Rating: given capacity rates and inlet temperatures, find duty and
    /// outlets via the ε-NTU method.
    #[must_use]
    pub fn rate(
        &self,
        c_hot: f64,
        c_cold: f64,
        duty_cap: f64,
        hot_in: f64,
        cold_in: f64,
    ) -> RatingResult {
        let _ = duty_cap;
        let effectiveness = self.effectiveness(c_hot, c_cold);
        let c_min = c_hot.min(c_cold);
        let duty = effectiveness * c_min * (hot_in - cold_in);
        RatingResult {
            duty,
            hot_outlet: hot_in - duty / c_hot,
            cold_outlet: cold_in + duty / c_cold,
            effectiveness,
        }
    }

    /// Sizing: given required duty and four terminal temperatures, return
    /// the required area, m².
    #[must_use]
    pub fn required_area(
        &self,
        duty: f64,
        hot_in: f64,
        hot_out: f64,
        cold_in: f64,
        cold_out: f64,
    ) -> f64 {
        let f = self.correction_factor(hot_in, hot_out, cold_in, cold_out);
        duty / (self.overall_u * self.lmtd(hot_in, hot_out, cold_in, cold_out) * f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hx() -> HeatExchanger {
        HeatExchanger::new(25.0, 900.0, FlowConfiguration::CounterCurrent)
    }

    #[test]
    fn lmtd_matches_reference() {
        // Terminal differences 50 K and 10 K: LMTD = 40/ln(5) = 24.85 K.
        let lmtd = hx().lmtd(373.15, 303.15, 293.15, 323.15);
        assert!((lmtd - 40.0_f64 / 5.0_f64.ln()).abs() < 1e-6);
    }

    #[test]
    fn lmtd_equal_terminal_differences() {
        // ΔT1 = ΔT2 = 30 → LMTD = 30 (avoids 0/0).
        let lmtd = hx().lmtd(353.15, 323.15, 293.15, 323.15);
        assert!((lmtd - 30.0).abs() < 1e-6);
    }

    #[test]
    fn counter_current_beats_co_current() {
        let counter = HeatExchanger::new(1.0, 500.0, FlowConfiguration::CounterCurrent);
        let co = HeatExchanger::new(1.0, 500.0, FlowConfiguration::CoCurrent);
        let (ch, cc) = (1.0e3, 2.0e3);
        assert!(counter.effectiveness(ch, cc) > co.effectiveness(ch, cc));
    }

    #[test]
    fn effectiveness_limits() {
        // Cr = 1 counter-current: ε = NTU/(1+NTU).
        let hx1 = HeatExchanger::new(1.0, 100.0, FlowConfiguration::CounterCurrent);
        let eps = hx1.effectiveness(1.0e3, 1.0e3); // NTU = 0.1
        assert!((eps - 0.1 / 1.1).abs() < 1e-9);
        // NTU → ∞ counter-current: ε → 1/(1+Cr) handled via Cr=0 limit.
        let huge = HeatExchanger::new(1e9, 1e3, FlowConfiguration::CounterCurrent);
        assert!(huge.effectiveness(1.0e3, 2.0e3) > 0.9);
    }

    #[test]
    fn rating_energy_balance_closes() {
        let rated = hx().rate(2.0e3, 4.0e3, f64::MAX, 330.0, 290.0);
        // Q = ch·(Th_in − Th_out) = cc·(Tc_out − Tc_in).
        let q_hot = 2.0e3 * (330.0 - rated.hot_outlet);
        let q_cold = 4.0e3 * (rated.cold_outlet - 290.0);
        assert!((q_hot - rated.duty).abs() < 1e-6);
        assert!((q_cold - rated.duty).abs() < 1e-6);
        // Duty cannot exceed the thermodynamic maximum.
        let q_max = 2.0e3 * (330.0 - 290.0);
        assert!(rated.duty <= q_max * (1.0 + 1e-12));
    }

    #[test]
    fn sizing_inverts_the_lmtd_method() {
        // Duty of UA·LMTD requires exactly the given area back.
        let duty = hx().duty_from_lmtd(373.15, 333.15, 293.15, 323.15, 1.0);
        let area = hx().required_area(duty, 373.15, 333.15, 293.15, 323.15);
        assert!((area - 25.0).abs() < 1e-9);
    }

    #[test]
    fn fouling_reduces_overall_u() {
        let clean = WallResistance::new().convective(1000.0).convective(1000.0);
        let fouled = clean.fouling(0.001);
        assert!(fouled.total > clean.total);
        let u_clean = HeatExchanger::overall_from_resistances(10.0, clean);
        let u_fouled = HeatExchanger::overall_from_resistances(10.0, fouled);
        assert!(u_fouled < u_clean);
    }
}
