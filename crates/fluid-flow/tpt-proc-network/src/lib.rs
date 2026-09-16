//! Pipe network solvers: Hardy Cross, Newton-Raphson (nodal), Linear Theory.
//!
//! Head loss per pipe is the quadratic law `h = r·Q·|Q|` with resistance
//! `r = 8·f·L/(π²·g·D⁵)` (frozen friction factor — the classic textbook
//! formulation; re-evaluate `r` and re-solve for friction updates).
//!
//! All three solvers must agree; the tests cross-verify them, which is the
//! strongest available check short of a published benchmark.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// Parent map: node → (parent, tree pipe, direction sign).
type ParentMap = BTreeMap<u64, Option<(u64, usize, f64)>>;

/// One loop: (pipe index, traversal direction) pairs plus a pseudo head
/// constant.
type Cycle = (Vec<(usize, f64)>, f64);

/// A network node: either a fixed-head reservoir or a demand junction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetworkNode {
    /// Hydraulic grade-line elevation, m (reservoir water level).
    pub head: f64,
    /// Fixed-head boundary when true (head known, demand ignored).
    pub is_reservoir: bool,
    /// Demand drawn at the node, m³/s (positive = out of network).
    pub demand: f64,
}

/// A network pipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetworkPipe {
    /// Upstream node id.
    pub from: u64,
    /// Downstream node id.
    pub to: u64,
    /// Resistance r in h = r·Q|Q|, s²/m⁵.
    pub resistance: f64,
}

/// Resistance factor `r` for a pipe given its friction factor.
#[must_use]
pub fn pipe_resistance(friction: f64, length: f64, diameter: f64) -> f64 {
    8.0 * friction * length
        / (std::f64::consts::PI * std::f64::consts::PI * 9.80665 * diameter.powi(5))
}

/// A solved network.
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkSolution {
    /// Flow per pipe (indexed like the input pipes), m³/s (from→to
    /// positive).
    pub pipe_flows: Vec<f64>,
    /// Computed heads at junction nodes, m.
    pub junction_heads: BTreeMap<u64, f64>,
    /// Converged flag.
    pub converged: bool,
    /// Iterations used.
    pub iterations: u32,
}

/// A pipe network.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PipeNetwork {
    /// Nodes by id.
    pub nodes: BTreeMap<u64, NetworkNode>,
    /// Pipes in declaration order (their index identifies them).
    pub pipes: Vec<NetworkPipe>,
}

impl PipeNetwork {
    /// An empty network.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node (or replaces it).
    pub fn add_node(&mut self, id: u64, node: NetworkNode) {
        self.nodes.insert(id, node);
    }

    /// Adds a pipe; returns its index.
    pub fn add_pipe(&mut self, pipe: NetworkPipe) -> usize {
        self.pipes.push(pipe);
        self.pipes.len() - 1
    }

    /// Head loss of pipe `index` at flow `q`, m (signed with flow).
    #[must_use]
    pub fn head_loss(&self, index: usize, q: f64) -> f64 {
        let r = self.pipes[index].resistance;
        r * q * q.abs()
    }

    /// Initial flows satisfying junction continuity exactly: chords carry
    /// zero flow; each tree edge carries the net withdrawal of its child
    /// subtree, with reservoir supply guessed as an equal split of the
    /// total demand (negative demand).
    fn initial_flows(&self) -> Result<Vec<f64>, String> {
        let (parent, order) = self.spanning_tree()?;
        let total_demand: f64 = self
            .nodes
            .values()
            .filter(|n| !n.is_reservoir)
            .map(|n| n.demand)
            .sum();
        let reservoirs: usize = self.nodes.values().filter(|n| n.is_reservoir).count();
        let supply_per_reservoir = if reservoirs > 0 {
            -total_demand / reservoirs as f64
        } else {
            0.0
        };
        // Subtree net withdrawal in reverse BFS order.
        let mut subtree: BTreeMap<u64, f64> = BTreeMap::new();
        for id in order.iter().rev() {
            let node = self.nodes.get(id).ok_or("unknown node in tree")?;
            let own = if node.is_reservoir {
                supply_per_reservoir
            } else {
                node.demand
            };
            let children_net: f64 = parent
                .iter()
                .filter(|(_, p)| matches!(p, Some((ancestor, _, _)) if ancestor == id))
                .map(|(child, _)| subtree.get(child).copied().unwrap_or(0.0))
                .sum();
            subtree.insert(*id, own + children_net);
        }
        let mut flows = vec![0.0_f64; self.pipes.len()];
        // Chords (non-tree pipes) stay at zero; tree edges carry the child
        // subtree net: declared flow q = sign · net_child.
        for (child, link) in parent.iter() {
            if let Some((_, pipe_index, sign)) = link {
                flows[*pipe_index] = sign * subtree.get(child).copied().unwrap_or(0.0);
            }
        }
        Ok(flows)
    }

