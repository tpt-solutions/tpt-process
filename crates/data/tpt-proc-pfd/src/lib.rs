//! Process flow diagram representation and export (SVG, Graphviz DOT,
//! Mermaid).
//!
//! # Example
//!
//! ```
//! use tpt_proc_pfd::{Node, Pfd};
//!
//! let mut pfd = Pfd::new("demo");
//! pfd.add_node(Node::new("u1", "Feed pump", 40.0, 100.0));
//! pfd.add_node(Node::new("u2", "Heater", 240.0, 100.0));
//! pfd.connect("u1", "u2", "S1");
//!
//! let dot = pfd.to_dot();
//! assert!(dot.contains("\"u1\" -> \"u2\""));
//! let svg = pfd.to_svg();
//! assert!(svg.contains("<svg"));
//! ```

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// A PFD node (unit operation or boundary marker).
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// Identifier.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Diagram x position.
    pub x: f64,
    /// Diagram y position.
    pub y: f64,
}

impl Node {
    /// Creates a node.
    #[must_use]
    pub fn new(id: &str, label: &str, x: f64, y: f64) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            x,
            y,
        }
    }
}

/// A PFD edge (material or energy stream).
#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    /// Source node id.
    pub from: String,
    /// Destination node id.
    pub to: String,
    /// Stream label (drawn on the arrow).
    pub label: String,
}

/// A process flow diagram.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pfd {
    title: String,
    nodes: BTreeMap<String, Node>,
    edges: Vec<Edge>,
}

impl Pfd {
    /// Creates an empty diagram.
    #[must_use]
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ..Self::default()
        }
    }

    /// Diagram title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Adds (or replaces) a node.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Adds an edge; missing endpoint nodes are created with default
    /// positions.
    pub fn connect(&mut self, from: &str, to: &str, label: &str) {
        for id in [from, to] {
            if !self.nodes.contains_key(id) {
                self.add_node(Node::new(id, id, 0.0, 0.0));
            }
        }
        self.edges.push(Edge {
            from: from.to_string(),
            to: to.to_string(),
            label: label.to_string(),
        });
    }

    /// The nodes, ordered by id.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// The edges, in insertion order.
    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Exports as Graphviz DOT.
    #[must_use]
    pub fn to_dot(&self) -> String {
        let mut out = String::from("digraph {\n  rankdir=LR;\n");
        if !self.title.is_empty() {
            out.push_str(&format!("  label=\"{}\";\n", escape(&self.title)));
        }
        for node in self.nodes.values() {
            out.push_str(&format!(
                "  \"{}\" [label=\"{}\", pos=\"{},{}!\"];\n",
                escape(&node.id),
                escape(&node.label),
                node.x,
                node.y
            ));
        }
        for e in &self.edges {
            out.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                escape(&e.from),
                escape(&e.to),
                escape(&e.label)
            ));
        }
        out.push('}');
        out
    }

    /// Exports as Mermaid flowchart syntax.
    #[must_use]
    pub fn to_mermaid(&self) -> String {
        let mut out = String::from("flowchart LR\n");
        for node in self.nodes.values() {
            out.push_str(&format!(
                "  {}[\"{}\"]\n",
                mermaid_id(&node.id),
                escape(&node.label)
            ));
        }
        for e in &self.edges {
            out.push_str(&format!(
                "  {} -->|\"{}\"| {}\n",
                mermaid_id(&e.from),
                escape(&e.label),
                mermaid_id(&e.to)
            ));
        }
        out
    }

    /// Exports as standalone SVG with node boxes and labeled arrows.
    #[must_use]
    pub fn to_svg(&self) -> String {
        const W: f64 = 120.0;
        const H: f64 = 50.0;
        let max_x = self
            .nodes
            .values()
            .map(|n| n.x + W)
            .fold(400.0_f64, f64::max);
        let max_y = self
            .nodes
            .values()
            .map(|n| n.y + H + 40.0)
            .fold(300.0_f64, f64::max);
        let mut out = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{max_x}\" height=\"{max_y}\">\n"
        );
        out.push_str(&format!(
            "  <text x=\"10\" y=\"20\" font-family=\"sans-serif\" font-size=\"14\">{}</text>\n",
            escape(&self.title)
        ));
        for node in self.nodes.values() {
            out.push_str(&format!(
                "  <rect x=\"{}\" y=\"{}\" width=\"{W}\" height=\"{H}\" fill=\"none\" stroke=\"black\"/>\n",
                node.x, node.y
            ));
            out.push_str(&format!(
                "  <text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"11\">{}</text>\n",
                node.x + 6.0,
                node.y + H / 2.0 + 4.0,
                escape(&node.label)
            ));
        }
        for e in &self.edges {
            let (Some(a), Some(b)) = (self.nodes.get(&e.from), self.nodes.get(&e.to)) else {
                continue;
            };
            let x1 = a.x + W;
            let y1 = a.y + H / 2.0;
            let x2 = b.x;
            let y2 = b.y + H / 2.0;
            out.push_str(&format!(
                "  <line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"black\" marker-end=\"url(#arrow)\"/>\n"
            ));
            out.push_str(&format!(
                "  <text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"10\">{}</text>\n",
                (x1 + x2) / 2.0,
                (y1 + y2) / 2.0 - 4.0,
                escape(&e.label)
            ));
        }
        out.push_str("  <defs><marker id=\"arrow\" markerWidth=\"10\" markerHeight=\"10\" refX=\"9\" refY=\"3\" orient=\"auto\"><path d=\"M0,0 L0,6 L9,3 z\" fill=\"black\"/></marker></defs>\n");
        out.push_str("</svg>\n");
        out
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn mermaid_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo() -> Pfd {
        let mut pfd = Pfd::new("demo plant");
        pfd.add_node(Node::new("u1", "Feed pump", 40.0, 100.0));
        pfd.add_node(Node::new("u2", "Heater", 240.0, 100.0));
        pfd.connect("u1", "u2", "S1");
        pfd
    }

    #[test]
    fn dot_export_has_edges_and_nodes() {
        let dot = demo().to_dot();
        assert!(dot.contains("digraph"));
        assert!(dot.contains("\"u1\" -> \"u2\""));
        assert!(dot.contains("Feed pump"));
    }

    #[test]
    fn svg_export_is_standalone() {
        let svg = demo().to_svg();
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("Feed pump"));
        assert!(svg.contains("S1"));
    }

    #[test]
    fn mermaid_export() {
        let m = demo().to_mermaid();
        assert!(m.starts_with("flowchart LR"));
        assert!(m.contains("-->"));
    }

    #[test]
    fn connect_creates_missing_nodes() {
        let mut pfd = Pfd::new("x");
        pfd.connect("a", "b", "s");
        assert_eq!(pfd.nodes().count(), 2);
    }
}
