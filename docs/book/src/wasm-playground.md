# WASM Playground

`tpt-proc-wasm` (in `crates/core/tpt-proc-wasm`) is the JS boundary of
the stack: a thin `#[wasm_bindgen]` facade over the flash and
heat-exchanger solvers, built for `wasm32-unknown-unknown`. The crate
also compiles natively (bindings become plain Rust types), so the
workspace tests it on any platform.

## What is exported

| Type | Members | Notes |
|---|---|---|
| `WasmFlashCalculator` | `new(components: &str)`, `component_names`, `pt_flash(z, T, P)`, `bubble_point(z, P)` | components are a whitespace-separated list resolved against the built-in database (`"water methanol"`); Peng-Robinson is attached automatically |
| `WasmFlashResult` | `vapor_fraction`, `temperature`, `pressure`, `iterations`, `degenerate`, `vapor_composition`, `liquid_composition` | getters surface as JS properties; compositions align with the component list |
| `WasmHeatExchanger` | `new(area, overall_u)`, `rate(c_hot, c_cold, hot_in, cold_in)` | counter-current ε-NTU rating |
| `WasmRatingResult` | `duty`, `hot_outlet`, `cold_outlet`, `effectiveness` | duty in W, temperatures in K |
| `lookup_component(name_or_cas)` | free function returning `Option<WasmComponent>` | `WasmComponent` exposes `name`, `cas`, `critical_temperature`, `critical_pressure`, `acentric_factor` |

Call conventions from JS: `&[f64]` slices are passed as typed arrays
(`Float64Array`), and failures are thrown as `WasmError` objects
carrying a `kind` (`"validation"`, `"solver"`, `"lookup"`) and a
`message`. A pure-component flash returns `degenerate = true` rather
than pretending β is unique.

## Building the package

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web crates/core/tpt-proc-wasm
```

This produces `crates/core/tpt-proc-wasm/pkg/` with
`tpt_proc_wasm.js` and `tpt_proc_wasm_bg.wasm`. The crate docs show the
equivalent two-step route (`cargo build … --target
wasm32-unknown-unknown` followed by `wasm-bindgen … --out-dir pkg`).
There is no prebuilt package in the repository — if you skip this
step, the playground below says so instead of failing silently.

## The playground

`assets/flash-playground.html` is a self-contained page (inline CSS/JS,
no dependencies, no build step). It probes for `pkg/tpt_proc_wasm.js`
next to itself, loads it as an ES module with a graceful "build the
wasm package to enable the demo" fallback, and — once loaded — runs
real Peng-Robinson PT flashes: type a component list from the built-in
database, mole fractions, T and P, and read off β and the phase
compositions.

<iframe src="assets/flash-playground.html" style="width:100%;height:34rem;border:1px solid #888;border-radius:4px;" title="PT flash playground"></iframe>

Open [assets/flash-playground.html](assets/flash-playground.html) in its
own tab if the frame is cramped. Because ES modules do not load from
`file://` URLs, serve the rendered book over HTTP (any static file
server on `docs/book/book/` after `mdbook build`) and copy the pkg
output into the served `assets/pkg/` — the page's own fallback text
documents the exact commands.

The page is deliberately defensive: a missing module, a stale package
without the expected exports, an unknown component name, or a
validation error (fractions not summing to 1, non-positive T/P) all
resolve to a message, never a crashed frame.
