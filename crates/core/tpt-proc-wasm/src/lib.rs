//! WebAssembly bindings for the `tpt-process` engineering suite.
//!
//! This crate is the JS boundary of the stack: browser-based PFD tools,
//! educational simulators, and edge controllers consume the solver crates
//! through the `#[wasm_bindgen]` façades defined here. The crate builds
//! for native targets too (bindings become plain Rust types), so the whole
//! workspace is testable on any platform.
//!
//! # Building for the browser
//!
//! ```sh
//! cargo build -p tpt-proc-wasm --target wasm32-unknown-unknown
//! wasm-bindgen target/wasm32-unknown-unknown/debug/tpt_proc_wasm.wasm --out-dir pkg
//! ```

#![forbid(unsafe_code)]

use std::sync::Arc;

use wasm_bindgen::prelude::*;

use tpt_proc_core::Composition;
use tpt_proc_thermo_core::{Component, PropertyPackage};
use tpt_proc_thermo_database::ChemicalDatabase;
use tpt_proc_thermo_eos::CubicEos;
use tpt_proc_thermo_phase::FlashSolver;

fn error(kind: &str, message: String) -> JsValue {
    JsValue::from(WasmError::new(kind, &message))
}

/// Result of a WASM-bound flash calculation (serialized to JS as a plain
/// object).
#[derive(Clone, Debug, PartialEq)]
#[wasm_bindgen]
pub struct WasmFlashResult {
    vapor_fraction: f64,
    temperature: f64,
    pressure: f64,
    iterations: u32,
    degenerate: bool,
    liquid: Vec<f64>,
    vapor: Vec<f64>,
}

#[wasm_bindgen]
impl WasmFlashResult {
    /// Molar vapor fraction β in [0, 1].
    #[wasm_bindgen(getter)]
    pub fn vapor_fraction(&self) -> f64 {
        self.vapor_fraction
    }

    /// Equilibrium temperature, K.
    #[wasm_bindgen(getter)]
    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    /// Equilibrium pressure, Pa.
    #[wasm_bindgen(getter)]
    pub fn pressure(&self) -> f64 {
        self.pressure
    }

    /// K-value iterations performed.
    #[wasm_bindgen(getter)]
    pub fn iterations(&self) -> u32 {
        self.iterations
    }

    /// True for pure-component saturation points (β not unique).
    #[wasm_bindgen(getter)]
    pub fn degenerate(&self) -> bool {
        self.degenerate
    }

    /// Vapor mole fractions, aligned with the constructor's component list.
    #[wasm_bindgen(getter)]
    pub fn vapor_composition(&self) -> Vec<f64> {
        self.vapor.clone()
    }

    /// Liquid mole fractions, aligned with the constructor's component list.
    #[wasm_bindgen(getter)]
    pub fn liquid_composition(&self) -> Vec<f64> {
        self.liquid.clone()
    }
}

/// Result of a WASM-bound heat-exchanger rating.
#[derive(Clone, Debug, PartialEq)]
#[wasm_bindgen]
pub struct WasmRatingResult {
    duty: f64,
    hot_outlet: f64,
    cold_outlet: f64,
    effectiveness: f64,
}

#[wasm_bindgen]
impl WasmRatingResult {
    /// Heat duty, W.
    #[wasm_bindgen(getter)]
    pub fn duty(&self) -> f64 {
        self.duty
    }

    /// Hot outlet temperature, K.
    #[wasm_bindgen(getter)]
    pub fn hot_outlet(&self) -> f64 {
        self.hot_outlet
    }

    /// Cold outlet temperature, K.
    #[wasm_bindgen(getter)]
    pub fn cold_outlet(&self) -> f64 {
        self.cold_outlet
    }

    /// Effectiveness ε ∈ [0, 1].
    #[wasm_bindgen(getter)]
    pub fn effectiveness(&self) -> f64 {
        self.effectiveness
    }
}

/// Error surfaced across the JS boundary.
#[derive(Clone, Debug, PartialEq)]
#[wasm_bindgen]
pub struct WasmError {
    kind: String,
    message: String,
}

#[wasm_bindgen]
impl WasmError {
    /// Error class tag (`"validation"`, `"solver"`, `"lookup"`).
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        self.kind.clone()
    }

    /// Human-readable message.
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl WasmError {
    /// Builds an error with the given class tag and message.
    pub fn new(kind: &str, message: &str) -> Self {
        Self {
            kind: kind.to_string(),
            message: message.to_string(),
        }
    }
}

