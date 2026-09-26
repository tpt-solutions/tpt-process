# Visualize your flowsheet

`tpt-proc-pfd` is a minimal process flow diagram model with three
text exports: standalone SVG, Graphviz DOT, and Mermaid flowchart
syntax. Nodes are placed at explicit (x, y); edges carry a stream
label drawn on the arrow.

```rust
use tpt_proc_pfd::{Node, Pfd};

let mut pfd = Pfd::new("demo plant");
pfd.add_node(Node::new("u1", "Feed pump", 40.0, 100.0));
pfd.add_node(Node::new("u2", "Heater", 240.0, 100.0));
pfd.connect("u1", "u2", "S1");

pfd.to_svg();     // standalone <svg> with boxes, arrows, labels
pfd.to_dot();     // "digraph" with rankdir=LR and pos="x,y!" hints
pfd.to_mermaid(); // "flowchart LR" for Markdown embeds
```

- **SVG** needs no tooling: it is a complete document with node boxes,
  labeled edges, and an arrow marker — write it to a file and open it.
- **DOT** renders through Graphviz (`dot -Tsvg pfd.dot -o pfd.svg`);
  the `pos` attributes are layout hints, not commands.
- **Mermaid** pastes directly into Markdown, GitLab/GitHub included.

`connect` creates missing endpoint nodes on the fly (default
position (0, 0)), and nodes are stored in a `BTreeMap`, so exports are
ordered by id and deterministic. Text is escaped, so labels may
contain quotes or angle brackets.

## From a simulated flowsheet

The same nodes and edges come out of a `tpt_proc_core::Flowsheet`:
`tpt_proc_topology::ProcessGraph::from_flowsheet(&flowsheet)` turns
every unit-to-unit connection into an `Edge { from, to, stream }`
(boundary feeds and withdrawals are not part of the unit digraph).
Map unit ids to `Node`s and stream ids to edge labels to diagram a
flowsheet you have just solved — [the CLI](cli.md) works the same
stream ids into its JSON report.
