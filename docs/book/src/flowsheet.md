# Flowsheet Simulation

The sequential-modular solver executes units in topological order over the
torn process graph. Recycle streams (chosen by the topology crate's cycle
basis) converge between passes with bounded Wegstein acceleration; unit
behaviors are stateless pure functions registered in a `UnitRegistry`.

Deterministic rules: ordered maps everywhere, id-order execution within a
pass, identical inputs → identical outputs.
