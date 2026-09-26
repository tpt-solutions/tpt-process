# Changelog

All notable changes to `tpt-proc-wasm` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The crate shares the workspace version (currently **0.1.0**, not yet published to crates.io).

## [Unreleased]

### Added
- `WasmFlashCalculator`: `#[wasm_bindgen]` flash façade over the built-in `ChemicalDatabase` and the Peng-Robinson `CubicEos`, with constructor component resolution, `pt_flash`, `bubble_point`, and the `component_names` getter.
- `WasmFlashResult`: getters for `vapor_fraction`, `temperature`, `pressure`, `iterations`, `degenerate`, `liquid_composition`, and `vapor_composition`.
- `WasmHeatExchanger`: epsilon-NTU counter-current rating façade over `tpt-proc-heat-exchangers`, with `area`/`overall_u` getters and a `rate` method.
- `WasmRatingResult`: getters for `duty`, `hot_outlet`, `cold_outlet`, and `effectiveness`.
- `WasmComponent` and `lookup_component`: single-component constants (CAS, critical properties, acentric factor) by name or CAS lookup.
- `WasmError`: structured error kind (`"validation"`, `"solver"`, `"lookup"`) and message crossing the JS boundary.
- Builds both as `cdylib` (wasm32-unknown-unknown) and `rlib` (native targets), so the whole surface is testable on any platform.
