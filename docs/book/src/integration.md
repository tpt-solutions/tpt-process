# Cross-Repo Integration

`tpt-process` is the flowsheet brain of a wider TPT ecosystem. The
crates in `crates/integration/` connect it to the sibling
repositories today:

| Crate | Counterpart | Bridge |
|---|---|---|
| `tpt-proc-energy` | `tpt-energy` (`tpt-nrg-core`) | pinch utility targets → electrical loads and CHP capacity |
| `tpt-proc-materials` | `tpt-materials` (`tpt-mat-core`) | species streams → elemental compositions |
| `tpt-proc-construction` | `tpt-construction` (`tpt-c-model`) | PFD units → site/element records |

## How they are distributed

These crates are **workspace-internal**: they live in this repository
but depend on the counterpart crates as pinned git dependencies
(`tpt-solutions/tpt-energy`, `tpt-solutions/tpt-materials`,
`tpt-solutions/tpt-construction` — see the workspace `Cargo.toml`).
They are not published to crates.io; building them requires network
access to fetch the pinned revisions. Everything else in the workspace
is self-contained.

## Energy: pinch targets → power systems

`tpt-proc-energy` translates the process side's minimum heating and
cooling duties into `EnergySystem` quantities: cooling duty becomes a
chiller electrical load Q_cold/(COP·1000) MW; heating duty can be
served by a CHP generator sized at Q_heat·η_elec/1000 MW with the
balance as a thermal-fuel load. The `UtilityBridge` carries the
assumptions (chiller COP, CHP electrical efficiency, electric-heating
fraction) so they are stated, not implicit.

## Materials: species → elements

`tpt-proc-materials` converts process streams (metal sulfates,
hydroxides) into elemental `Composition`s by exact mass balance, and
implements the battery-precursor design: `PrecursorDesign::new(8.0,
1.0, 1.0)` targets the NMC-811 hydroxide M(OH)₂ composition.

## Construction: PFD → plant model

`tpt-proc-construction` exports a `Pfd` as a `tpt-construction`
project via `plant_project(&pfd)`: one site named after the diagram,
one `ProcessEquipment` element per node with a `Placement` property
set carrying the x/y coordinates. Element ids are deterministic
(UUIDv5 from the node id), so re-exporting an unchanged diagram is
idempotent and downstream diffs stay meaningful.

## Planned: medical

`tpt-medical` is the planned counterpart for pharmaceutical batch
processes; `tpt-proc-pharma` ([see the pharma page](pharma.md)) is the
process-side model that will feed it. The bridge lands when code
exists in the counterpart repository — no stub integration ships
before then.
