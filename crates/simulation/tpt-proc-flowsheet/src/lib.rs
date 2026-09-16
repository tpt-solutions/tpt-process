//! Sequential-modular flowsheet solver with tear streams and Wegstein
//! acceleration (RFC 0003).
//!
//! Units execute in topological order over the torn process graph; recycle
//! (tear) streams converge by accelerated fixed-point iteration between
//! passes.
//!
//! # Example
//!
//! ```
//! use std::sync::Arc;
//! use tpt_proc_core::{Flowsheet, UnitId};
//! use tpt_proc_units::{Mixer, Splitter, UnitRegistry};
//! use tpt_proc_flowsheet::FlowsheetSolver;
//!
//! let flowsheet = Flowsheet::new(0, "loop");
//! let mut registry = UnitRegistry::new();
//! registry.register(UnitId(1), Arc::new(Mixer));
//! registry.register(UnitId(2), Arc::new(Splitter::new(vec![0.8, 0.2])));
//!
//! let solver = FlowsheetSolver::new(flowsheet, registry);
//! let result = solver.run(1e-9, 200).unwrap();
//! println!("converged: {}", result.converged);
//! ```

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use tpt_proc_core::{FlowRate, Flowsheet, MaterialStream, PortId, StreamId, UnitId};
use tpt_proc_topology::ProcessGraph;
use tpt_proc_units::UnitRegistry;

/// Solver outcomes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlowsheetResult {
    /// Tear-stream convergence achieved.
    pub converged: bool,
    /// Passes executed.
    pub iterations: u32,
    /// Last relative change on the tear streams.
    pub tear_residual: f64,
    /// Final tear-stream molar/mass flows (diagnostics).
    pub tear_values: BTreeMap<StreamId, f64>,
}

/// Errors raised while executing a flowsheet.
#[derive(Clone, PartialEq, Debug)]
pub enum FlowsheetError {
    /// The flowsheet failed structural validation.
    Invalid(String),
    /// A unit behavior failed during execution.
    Unit(tpt_proc_units::UnitError),
    /// A unit lacks inlets required by its connections.
    MissingInlet(UnitId),
    /// A unit id has no registered behavior.
    NoBehavior(UnitId),
}

impl std::fmt::Display for FlowsheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(m) => write!(f, "invalid flowsheet: {m}"),
            Self::Unit(e) => write!(f, "unit failure: {e}"),
            Self::MissingInlet(id) => write!(f, "unit {id} missing an inlet stream"),
            Self::NoBehavior(id) => write!(f, "unit {id} has no registered behavior"),
        }
    }
}

impl std::error::Error for FlowsheetError {}

/// The sequential-modular solver.
pub struct FlowsheetSolver {
    flowsheet: Flowsheet,
    registry: UnitRegistry,
}

/// Flow value of a stream for convergence bookkeeping.
fn flow_of(stream: &MaterialStream) -> f64 {
    stream.total_flow().unwrap_or(0.0)
}

/// Rescales a stream flow by a factor, preserving T/P/composition.
fn scaled(stream: &MaterialStream, factor: f64) -> MaterialStream {
    match stream.flow_rate() {
        Some(FlowRate::Mass(f)) => return with_flow_or_clone(stream, FlowRate::Mass(f * factor)),
        Some(FlowRate::Molar(f)) => return with_flow_or_clone(stream, FlowRate::Molar(f * factor)),
        Some(FlowRate::Volumetric(f)) => {
            return with_flow_or_clone(stream, FlowRate::Volumetric(f * factor))
        }
        None => {}
    }
    stream.clone()
}

fn with_flow_or_clone(stream: &MaterialStream, flow: FlowRate) -> MaterialStream {
    match stream.clone().with_flow(flow) {
        Ok(s) => s,
        Err(_) => stream.clone(),
    }
}

impl FlowsheetSolver {
    /// Creates a solver over a flowsheet and its unit behaviors.
    #[must_use]
    pub fn new(flowsheet: Flowsheet, registry: UnitRegistry) -> Self {
        Self {
            flowsheet,
            registry,
        }
    }

