//! Process flow diagram graph analysis.
//!
//! [`ProcessGraph`] models a flowsheet as a directed multigraph of unit
//! operations connected by material streams. It answers the structural
//! questions a flowsheet solver needs before executing anything:
//!
//! - [`ProcessGraph::topological_order`] — execution order for acyclic
//!   flowsheets (sequential modular, no recycle);
//! - [`ProcessGraph::find_recycles`] — strongly connected components with
//!   internal cycles (recycle loops);
//! - [`ProcessGraph::tear_streams`] — a small deterministic set of streams
//!   whose removal breaks every recycle;
//! - [`ProcessGraph::connected_components`] — independent sections that can
//!   be solved (or parallelized) separately.
//!
//! All algorithms are iterative (no recursion), deterministic (ordered maps,
//! stable tie-breaks), and allocation-frugal.
//!
//! # Example
//!
//! ```
//! use tpt_proc_topology::ProcessGraph;
//!
//! // feed -> reactor -> separator, with unreacted feed recycled to the front
//! let text = r#"
//!     U1 -> U2 [S1]
//!     U2 -> U3 [S2]
//!     U3 -> U1 [S3]   # recycle
//! "#;
//! let graph = ProcessGraph::parse_pfd(text).unwrap();
//!
//! assert!(graph.topological_order().is_none());     // cycle present
//! assert_eq!(graph.find_recycles().len(), 1);        // one recycle loop
//! assert_eq!(graph.tear_streams().len(), 1);         // tear one stream
//! ```

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use tpt_proc_core::{CoreError, Flowsheet, Result, StreamId, UnitId};
mod graph;
mod parse;

use graph::TarjanState;

pub use graph::Edge;

/// A directed multigraph of unit operations (nodes) and material streams
/// (edges).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessGraph {
    units: BTreeSet<UnitId>,
    edges: Vec<Edge>,
}

impl ProcessGraph {
    /// An empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses PFD edge-list text (`U1 -> U2 [S1]` lines, `#` comments) into
    /// a graph. See the [`parse`](self) module docs for the format.
    ///
    /// # Errors
    /// [`CoreError::InvalidFlowsheet`] for malformed input.
    pub fn parse_pfd(text: &str) -> Result<Self> {
        parse::parse_pfd(text)
    }

    /// Builds the graph from a flowsheet: every connection whose *both*
    /// endpoints are units becomes an edge; boundary feeds/withdrawals are
    /// ignored (they are not part of the unit digraph).
    #[must_use]
    pub fn from_flowsheet(flowsheet: &Flowsheet) -> Self {
        let mut graph = Self::new();
        for unit in flowsheet.units() {
            graph.add_unit(unit.id());
        }
        for connection in flowsheet.connections() {
            if let (Some((from, _)), Some((to, _))) = (connection.from, connection.to) {
                if from != to {
                    graph.add_stream(connection.stream, from, to);
                }
            }
        }
        graph
    }

    /// Adds a unit node.
    pub fn add_unit(&mut self, unit: UnitId) {
        self.units.insert(unit);
    }

    /// Adds a directed stream edge `from → to` (both units are inserted if
    /// unknown).
    pub fn add_stream(&mut self, stream: StreamId, from: UnitId, to: UnitId) {
        self.units.insert(from);
        self.units.insert(to);
        self.edges.push(Edge { from, to, stream });
    }

    /// A copy of the graph with every edge carrying `stream` removed —
    /// the torn graph used by the flowsheet solver.
    #[must_use]
    pub fn without_stream(&self, stream: StreamId) -> ProcessGraph {
        let mut copy = self.clone();
        copy.edges.retain(|e| e.stream != stream);
        copy
    }

