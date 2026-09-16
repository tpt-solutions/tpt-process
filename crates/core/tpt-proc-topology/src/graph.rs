//! Graph edge type and the iterative Tarjan SCC algorithm.

use std::collections::{BTreeMap, BTreeSet};

use tpt_proc_core::{StreamId, UnitId};

/// A directed material-stream edge between two units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    /// Source unit.
    pub from: UnitId,
    /// Destination unit.
    pub to: UnitId,
    /// The stream carrying the flow.
    pub stream: StreamId,
}

/// Iterative Tarjan strongly-connected-components state.
pub(super) struct TarjanState {
    adjacency: BTreeMap<UnitId, Vec<UnitId>>,
    index: BTreeMap<UnitId, usize>,
    lowlink: BTreeMap<UnitId, usize>,
    on_stack: BTreeSet<UnitId>,
    stack: Vec<UnitId>,
    next_index: usize,
    components: Vec<Vec<UnitId>>,
}

impl TarjanState {
    pub(super) fn new(adjacency: &BTreeMap<UnitId, Vec<UnitId>>) -> Self {
        Self {
            adjacency: adjacency.clone(),
            index: BTreeMap::new(),
            lowlink: BTreeMap::new(),
            on_stack: BTreeSet::new(),
            stack: Vec::new(),
            next_index: 0,
            components: Vec::new(),
        }
    }

    /// Runs the full DFS over all nodes; components are returned with their
    /// members sorted.
    pub(super) fn run(mut self) -> Vec<Vec<UnitId>> {
        for root in self.adjacency.keys().copied().collect::<Vec<_>>() {
            if !self.index.contains_key(&root) {
                self.strong_connect(root);
            }
        }
        let mut components = self.components;
        for component in &mut components {
            component.sort_unstable();
        }
        components.sort();
        components
    }

    /// Iterative depth-first search for one root.
    fn strong_connect(&mut self, root: UnitId) {
        // Frame: (node, position in successor list).
        let mut frames: Vec<(UnitId, usize)> = vec![(root, 0)];

        self.index.insert(root, self.next_index);
        self.lowlink.insert(root, self.next_index);
        self.next_index += 1;
        self.stack.push(root);
        self.on_stack.insert(root);

        while let Some(&mut (node, ref mut child_pos)) = frames.last_mut() {
            let successors = &self.adjacency[&node];
            if *child_pos < successors.len() {
                let successor = successors[*child_pos];
                *child_pos += 1;
                if !self.index.contains_key(&successor) {
                    self.index.insert(successor, self.next_index);
                    self.lowlink.insert(successor, self.next_index);
                    self.next_index += 1;
                    self.stack.push(successor);
                    self.on_stack.insert(successor);
                    frames.push((successor, 0));
                } else if self.on_stack.contains(&successor) {
                    let s_index = self.index[&successor];
                    let l = self.lowlink.get_mut(&node).expect("visited");
                    if s_index < *l {
                        *l = s_index;
                    }
                }
            } else {
                // All successors processed: pop the frame and propagate the
                // lowlink to the parent.
                frames.pop();
                if let Some(&(parent, _)) = frames.last() {
                    let l = self.lowlink[&node];
                    let parent_low = self.lowlink.get_mut(&parent).expect("visited");
                    if l < *parent_low {
                        *parent_low = l;
                    }
                }
                if self.lowlink[&node] == self.index[&node] {
                    let mut component = Vec::new();
                    while let Some(top) = self.stack.pop() {
                        self.on_stack.remove(&top);
                        component.push(top);
                        if top == node {
                            break;
                        }
                    }
                    self.components.push(component);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(stream: u64, from: u64, to: u64) -> Edge {
        Edge {
            from: UnitId(from),
            to: UnitId(to),
            stream: StreamId(stream),
        }
    }

    fn graph(edges: &[Edge]) -> TarjanState {
        let mut adj: BTreeMap<UnitId, Vec<UnitId>> = BTreeMap::new();
        let mut units = BTreeSet::new();
        for e in edges {
            units.insert(e.from);
            units.insert(e.to);
            adj.entry(e.from).or_default().push(e.to);
        }
        for u in units {
            adj.entry(u).or_default();
        }
        TarjanState::new(&adj)
    }

    #[test]
    fn tarjan_finds_cycle() {
        let sccs = graph(&[edge(1, 1, 2), edge(2, 2, 3), edge(3, 3, 1)]).run();
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0], vec![UnitId(1), UnitId(2), UnitId(3)]);
    }

    #[test]
    fn tarjan_separates_acyclic() {
        let sccs = graph(&[edge(1, 1, 2), edge(2, 2, 3)]).run();
        assert_eq!(
            sccs,
            vec![vec![UnitId(1)], vec![UnitId(2)], vec![UnitId(3)]]
        );
    }

    #[test]
    fn tarjan_nested_and_branching() {
        // 1 -> 2 -> 3 -> 2 (cycle 2-3), 1 -> 4
        let sccs = graph(&[edge(1, 1, 2), edge(2, 2, 3), edge(3, 3, 2), edge(4, 1, 4)]).run();
        let mut cyclic: Vec<_> = sccs.iter().filter(|s| s.len() > 1).collect();
        cyclic.sort();
        assert_eq!(cyclic.len(), 1);
        assert_eq!(cyclic[0], &vec![UnitId(2), UnitId(3)]);
    }
}
