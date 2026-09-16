# RFC 0004 — Pinch Analysis and Heat Exchanger Network Synthesis

- **Status:** Accepted
- **Start date:** 2026-03
- **Crates:** `tpt-proc-heat-network` (synthesis), `tpt-proc-heat-exchangers`
  (consumers)

## Summary

Implement pinch analysis per the problem table algorithm (Linnhoff/
March): utility targets (Q_H,min, Q_C,min), pinch location, composite and
grand composite curves, and a pinch-design-method HEN synthesizer producing
a candidate network respecting ΔT_min.

## Motivation

Heat integration is the highest-value analysis in process engineering —
energy targets before any equipment is designed. The TPT ecosystem also
needs it as the process-side half of the `tpt-energy` connector: pinch
targets bound the utility system optimization. The algorithms are exact
(they are discrete sorting/network problems), so results can be verified
to machine precision against published benchmark problems.

## Detailed design

### Problem inputs

```rust
pub struct ProcessStream {
    pub supply_temp: f64,        // K (shifted internally)
    pub target_temp: f64,        // K
    pub heat_capacity_flow: f64, // W/K  (m·cp, constant per interval)
    pub is_hot: bool,
}
pub struct PinchAnalysis { hot: Vec<ProcessStream>, cold: Vec<ProcessStream>,
                           delta_t_min: f64 }
```

Streams may be piecewise-linear (multiple segments with different cp);
the API accepts a segment list and flattens it.

### Problem table algorithm

1. Shift temperatures: hot T′ = T − ΔT_min/2, cold T′ = T + ΔT_min/2.
2. Build sorted interval boundaries; per interval compute ΔH = ΔT·Σ(cp_in −
   cp_out) as net heat cascade.
3. Cascade with zero cold utility; negative cascade entries identify the
   minimum → add |min| as hot utility; the zero-crossing is the pinch.
4. Outputs: `UtilityTargets { min_heating_duty, min_cooling_duty,
   pinch_hot_temp, pinch_cold_temp }`, hot/cold composite curves as
   (T, H) polyline points, and the grand composite curve.

### Network synthesis (pinch design method)

- Split at the pinch into above (heat sink) and below (heat source)
  sub-problems.
- Above pinch: match streams honoring cp-rules (CPh ≤ CPc for matches
  adjacent to the pinch); place minimum-count matches: N_matches ≤ N_hot +
  N_cold; utility heaters only above, coolers only below.
- Below pinch: mirrored cp-rules.
- Merge sub-networks; residual heat crossing the pinch is not permitted
  (asserted); report network with heat loads per exchanger.
- The result is a *feasible minimum-utility* network, not the global
  minimum-area network: area/cost optimization is `tpt-proc-optimization`
  territory, out of scope here.

### Public API

```rust
impl PinchAnalysis {
    pub fn new(hot: Vec<ProcessStream>, cold: Vec<ProcessStream>,
               delta_t_min: f64) -> Self;
    pub fn minimum_utilities(&self) -> UtilityTargets;
    pub fn pinch_temperature(&self) -> Option<PinchPoint>;
    pub fn composite_curves(&self) -> CompositeCurves;   // plottable polylines
    pub fn grand_composite_curve(&self) -> GrandCompositeCurve;
    pub fn network_synthesis(&self) -> Result<HeatExchangerNetwork>;
}
```

## Verification & validation

- The classic four-stream problem (hot: 160→45 °C/15 kW/K, 220→60 °C/25
  kW/K? per published example set) with ΔT_min = 10 K — targets must match
  the published Q_H,min / Q_C,min / pinch exactly; encoded in
  `test-data/golden/heat-transfer/pinch-analysis.json`.
- Energy conservation: Q_H,min − Q_C,min = Σ_hot − Σ_cold (first-law
  identity) to 1e-9 for randomized stream sets.
- Synthesized networks are re-rated with `tpt-proc-heat-exchangers` to
  confirm each match meets its duty within tolerance and no match violates
  ΔT_min.

## Alternatives considered

- **Superstructure MINLP synthesis (Yee-Grossmann):** can beat the pinch
  design on total cost, requires MILP/MINLP machinery from
  `tpt-math-optimize-general`; future RFC.
- **Mathematical targeting for area/units (Ahmad-Smith):** useful targets
  layer on top of composite curves; can be added without API change.

## Unresolved questions

- Variable cp with phase change (zigzag streams) — needs segment
  decomposition of latent sections; planned post-Phase 5.
- Multiple utilities (steam levels): GCC-based utility placement is a
  natural extension of the grand composite; API reserved.