    /// All unit ids, ordered.
    pub fn units(&self) -> impl Iterator<Item = UnitId> + '_ {
        self.units.iter().copied()
    }

    /// All stream edges, in insertion order.
    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Adjacency list `unit → successors` (successors in insertion order,
    /// duplicates preserved for multigraph semantics).
    #[must_use]
    pub fn adjacency(&self) -> BTreeMap<UnitId, Vec<UnitId>> {
        let mut adj: BTreeMap<UnitId, Vec<UnitId>> = BTreeMap::new();
        for u in &self.units {
            adj.entry(*u).or_default();
        }
        for e in &self.edges {
            adj.entry(e.from).or_default().push(e.to);
        }
        adj
    }

    /// Predecessor count per unit.
    fn in_degrees(&self) -> BTreeMap<UnitId, usize> {
        let mut deg: BTreeMap<UnitId, usize> = BTreeMap::new();
        for u in &self.units {
            deg.entry(*u).or_default();
        }
        for e in &self.edges {
            *deg.entry(e.to).or_default() += 1;
        }
        deg
    }

    /// Topological execution order for acyclic graphs.
    ///
    /// Returns `None` if the graph contains a cycle (build a tear set with
    /// [`ProcessGraph::tear_streams`] first).
    #[must_use]
    pub fn topological_order(&self) -> Option<Vec<UnitId>> {
        let mut indegree = self.in_degrees();
        let adj = self.adjacency();
        let mut queue: VecDeque<UnitId> = indegree
            .iter()
            .filter(|(_, d)| **d == 0)
            .map(|(u, _)| *u)
            .collect();
        let mut order = Vec::with_capacity(self.units.len());
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for v in &adj[&u] {
                let d = indegree.get_mut(v).expect("successor is known");
                *d -= 1;
                if *d == 0 {
                    queue.push_back(*v);
                }
            }
        }
        if order.len() == self.units.len() {
            Some(order)
        } else {
            None
        }
    }

    /// Strongly connected components with more than one unit (or a unit
    /// with a stream back to itself): the recycle loops of the flowsheet.
    ///
    /// Components are ordered lexicographically for determinism.
    #[must_use]
    pub fn find_recycles(&self) -> Vec<Vec<UnitId>> {
        let state = TarjanState::new(&self.adjacency());
        let sccs = state.run();
        let mut recycles: Vec<Vec<UnitId>> = sccs
            .into_iter()
            .filter(|scc| {
                scc.len() > 1
                    || self
                        .edges
                        .iter()
                        .any(|e| e.from == e.to && e.from == scc[0])
            })
            .collect();
        recycles.sort();
        recycles
    }

    /// Independent sections of the flowsheet (connected components of the
    /// *undirected* stream structure), units ordered, sections ordered by
    /// smallest member.
    #[must_use]
    pub fn connected_components(&self) -> Vec<Vec<UnitId>> {
        let mut undirected: BTreeMap<UnitId, BTreeSet<UnitId>> = BTreeMap::new();
        for u in &self.units {
            undirected.entry(*u).or_default();
        }
        for e in &self.edges {
            undirected.entry(e.from).or_default().insert(e.to);
            undirected.entry(e.to).or_default().insert(e.from);
        }
        let mut visited: BTreeSet<UnitId> = BTreeSet::new();
        let mut components = Vec::new();
        for start in &self.units {
            if visited.contains(start) {
                continue;
            }
            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(*start);
            visited.insert(*start);
            while let Some(u) = queue.pop_front() {
                component.push(u);
                for v in &undirected[&u] {
                    if visited.insert(*v) {
                        queue.push_back(*v);
                    }
                }
            }
            component.sort_unstable();
            components.push(component);
        }
        components
    }

    /// Streams whose removal breaks every recycle loop, one per strongly
    /// connected cyclic component.
    ///
    /// Selecting the provably minimum tear set is NP-hard; this uses the
    /// standard degree heuristic — within each cyclic component, tear the
    /// internal stream whose source has the most in-component successors
    /// (breaking it disconnects the most units), tie-broken by the lowest
    /// stream id for determinism. Tear streams converge well under Wegstein
    /// acceleration in the flowsheet solver.
    #[must_use]
    pub fn tear_streams(&self) -> Vec<StreamId> {
        let state = TarjanState::new(&self.adjacency());
        let sccs = state.run();
        let mut tears = Vec::new();
        for scc in &sccs {
            let cyclic = scc.len() > 1
                || self
                    .edges
                    .iter()
                    .any(|e| e.from == e.to && e.from == scc[0]);
            if !cyclic {
                continue;
            }
            let members: BTreeSet<UnitId> = scc.iter().copied().collect();
            // In-component out-degree per candidate source.
            let mut out_degree: BTreeMap<UnitId, usize> = BTreeMap::new();
            for e in &self.edges {
                if members.contains(&e.from) && members.contains(&e.to) && e.from != e.to {
                    *out_degree.entry(e.from).or_default() += 1;
                }
            }
            let best = self
                .edges
                .iter()
                .filter(|e| members.contains(&e.from) && members.contains(&e.to) && e.from != e.to)
                .max_by(|a, b| {
                    let da = out_degree.get(&a.from).copied().unwrap_or(0);
                    let db = out_degree.get(&b.from).copied().unwrap_or(0);
                    da.cmp(&db).then_with(|| b.stream.cmp(&a.stream))
                });
            if let Some(e) = best {
                tears.push(e.stream);
            }
        }
        tears.sort();
        tears
    }
}