/// Browser-facing flash calculator over the built-in database and the
/// Peng-Robinson EOS.
#[wasm_bindgen]
pub struct WasmFlashCalculator {
    names: Vec<String>,
    solver: FlashSolver,
}

#[wasm_bindgen]
impl WasmFlashCalculator {
    /// Creates a calculator for a whitespace-separated component list
    /// resolved against the built-in database (e.g. `"water methanol"`).
    #[wasm_bindgen(constructor)]
    pub fn new(components_json: &str) -> Result<WasmFlashCalculator, JsValue> {
        let names: Vec<String> = components_json
            .split_whitespace()
            .map(str::to_string)
            .collect();
        if names.is_empty() {
            return Err(error(
                "validation",
                "component list must not be empty".into(),
            ));
        }
        let db = ChemicalDatabase::builtin();
        let lookup: Vec<&str> = names.iter().map(String::as_str).collect();
        let components = db
            .components_for(&lookup)
            .map_err(|e| error("lookup", format!("database lookup failed: {e}")))?;
        let eos = CubicEos::peng_robinson(components.clone());
        let package = PropertyPackage::new(components)
            .map_err(|e| error("validation", e.to_string()))?
            .with_eos(Arc::new(eos));
        Ok(Self {
            names,
            solver: FlashSolver::new(package),
        })
    }

    /// Component names backing this calculator.
    #[wasm_bindgen(getter)]
    pub fn component_names(&self) -> Vec<String> {
        self.names.clone()
    }

    /// PT flash of `composition` (mole fractions) at `temperature` (K) and
    /// `pressure` (Pa).
    ///
    /// # Errors
    /// Returns a [`WasmError`] for invalid input or solver failure.
    pub fn pt_flash(
        &self,
        composition: &[f64],
        temperature: f64,
        pressure: f64,
    ) -> Result<WasmFlashResult, JsValue> {
        if composition.len() != self.names.len() {
            return Err(error(
                "validation",
                format!(
                    "composition length {} must match the component list {}",
                    composition.len(),
                    self.names.len()
                ),
            ));
        }
        let fractions = Composition::from_mole_fractions(composition)
            .map_err(|e| error("validation", e.to_string()))?;
        let result = self
            .solver
            .pt_flash(&fractions, temperature, pressure)
            .map_err(|e| error("solver", e.to_string()))?;
        Ok(WasmFlashResult {
            vapor_fraction: result.vapor_fraction,
            temperature: result.temperature,
            pressure: result.pressure,
            iterations: result.iterations,
            degenerate: result.degenerate,
            liquid: result.liquid_composition.as_slice().to_vec(),
            vapor: result.vapor_composition.as_slice().to_vec(),
        })
    }

    /// Bubble-point temperature of the given liquid composition, K.
    pub fn bubble_point(&self, composition: &[f64], pressure: f64) -> Result<f64, JsValue> {
        let fractions = Composition::from_mole_fractions(composition)
            .map_err(|e| error("validation", e.to_string()))?;
        self.solver
            .bubble_point_t(&fractions, pressure)
            .map_err(|e| error("solver", e.to_string()))
    }
}

/// Browser-facing heat-exchanger rating façade.
#[wasm_bindgen]
pub struct WasmHeatExchanger {
    area: f64,
    overall_u: f64,
}

#[wasm_bindgen]
impl WasmHeatExchanger {
    /// Creates an exchanger facade from area (m2) and overall coefficient
    /// (W/(m2*K)).
    #[wasm_bindgen(constructor)]
    pub fn new(area: f64, overall_u: f64) -> Result<WasmHeatExchanger, JsValue> {
        if !(area.is_finite() && area > 0.0) || !(overall_u.is_finite() && overall_u > 0.0) {
            return Err(error(
                "validation",
                "area and overall coefficient must be finite and positive".into(),
            ));
        }
        Ok(Self { area, overall_u })
    }

    /// Heat-transfer area, m2.
    #[wasm_bindgen(getter)]
    pub fn area(&self) -> f64 {
        self.area
    }

    /// Overall heat-transfer coefficient, W/(m2*K).
    #[wasm_bindgen(getter)]
    pub fn overall_u(&self) -> f64 {
        self.overall_u
    }