    /// BFS spanning tree rooted at the lowest reservoir id (so pseudo
    /// cycles between reservoirs share the root): parent map plus BFS
    /// visit order.
    fn spanning_tree(&self) -> Result<(ParentMap, Vec<u64>), String> {
        let mut adjacency: BTreeMap<u64, Vec<(u64, usize)>> = BTreeMap::new();
        for (i, pipe) in self.pipes.iter().enumerate() {
            adjacency.entry(pipe.from).or_default().push((pipe.to, i));
            adjacency.entry(pipe.to).or_default().push((pipe.from, i));
        }
        let root = self
            .nodes
            .iter()
            .filter(|(_, n)| n.is_reservoir)
            .map(|(id, _)| *id)
            .next()
            .or_else(|| self.nodes.keys().next().copied());
        let mut parent: BTreeMap<u64, Option<(u64, usize, f64)>> = BTreeMap::new();
        let mut order = Vec::new();
        if let Some(root) = root {
            parent.insert(root, None);
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(root);
            while let Some(node) = queue.pop_front() {
                order.push(node);
                if let Some(neighbors) = adjacency.get(&node) {
                    for (next, pipe_index) in neighbors {
                        if !parent.contains_key(next) {
                            let p = &self.pipes[*pipe_index];
                            let sign = if p.from == node && p.to == *next {
                                1.0
                            } else {
                                -1.0
                            };
                            parent.insert(*next, Some((node, *pipe_index, sign)));
                            queue.push_back(*next);
                        }
                    }
                }
            }
        }
        if order.len() != self.nodes.len() {
            return Err("graph is disconnected: some nodes unreachable".into());
        }
        Ok((parent, order))
    }