/// Errors surfaced by PFD parsing (re-exported [`CoreError`] variants).
pub type ParseError = CoreError;

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_proc_core::{MaterialStream, PortId};

    fn graph_from_text(text: &str) -> ProcessGraph {
        ProcessGraph::parse_pfd(text).expect("valid PFD text")
    }

    #[test]
    fn linear_chain_is_topologically_ordered() {
        let g = graph_from_text("U1 -> U2 [S1]\nU2 -> U3 [S2]");
        let order = g.topological_order().expect("acyclic");
        assert_eq!(order, vec![UnitId(1), UnitId(2), UnitId(3)]);
        assert!(g.find_recycles().is_empty());
        assert!(g.tear_streams().is_empty());
        assert_eq!(
            g.connected_components(),
            vec![vec![UnitId(1), UnitId(2), UnitId(3)]]
        );
    }

    #[test]
    fn recycle_detected_and_torn() {
        let g = graph_from_text("U1 -> U2 [S1]\nU2 -> U3 [S2]\nU3 -> U1 [S3]\nU3 -> U4 [S4]");
        assert!(g.topological_order().is_none());
        let recycles = g.find_recycles();
        assert_eq!(recycles.len(), 1);
        assert_eq!(recycles[0], vec![UnitId(1), UnitId(2), UnitId(3)]);
        let tears = g.tear_streams();
        assert_eq!(tears.len(), 1);
        // Tearing the selected stream restores acyclicity.
        let tear = tears[0];
        let mut torn = g.clone();
        torn.edges.retain(|e| e.stream != tear);
        assert!(torn.topological_order().is_some());
    }

    #[test]
    fn two_independent_recycles() {
        let g = graph_from_text("U1 -> U2 [S1]\nU2 -> U1 [S2]\nU3 -> U4 [S3]\nU4 -> U3 [S4]");
        assert_eq!(g.find_recycles().len(), 2);
        assert_eq!(g.tear_streams().len(), 2);
        // The two loops are not connected to each other.
        assert_eq!(g.connected_components().len(), 2);
    }

    #[test]
    fn connected_components_split() {
        let g = graph_from_text("U1 -> U2 [S1]\nU5 -> U6 [S2]");
        assert_eq!(
            g.connected_components(),
            vec![vec![UnitId(1), UnitId(2)], vec![UnitId(5), UnitId(6)]]
        );
    }

    #[test]
    fn from_flowsheet_ignores_boundary_streams() {
        let mut fs = Flowsheet::new(0, "pfd");
        for id in 1..=3 {
            fs.add_stream(MaterialStream::new(id, format!("s{id}")));
        }
        fs.add_unit(tpt_proc_core::UnitOperation::Mixer {
            id: UnitId(1),
            inlets: 1,
        });
        fs.add_unit(tpt_proc_core::UnitOperation::Flash { id: UnitId(2) });
        fs.add_unit(tpt_proc_core::UnitOperation::Mixer {
            id: UnitId(3),
            inlets: 1,
        });
        fs.feed(UnitId(1), PortId(0), StreamId(1)).unwrap();
        fs.connect(UnitId(1), PortId(0), UnitId(2), PortId(0), StreamId(2))
            .unwrap();
        fs.withdraw(UnitId(2), PortId(0), StreamId(3)).unwrap();

        let g = ProcessGraph::from_flowsheet(&fs);
        assert_eq!(g.units().count(), 3);
        assert_eq!(g.edges().len(), 1);
        // Mixer(1) feeds Flash(2); standalone Mixer(3) is a root too.
        let order = g.topological_order().expect("acyclic");
        assert_eq!(order.first(), Some(&UnitId(1)));
        assert_eq!(order.last(), Some(&UnitId(2)));
    }

    #[test]
    fn parse_errors() {
        assert!(ProcessGraph::parse_pfd("U1 -> [S1]").is_err());
        assert!(ProcessGraph::parse_pfd("U1 -> U1 [S1]").is_err());
        assert!(ProcessGraph::parse_pfd("garbage").is_err());
    }
}
