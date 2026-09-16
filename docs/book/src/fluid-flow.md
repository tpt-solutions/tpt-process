# Fluid Flow

- `tpt-proc-fluid`: Reynolds number, laminar (64/Re) and Colebrook-White
  friction factors (Newton iteration from a Swamee-Jain start),
  Darcy-Weisbach pressure drop, minor losses.
- `tpt-proc-pumps`: quadratic curves, operating point by bisection,
  affinity laws, NPSH.
- `tpt-proc-compressors`: isentropic and polytropic head/work and
  discharge temperature.
- `tpt-proc-valves`: Cv sizing for liquids and subcritical gases,
  inherent characteristics.
- `tpt-proc-network`: three solvers for the quadratic head-loss law —
  Hardy Cross (cycle basis + pseudo cycles for multiple reservoirs),
  nodal Newton-Raphson, and Linear Theory. All three agree on test grids.