    /// Rates a counter-current exchanger via the epsilon-NTU method.
    pub fn rate(
        &self,
        c_hot: f64,
        c_cold: f64,
        hot_in: f64,
        cold_in: f64,
    ) -> Result<WasmRatingResult, JsValue> {
        if !(c_hot.is_finite() && c_hot > 0.0) || !(c_cold.is_finite() && c_cold > 0.0) {
            return Err(error(
                "validation",
                "capacity rates must be finite and positive".into(),
            ));
        }
        if hot_in <= cold_in {
            return Err(error(
                "validation",
                format!("hot inlet {hot_in} K must exceed cold inlet {cold_in} K"),
            ));
        }
        let hx = tpt_proc_heat_exchangers::HeatExchanger::new(
            self.area,
            self.overall_u,
            tpt_proc_heat_exchangers::FlowConfiguration::CounterCurrent,
        );
        let rated = hx.rate(c_hot, c_cold, f64::MAX, hot_in, cold_in);
        Ok(WasmRatingResult {
            duty: rated.duty,
            hot_outlet: rated.hot_outlet,
            cold_outlet: rated.cold_outlet,
            effectiveness: rated.effectiveness,
        })
    }
}

/// Component constants for one chemical, serialized to JS as an object.
#[wasm_bindgen]
pub struct WasmComponent {
    name: String,
    cas: String,
    critical_temperature: f64,
    critical_pressure: f64,
    acentric_factor: f64,
}

#[wasm_bindgen]
impl WasmComponent {
    /// Chemical name.
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// CAS number.
    #[wasm_bindgen(getter)]
    pub fn cas(&self) -> String {
        self.cas.clone()
    }

    /// Critical temperature, K.
    #[wasm_bindgen(getter)]
    pub fn critical_temperature(&self) -> f64 {
        self.critical_temperature
    }

    /// Critical pressure, Pa.
    #[wasm_bindgen(getter)]
    pub fn critical_pressure(&self) -> f64 {
        self.critical_pressure
    }

    /// Pitzer acentric factor.
    #[wasm_bindgen(getter)]
    pub fn acentric_factor(&self) -> f64 {
        self.acentric_factor
    }
}

/// Looks a component up in the built-in database (by name or CAS).
pub fn lookup_component(name_or_cas: &str) -> Option<WasmComponent> {
    let db = ChemicalDatabase::builtin();
    db.get_component(name_or_cas)
        .map(|c: &Component| WasmComponent {
            name: c.name.clone(),
            cas: c.cas_number.clone(),
            critical_temperature: c.critical_temperature,
            critical_pressure: c.critical_pressure,
            acentric_factor: c.acentric_factor,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flash_calculator_solves_a_binary() {
        let calc = WasmFlashCalculator::new("water methanol").unwrap();
        assert_eq!(calc.component_names().len(), 2);
        let result = calc.pt_flash(&[0.5, 0.5], 350.0, 101_325.0).expect("flash");
        assert!((0.0..=1.0).contains(&result.vapor_fraction()));
        assert_eq!(result.vapor_composition().len(), 2);
        // Degenerate pure-component flash carries the flag.
        let pure = WasmFlashCalculator::new("water").unwrap();
        let r = pure.pt_flash(&[1.0], 373.15, 101_325.0).unwrap();
        assert!(r.degenerate());
    }

    #[test]
    fn bubble_point_wiring() {
        let calc = WasmFlashCalculator::new("benzene toluene").unwrap();
        let t = calc.bubble_point(&[1.0, 0.0], 101_325.0).unwrap();
        assert!((t - 353.25).abs() < 4.0, "T = {t}");
    }

    #[test]
    fn exchanger_rating_wiring() {
        let hx = WasmHeatExchanger::new(25.0, 900.0).unwrap();
        let rated = hx.rate(2.0e3, 4.0e3, 330.0, 290.0).unwrap();
        assert!(rated.duty() > 0.0);
        assert!(rated.hot_outlet() < 330.0 && rated.cold_outlet() > 290.0);
    }

    #[test]
    fn component_lookup() {
        let water = lookup_component("water").expect("water in database");
        assert_eq!(water.cas(), "7732-18-5");
        assert!((water.critical_temperature() - 647.096).abs() < 1e-6);
        assert!(lookup_component("unobtanium").is_none());
    }
}