    /// Runs the sequential-modular passes until the tear streams converge.
    ///
    /// # Errors
    /// [`FlowsheetError::Invalid`] for structural problems; unit errors
    /// propagate as [`FlowsheetError::Unit`].
    pub fn run(
        self,
        tolerance: f64,
        max_iterations: u32,
    ) -> Result<FlowsheetResult, FlowsheetError> {
        self.flowsheet
            .validate()
            .map_err(|e| FlowsheetError::Invalid(e.to_string()))?;
        let graph = ProcessGraph::from_flowsheet(&self.flowsheet);
        let tears = graph.tear_streams();

        // Execution order over the torn graph.
        let mut torn = graph.clone();
        for t in &tears {
            torn = torn.without_stream(*t);
        }
        let order: Vec<UnitId> = torn
            .topological_order()
            .or_else(|| graph.topological_order())
            .unwrap_or_else(|| graph.units().collect());

        // Working stream values: boundary feeds keep their state; others
        // start as zero-flow placeholders.
        let mut streams: BTreeMap<StreamId, MaterialStream> = BTreeMap::new();
        for s in self.flowsheet.streams() {
            streams.insert(s.id(), s.clone());
        }

        let mut converged = false;
        let mut tear_residual = f64::INFINITY;
        let mut iterations = 0;
        let mut previous_tear: BTreeMap<StreamId, f64> = BTreeMap::new();

        for pass in 0..max_iterations {
            iterations = pass + 1;
            // Snapshot tear flows at pass start.
            let start_tear: BTreeMap<StreamId, f64> = tears
                .iter()
                .filter_map(|t| streams.get(t).map(|s| (*t, flow_of(s))))
                .collect();

            // Gauss-Seidel pass in execution order.
            for unit_id in &order {
                let Some(unit_op) = self.flowsheet.unit(*unit_id) else {
                    continue;
                };
                let mut inlets: Vec<MaterialStream> = Vec::new();
                for port in 0..unit_op.num_inlets() {
                    let port_id = PortId(port as u64);
                    let stream_id = self
                        .flowsheet
                        .connections()
                        .iter()
                        .find(|c| c.to == Some((*unit_id, port_id)))
                        .map(|c| c.stream)
                        .ok_or(FlowsheetError::MissingInlet(*unit_id))?;
                    let value = streams
                        .get(&stream_id)
                        .cloned()
                        .ok_or(FlowsheetError::MissingInlet(*unit_id))?;
                    inlets.push(value);
                }
                let behavior = self
                    .registry
                    .get(*unit_id)
                    .ok_or(FlowsheetError::NoBehavior(*unit_id))?;
                let outlets = behavior.solve(&inlets).map_err(FlowsheetError::Unit)?;
                for (port, outlet) in outlets.iter().enumerate() {
                    let port_id = PortId(port as u64);
                    for c in self.flowsheet.connections() {
                        if c.from == Some((*unit_id, port_id)) {
                            // Relabel the computed outlet to the connection
                            // stream identity so synthetic unit ids never
                            // feed back through recycle loops.
                            let relabeled = outlet.clone().with_id(c.stream);
                            streams.insert(c.stream, relabeled);
                        }
                    }
                }
            }

            // Tear convergence with bounded Wegstein acceleration.
            tear_residual = 0.0;
            let mut accelerated: Vec<(StreamId, f64)> = Vec::new();
            for tear in &tears {
                let Some(&start) = start_tear.get(tear) else {
                    continue;
                };
                let direct = streams.get(tear).map(flow_of).unwrap_or(0.0);
                let change = direct - start;
                tear_residual = tear_residual.max(change.abs() / start.abs().max(1e-9));
                if pass == 0 || change.abs() < 1e-15 {
                    previous_tear.insert(*tear, direct);
                    continue;
                }
                let s = change / (previous_tear.get(tear).map(|p| start - p).unwrap_or(change));
                let q = (s / (s - 1.0)).clamp(-5.0, 1.0);
                let value = q * direct + (1.0 - q) * start;
                previous_tear.insert(*tear, value);
                accelerated.push((*tear, value));
            }
            for (tear, value) in accelerated {
                if let Some(stream) = streams.get_mut(&tear) {
                    let current = flow_of(stream);
                    if current.abs() > 1e-12 {
                        let factor = value / current;
                        if factor.is_finite() && factor >= 0.0 {
                            *stream = scaled(stream, factor);
                        }
                    }
                }
            }

            if tear_residual < tolerance {
                converged = true;
                break;
            }
        }

        let tear_values: BTreeMap<StreamId, f64> = tears
            .iter()
            .filter_map(|t| streams.get(t).map(|s| (*t, flow_of(s))))
            .collect();
        Ok(FlowsheetResult {
            converged,
            iterations,
            tear_residual,
            tear_values,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tpt_proc_core::{Composition, FlowRate, PhaseState, PortId};
    use tpt_proc_units::{Mixer, Splitter};

    /// Mixer → splitter with recycle: fresh 100 mol/s, splitter [0.8, 0.2].
    /// Steady state: mixer out M = 100 + 0.2·M → M = 125; product = 100.
    fn recycle_flowsheet() -> Flowsheet {
        let mut fs = Flowsheet::new(0, "recycle loop");
        let mk = |id: u64, name: &str, flow: f64| {
            MaterialStream::new(id, name)
                .with_state(300.0, 101_325.0)
                .unwrap()
                .with_flow(FlowRate::Molar(flow))
                .unwrap()
                .with_composition(Composition::from_mole_fractions(&[1.0]).unwrap())
                .with_phase(PhaseState::Liquid)
        };
        fs.add_stream(mk(1, "fresh", 100.0));
        fs.add_stream(mk(2, "mixed", 0.0));
        fs.add_stream(mk(3, "product", 0.0));
        fs.add_stream(mk(4, "recycle", 0.0));
        fs.add_unit(tpt_proc_core::UnitOperation::Mixer {
            id: UnitId(1),
            inlets: 2,
        });
        fs.add_unit(tpt_proc_core::UnitOperation::Splitter {
            id: UnitId(2),
            split_ratios: vec![0.8, 0.2],
        });
        fs.feed(UnitId(1), PortId(0), StreamId(1)).unwrap();
        fs.connect(UnitId(2), PortId(1), UnitId(1), PortId(1), StreamId(4))
            .unwrap();
        fs.connect(UnitId(1), PortId(0), UnitId(2), PortId(0), StreamId(2))
            .unwrap();
        fs.withdraw(UnitId(2), PortId(0), StreamId(3)).unwrap();
        fs
    }

    #[test]
    fn recycle_loop_converges_to_analytic_solution() {
        let mut registry = UnitRegistry::new();
        registry.register(UnitId(1), Arc::new(Mixer));
        registry.register(UnitId(2), Arc::new(Splitter::new(vec![0.8, 0.2])));
        let solver = FlowsheetSolver::new(recycle_flowsheet(), registry);
        let result = solver.run(1e-10, 500).unwrap();
        assert!(result.converged, "residual {}", result.tear_residual);
        // The tear stream is the mixed line (tie-break prefers the lowest
        // stream id): steady state M = 100 + 0.2·M → M = 125 mol/s.
        let mixed = result.tear_values[&StreamId(2)];
        assert!(
            (mixed - 125.0).abs() < 1e-2,
            "mixed = {mixed}, expected 125"
        );
    }

    #[test]
    fn missing_behavior_is_reported() {
        let registry = UnitRegistry::new();
        let solver = FlowsheetSolver::new(recycle_flowsheet(), registry);
        assert!(matches!(
            solver.run(1e-9, 5),
            Err(FlowsheetError::NoBehavior(_))
        ));
    }

    #[test]
    fn acyclic_flowsheet_converges_immediately() {
        let mut fs = Flowsheet::new(1, "chain");
        let mk = |id: u64, flow: f64| {
            MaterialStream::new(id, format!("s{id}"))
                .with_state(300.0, 101_325.0)
                .unwrap()
                .with_flow(FlowRate::Molar(flow))
                .unwrap()
                .with_composition(Composition::from_mole_fractions(&[1.0]).unwrap())
        };
        fs.add_stream(mk(1, 80.0));
        fs.add_stream(mk(2, 0.0));
        fs.add_unit(tpt_proc_core::UnitOperation::Splitter {
            id: UnitId(0),
            split_ratios: vec![0.5, 0.5],
        });
        fs.feed(UnitId(0), PortId(0), StreamId(1)).unwrap();
        fs.withdraw(UnitId(0), PortId(0), StreamId(2)).unwrap();
        let mut registry = UnitRegistry::new();
        registry.register(UnitId(0), Arc::new(Splitter::new(vec![0.5, 0.5])));
        let result = FlowsheetSolver::new(fs, registry).run(1e-9, 50).unwrap();
        assert!(result.converged);
        assert!(result.tear_values.is_empty());
    }
}