    /// Solves with the classic loop-correction (Hardy Cross) method over a
    /// cycle basis of the graph.
    ///
    /// # Errors
    /// Returns `Err` for malformed networks (unknown nodes, no cycles).
    pub fn solve_hardy_cross(
        &self,
        tolerance: f64,
        max_iterations: u32,
    ) -> Result<NetworkSolution, String> {
        let cycles = self.cycle_basis()?;
        let mut flows = self.initial_flows()?;
        let mut converged = false;
        let mut iterations = 0;

        'outer: for iteration in 0..max_iterations {
            iterations = iteration + 1;
            let mut max_correction = 0.0_f64;
            for (cycle, pseudo_head) in &cycles {
                // Σ ±h + ΔH_pseudo = 0 → ΔQ = −Σh/Σ(2r|Q|). The signed head
                // h = r·q·|q| uses the traversal flow q = direction·F; the
                // sign of q already encodes flow direction.
                let mut sum_h = *pseudo_head;
                let mut sum_dh = 0.0;
                for (pipe_index, direction) in cycle {
                    let q = direction * flows[*pipe_index];
                    let r = self.pipes[*pipe_index].resistance;
                    sum_h += r * q * q.abs();
                    sum_dh += 2.0 * r * q.abs();
                }
                let correction = if sum_dh > 0.0 { -sum_h / sum_dh } else { 0.0 };
                max_correction = max_correction.max(correction.abs());
                for (pipe_index, direction) in cycle {
                    // Loop corrections preserve junction continuity.
                    flows[*pipe_index] += direction * correction;
                }
            }
            if max_correction < tolerance {
                converged = true;
                break 'outer;
            }
        }
        let junction_heads = self.junction_heads_from_flows(&flows, tolerance)?;
        Ok(NetworkSolution {
            pipe_flows: flows,
            junction_heads,
            converged,
            iterations,
        })
    }

    /// Shared nodal solver. `method` selects the linearization:
    /// Newton-Raphson uses the full quadratic law and its tangent;
    /// Linear Theory uses the chord slope h ≈ (r·|Q₀|)·Q.
    fn solve_nodal(
        &self,
        tolerance: f64,
        max_iterations: u32,
        linear_theory: bool,
    ) -> Result<NetworkSolution, String> {
        let junctions: Vec<u64> = self
            .nodes
            .iter()
            .filter(|(_, n)| !n.is_reservoir)
            .map(|(id, _)| *id)
            .collect();
        if junctions.is_empty() {
            // All nodes are fixed-head boundaries: flows follow directly.
            let heads = BTreeMap::new();
            let flows = self.flows_from_heads(&heads)?;
            return Ok(NetworkSolution {
                pipe_flows: flows,
                junction_heads: heads,
                converged: true,
                iterations: 0,
            });
        }
        let index_of = |id: u64| -> Option<usize> { junctions.iter().position(|j| *j == id) };
        let max_head = self
            .nodes
            .values()
            .map(|n| n.head)
            .fold(f64::NEG_INFINITY, f64::max);
        // Seed junction heads hydraulically: propagate from reservoirs
        // along the continuity-consistent initial flows, so the first
        // linearization sees realistic head differences.
        let mut heads: BTreeMap<u64, f64> =
            self.junction_heads_from_flows(&self.initial_flows()?, tolerance)?;
        for id in &junctions {
            heads.entry(*id).or_insert(max_head - 5.0);
        }

        let mut converged = false;
        let mut iterations = 0;
        for iteration in 0..max_iterations {
            iterations = iteration + 1;
            let mut residuals = vec![0.0_f64; junctions.len()];
            let mut jacobian = vec![vec![0.0_f64; junctions.len()]; junctions.len()];

            for pipe in &self.pipes {
                let h_from = self.head_of(pipe.from, &heads)?;
                let h_to = self.head_of(pipe.to, &heads)?;
                let dh = h_from - h_to;
                let r = pipe.resistance;
                let adh = dh.abs().max(1e-9);
                let q = sign(dh) * (adh / r).sqrt();
                // Newton tangent: dQ/d|dh| = 1/(2√(r·|dh|));
                // Linear theory chord: dQ/d|dh| = 1/√(r·|dh|).
                let half = if linear_theory { 1.0 } else { 2.0 };
                let slope = 1.0 / (half * (r * adh).sqrt());
                // Residual R_j = inflow − outflow − demand; Newton tangent
                // c = ∂Q/∂Δh = 1/(2√(r·|Δh|)) > 0:
                //   row(from): R −= Q → ∂R/∂h_from = −c, ∂R/∂h_to = +c
                //   row(to):   R += Q → ∂R/∂h_from = +c, ∂R/∂h_to = −c
                if let Some(pos) = index_of(pipe.to) {
                    residuals[pos] += q;
                    jacobian[pos][pos] -= slope;
                }
                if let Some(pos) = index_of(pipe.from) {
                    residuals[pos] -= q;
                    jacobian[pos][pos] -= slope;
                }
                if let (Some(pf), Some(pt)) = (index_of(pipe.from), index_of(pipe.to)) {
                    jacobian[pf][pt] += slope;
                    jacobian[pt][pf] += slope;
                }
            }
            for (j, jid) in junctions.iter().enumerate() {
                residuals[j] -= self.nodes[jid].demand;
            }

            let max_residual = residuals.iter().fold(0.0_f64, |m, r| m.max(r.abs()));
            if max_residual < tolerance {
                converged = true;
                break;
            }

            if linear_theory {
                // Linear theory: assemble the (positive) Laplacian with the
                // chord slopes and solve M·h = demand outright. Chord slope
                // Q ≈ dh/√(r·|dh₀|).
                let n = junctions.len();
                let mut m = vec![vec![0.0_f64; n]; n];
                let mut rhs = vec![0.0_f64; n];
                for pipe in &self.pipes {
                    let h_from = self.head_of(pipe.from, &heads)?;
                    let h_to = self.head_of(pipe.to, &heads)?;
                    let slope = 1.0 / (pipe.resistance * (h_from - h_to).abs().max(1e-9)).sqrt();
                    if let (Some(pf), Some(pt)) = (index_of(pipe.from), index_of(pipe.to)) {
                        m[pf][pf] += slope;
                        m[pt][pt] += slope;
                        m[pf][pt] -= slope;
                        m[pt][pf] -= slope;
                    } else if let Some(pt) = index_of(pipe.to) {
                        // Feeder from a fixed-head reservoir: known-side term
                        // moves to the right-hand side.
                        m[pt][pt] += slope;
                        rhs[pt] += slope * h_from;
                    } else if let Some(pf) = index_of(pipe.from) {
                        m[pf][pf] += slope;
                        rhs[pf] += slope * h_to;
                    }
                }
                for (j, jid) in junctions.iter().enumerate() {
                    // Positive-diagonal Laplacian: demands enter negatively.
                    rhs[j] -= self.nodes[jid].demand;
                }
                if let Some(h) = dense_solve(&mut m, &mut rhs) {
                    for (j, jid) in junctions.iter().enumerate() {
                        let new_head = heads[jid] + (h[j] - heads[jid]).clamp(-10.0, 10.0);
                        heads.insert(*jid, new_head);
                    }
                } else {
                    return Err("singular system in linear theory".into());
                }
            } else {
                // Newton: J·Δh = −R with h ← h + Δh.
                let mut a = jacobian.clone();
                let mut b: Vec<f64> = residuals.iter().map(|r| -r).collect();
                if let Some(dh) = dense_solve(&mut a, &mut b) {
                    for (j, jid) in junctions.iter().enumerate() {
                        heads.insert(*jid, heads[jid] + dh[j].clamp(-10.0, 10.0));
                    }
                } else {
                    return Err("singular Jacobian: network topology degenerate".into());
                }
            }
        }

        let flows = self.flows_from_heads(&heads)?;
        Ok(NetworkSolution {
            pipe_flows: flows,
            junction_heads: heads,
            converged,
            iterations,
        })
    }

    /// Solves with the global Newton-Raphson nodal method: unknowns are
    /// junction heads, residuals are continuity errors.
    pub fn solve_newton_raphson(
        &self,
        tolerance: f64,
        max_iterations: u32,
    ) -> Result<NetworkSolution, String> {
        self.solve_nodal(tolerance, max_iterations, false)
    }

    /// Solves with the Linear Theory (Wood): the head-loss law is
    /// linearized as h ≈ (r·|Q₀|)·Q and the nodal system is re-solved each
    /// pass until the flows stabilize.
    pub fn solve_linear_theory(
        &self,
        tolerance: f64,
        max_iterations: u32,
    ) -> Result<NetworkSolution, String> {
        self.solve_nodal(tolerance, max_iterations, true)
    }

    fn head_of(&self, node: u64, heads: &BTreeMap<u64, f64>) -> Result<f64, String> {
        if let Some(n) = self.nodes.get(&node) {
            if n.is_reservoir {
                return Ok(n.head);
            }
        }
        heads
            .get(&node)
            .copied()
            .ok_or_else(|| format!("node {node} unknown or uninitialized"))
    }

    fn flows_from_heads(&self, heads: &BTreeMap<u64, f64>) -> Result<Vec<f64>, String> {
        self.pipes
            .iter()
            .map(|pipe| {
                let dh = self.head_of(pipe.from, heads)? - self.head_of(pipe.to, heads)?;
                Ok(sign(dh) * (dh.abs() / pipe.resistance).sqrt())
            })
            .collect()
    }

    fn junction_heads_from_flows(
        &self,
        flows: &[f64],
        _tolerance: f64,
    ) -> Result<BTreeMap<u64, f64>, String> {
        // Propagate heads from reservoirs along the tree of positive
        // pressure paths (approximate reporting for the loop method).
        let mut heads = BTreeMap::new();
        for (id, node) in &self.nodes {
            if node.is_reservoir {
                heads.insert(*id, node.head);
            }
        }
        // For each pipe, record the head at the downstream end when known.
        for _ in 0..self.nodes.len() {
            let mut changed = false;
            for (pipe, flow) in self.pipes.iter().zip(flows) {
                let flow = *flow;
                let h_from_known = heads.contains_key(&pipe.from);
                let h_to_known = heads.contains_key(&pipe.to);
                if flow >= 0.0 && h_from_known && !h_to_known {
                    heads.insert(
                        pipe.to,
                        heads[&pipe.from] - self.head_loss_index(pipe, flow),
                    );
                    changed = true;
                } else if flow < 0.0 && h_to_known && !h_from_known {
                    heads.insert(
                        pipe.from,
                        heads[&pipe.to] - self.head_loss_index(pipe, -flow),
                    );
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        Ok(heads)
    }

    fn head_loss_index(&self, pipe: &NetworkPipe, flow: f64) -> f64 {
        let r = pipe.resistance;
        r * flow * flow.abs()
    }

    /// Cycle basis from a spanning tree (one independent loop per non-tree
    /// edge), plus pseudo cycles linking each additional fixed-grade node
    /// to the root reservoir. Each entry is `(edges, pseudo_head)`; the
    /// loop residual is Σ ±h over edges plus `pseudo_head` (zero for graph
    /// cycles, the head difference H_root − H_reservoir for pseudo cycles
    /// — without them, networks with several reservoirs are
    /// under-determined).
    fn cycle_basis(&self) -> Result<Vec<Cycle>, String> {
        for pipe in &self.pipes {
            if !self.nodes.contains_key(&pipe.from) || !self.nodes.contains_key(&pipe.to) {
                return Err(format!(
                    "pipe {}→{} references unknown nodes",
                    pipe.from, pipe.to
                ));
            }
        }
        let (parent, _order) = self.spanning_tree()?;
        let root = parent
            .iter()
            .find(|(_, p)| p.is_none())
            .map(|(id, _)| *id)
            .ok_or("network has no nodes")?;
        let root_head = self.nodes.get(&root).map(|n| n.head).ok_or("root node")?;
        let mut visited = vec![false; self.pipes.len()];
        for (_, pipe_index, _) in parent.values().flatten() {
            visited[*pipe_index] = true;
        }
        // Each non-tree edge closes one independent cycle.
        let mut cycles: Vec<Cycle> = Vec::new();
        for (i, pipe) in self.pipes.iter().enumerate() {
            if visited[i] {
                continue;
            }
            // Cycle: from →(non-tree edge, +1)→ to →(up-tree)→ LCA
            // →(down-tree)→ from.
            let (path_a, root_a) = Self::path_to_root(&parent, pipe.from);
            let (path_b, root_b) = Self::path_to_root(&parent, pipe.to);
            if root_a != root_b {
                return Err("graph is disconnected: cycle edge spans two trees".into());
            }
            let ancestors_a: Vec<u64> = std::iter::once(pipe.from)
                .chain(path_a.iter().map(|(node, _, _)| *node))
                .chain(std::iter::once(root_a))
                .collect();
            let ancestors_b: std::collections::BTreeSet<u64> = std::iter::once(pipe.to)
                .chain(path_b.iter().map(|(node, _, _)| *node))
                .chain(std::iter::once(root_b))
                .collect();
            let lca = ancestors_a
                .iter()
                .find(|n| ancestors_b.contains(n))
                .copied()
                .ok_or("graph is disconnected: no common ancestor")?;

            let mut cycle: Vec<(usize, f64)> = vec![(i, 1.0)];
            // to → LCA: tree edges traversed child→parent.
            for (step_node, pipe_index, sign) in &path_b {
                if *step_node == lca {
                    break;
                }
                cycle.push((*pipe_index, -sign));
            }
            // LCA → from: reverse of from → LCA, sign-flipped.
            let mut up_from: Vec<(usize, f64)> = Vec::new();
            for (step_node, pipe_index, sign) in &path_a {
                if *step_node == lca {
                    break;
                }
                up_from.push((*pipe_index, -sign));
            }
            up_from.reverse();
            for (pipe_index, direction) in up_from {
                cycle.push((pipe_index, -direction));
            }
            cycles.push((cycle, 0.0));
            visited[i] = true;
        }
        // Pseudo cycles: one per additional fixed-grade node. Walking from
        // the reservoir up the tree to the root must lose exactly
        // H_root − H_reservoir of head.
        let reservoirs: Vec<u64> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.is_reservoir)
            .map(|(id, _)| *id)
            .collect();
        for reservoir in &reservoirs {
            if *reservoir == root {
                continue;
            }
            let (path, _) = Self::path_to_root(&parent, *reservoir);
            let cycle: Vec<(usize, f64)> = path
                .iter()
                .map(|(_, pipe_index, sign)| (*pipe_index, -sign))
                .collect();
            let h_res = self.nodes.get(reservoir).map(|n| n.head).unwrap_or(0.0);
            cycles.push((cycle, root_head - h_res));
        }
        if cycles.is_empty() {
            return Err("network has no loops: Hardy Cross needs at least one cycle".into());
        }
        Ok(cycles)
    }

    /// Path from `node` up to the tree root: entries
    /// `(child_reached, pipe_index, sign)` where sign means parent→child
    /// follows (+1) or opposes (−1) the pipe's declared direction. Also
    /// returns the root id.
    fn path_to_root(
        parent: &BTreeMap<u64, Option<(u64, usize, f64)>>,
        node: u64,
    ) -> (Vec<(u64, usize, f64)>, u64) {
        let mut path = Vec::new();
        let mut current = node;
        while let Some(Some((ancestor, pipe_index, sign))) = parent.get(&current) {
            path.push((current, *pipe_index, *sign));
            current = *ancestor;
        }
        (path, current)
    }
}

fn sign(v: f64) -> f64 {
    if v >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// Dense linear solve by Gaussian elimination with partial pivoting.
/// Destroys `a` and `b`; returns `None` for singular systems.
fn dense_solve(a: &mut [Vec<f64>], b: &mut [f64]) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        // Pivot.
        let mut pivot = col;
        for row in col + 1..n {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }
        if a[pivot][col].abs() < 1e-14 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        let inv = 1.0 / a[col][col];
        let pivot_row = a[col].clone();
        for row in col + 1..n {
            let factor = a[row][col] * inv;
            if factor != 0.0 {
                for (k, value) in pivot_row.iter().enumerate().skip(col) {
                    a[row][k] -= factor * value;
                }
                b[row] -= factor * b[col];
            }
        }
    }
    let mut x = vec![0.0; n];
    for row in (0..n).rev() {
        let sum: f64 = (row + 1..n).map(|k| a[row][k] * x[k]).sum();
        x[row] = (b[row] - sum) / a[row][row];
    }
    Some(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two reservoirs joined by one pipe — analytic solution.
    #[test]
    fn single_pipe_between_reservoirs() {
        let mut net = PipeNetwork::new();
        net.add_node(
            1,
            NetworkNode {
                head: 110.0,
                is_reservoir: true,
                demand: 0.0,
            },
        );
        net.add_node(
            2,
            NetworkNode {
                head: 100.0,
                is_reservoir: true,
                demand: 0.0,
            },
        );
        let r = pipe_resistance(0.02, 1000.0, 0.2);
        net.add_pipe(NetworkPipe {
            from: 1,
            to: 2,
            resistance: r,
        });
        // Newton: Q = √(10/r).
        let sol = net.solve_newton_raphson(1e-10, 100).unwrap();
        assert!(sol.converged);
        let expected = (10.0 / r).sqrt();
        assert!((sol.pipe_flows[0] - expected).abs() < 1e-6);
    }

    /// Classic two-loop grid: 2 reservoirs feeding a 2×2 junction grid.
    /// All three solvers must agree on every pipe flow.
    #[test]
    fn two_loop_network_all_solvers_agree() {
        let mut net = PipeNetwork::new();
        // Reservoirs
        net.add_node(
            1,
            NetworkNode {
                head: 100.0,
                is_reservoir: true,
                demand: 0.0,
            },
        );
        net.add_node(
            2,
            NetworkNode {
                head: 90.0,
                is_reservoir: true,
                demand: 0.0,
            },
        );
        // Junctions with demands (m³/s)
        net.add_node(
            3,
            NetworkNode {
                head: 0.0,
                is_reservoir: false,
                demand: 0.10,
            },
        );
        net.add_node(
            4,
            NetworkNode {
                head: 0.0,
                is_reservoir: false,
                demand: 0.05,
            },
        );
        net.add_node(
            5,
            NetworkNode {
                head: 0.0,
                is_reservoir: false,
                demand: 0.08,
            },
        );
        net.add_node(
            6,
            NetworkNode {
                head: 0.0,
                is_reservoir: false,
                demand: 0.07,
            },
        );
        // Pipes (resistance units s²/m⁵)
        let r = |v: f64| v;
        for (from, to, resistance) in [
            (1u64, 3u64, r(300.0)),
            (1, 4, r(500.0)),
            (2, 5, r(400.0)),
            (2, 6, r(600.0)),
            (3, 4, r(200.0)),
            (3, 5, r(700.0)),
            (4, 6, r(350.0)),
            (5, 6, r(250.0)),
        ] {
            net.add_pipe(NetworkPipe {
                from,
                to,
                resistance,
            });
        }

        let hardy = net.solve_hardy_cross(1e-8, 500).unwrap();
        let newton = net.solve_newton_raphson(1e-8, 100).unwrap();
        let linear = net.solve_linear_theory(1e-8, 200).unwrap();

        assert!(hardy.converged, "hardy cross must converge");
        assert!(newton.converged, "newton must converge");
        assert!(linear.converged, "linear theory must converge");

        // Global continuity: total supply = total demand.
        let total_demand: f64 = [0.10, 0.05, 0.08, 0.07].iter().sum();
        let supply: f64 = (0..net.pipes.len())
            .filter(|i| {
                let node = &net.pipes[*i].from;
                net.nodes[node].is_reservoir
            })
            .map(|i| hardy.pipe_flows[i])
            .sum();
        assert!(
            (supply - total_demand).abs() < 1e-4,
            "supply {supply} vs demand {total_demand}"
        );

        // Junction continuity under Hardy Cross flows.
        for (id, node) in &net.nodes {
            if node.is_reservoir {
                continue;
            }
            let mut balance = -node.demand;
            for (i, pipe) in net.pipes.iter().enumerate() {
                if pipe.to == *id {
                    balance += hardy.pipe_flows[i];
                }
                if pipe.from == *id {
                    balance -= hardy.pipe_flows[i];
                }
            }
            assert!(balance.abs() < 1e-4, "junction {id} imbalance {balance}");
        }

        // Cross-solver agreement on flows (Newton vs Hardy within 2%).
        for (fh, fnw) in hardy.pipe_flows.iter().zip(&newton.pipe_flows) {
            assert!(
                (fh - fnw).abs() < 0.02 * fh.abs().max(1e-3) + 1e-4,
                "hardy {fh} vs newton {fnw}"
            );
        }
        for (fl, fnw) in linear.pipe_flows.iter().zip(&newton.pipe_flows) {
            assert!(
                (fl - fnw).abs() < 0.02 * fnw.abs().max(1e-3) + 1e-4,
                "linear {fl} vs newton {fnw}"
            );
        }
    }

    #[test]
    fn resistance_formula_matches_definition() {
        // r = 8 f L / (π² g D⁵).
        let r = pipe_resistance(0.02, 1000.0, 0.2);
        let expected = 8.0 * 0.02 * 1000.0
            / (std::f64::consts::PI * std::f64::consts::PI * 9.80665 * 0.2f64.powi(5));
        assert!((r - expected).abs() < 1e-12);
    }

    #[test]
    fn malformed_networks_error() {
        // Unknown node reference.
        let mut net = PipeNetwork::new();
        net.add_pipe(NetworkPipe {
            from: 9,
            to: 8,
            resistance: 1.0,
        });
        assert!(net.solve_hardy_cross(1e-6, 10).is_err());
        // Tree (no cycles).
        let mut tree = PipeNetwork::new();
        tree.add_node(
            1,
            NetworkNode {
                head: 10.0,
                is_reservoir: true,
                demand: 0.0,
            },
        );
        tree.add_node(
            2,
            NetworkNode {
                head: 0.0,
                is_reservoir: false,
                demand: 0.0,
            },
        );
        tree.add_pipe(NetworkPipe {
            from: 1,
            to: 2,
            resistance: 1.0,
        });
        assert!(tree.solve_hardy_cross(1e-6, 10).is_err());
    }
}
