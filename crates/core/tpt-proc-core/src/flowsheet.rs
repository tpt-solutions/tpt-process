//! Flowsheets: the container tying streams, units, and connections together.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::error::{CoreError, Result};
use crate::ids::{FlowsheetId, PortId, StreamId, UnitId};
use crate::property::PropertyPackage;
use crate::stream::MaterialStream;
use crate::units::UnitOperation;

/// A directed attachment of a stream to unit ports.
///
/// `from == None` marks a boundary feed (no producing unit);
/// `to == None` marks a product withdrawal (no consuming unit).
/// At least one endpoint must be set.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Connection {
    /// Source (unit, port), or `None` for a boundary feed.
    pub from: Option<(UnitId, PortId)>,
    /// Destination (unit, port), or `None` for a product withdrawal.
    pub to: Option<(UnitId, PortId)>,
    /// The material stream carrying the flow.
    pub stream: StreamId,
}

impl Connection {
    /// Internal connection unit → unit.
    #[must_use]
    pub fn internal(
        from_unit: UnitId,
        from_port: PortId,
        to_unit: UnitId,
        to_port: PortId,
        stream: StreamId,
    ) -> Self {
        Self {
            from: Some((from_unit, from_port)),
            to: Some((to_unit, to_port)),
            stream,
        }
    }

    /// Boundary feed into a unit port (no producing unit).
    #[must_use]
    pub fn feed(to_unit: UnitId, to_port: PortId, stream: StreamId) -> Self {
        Self {
            from: None,
            to: Some((to_unit, to_port)),
            stream,
        }
    }

    /// Product withdrawal from a unit port (no consuming unit).
    #[must_use]
    pub fn product(from_unit: UnitId, from_port: PortId, stream: StreamId) -> Self {
        Self {
            from: Some((from_unit, from_port)),
            to: None,
            stream,
        }
    }
}

/// A process flowsheet: streams, unit operations, and their connections.
///
/// Maps are ordered (`BTreeMap`), so iteration and graph analysis are
/// deterministic regardless of insertion order.
///
/// With the `serde` feature, a flowsheet serializes to and from plain
/// data (JSON, TOML, …). The attached [`PropertyPackage`] is a trait
/// object and is deliberately **not** serialized (`serde(skip)`); after
/// deserializing, re-attach a package with
/// [`Flowsheet::set_property_package`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default)]
pub struct Flowsheet {
    id: FlowsheetId,
    name: String,
    streams: BTreeMap<StreamId, MaterialStream>,
    units: BTreeMap<UnitId, UnitOperation>,
    connections: Vec<Connection>,
    #[cfg_attr(feature = "serde", serde(skip))]
    property_package: Option<Arc<dyn PropertyPackage>>,
}

impl std::fmt::Debug for Flowsheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Flowsheet")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("streams", &self.streams.len())
            .field("units", &self.units.len())
            .field("connections", &self.connections.len())
            .finish_non_exhaustive()
    }
}

impl Flowsheet {
    /// Creates an empty flowsheet.
    #[must_use]
    pub fn new(id: impl Into<FlowsheetId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Self::default()
        }
    }

    /// Flowsheet identifier.
    #[must_use]
    pub const fn id(&self) -> FlowsheetId {
        self.id
    }

    /// Flowsheet name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Attaches the property package used for evaluations.
    pub fn set_property_package(&mut self, package: Arc<dyn PropertyPackage>) {
        self.property_package = Some(package);
    }

    /// The attached property package, if any.
    pub fn property_package(&self) -> Option<&Arc<dyn PropertyPackage>> {
        self.property_package.as_ref()
    }

    /// Adds (or replaces) a stream.
    pub fn add_stream(&mut self, stream: MaterialStream) {
        self.streams.insert(stream.id(), stream);
    }

    /// Adds (or replaces) a unit operation.
    pub fn add_unit(&mut self, unit: UnitOperation) {
        self.units.insert(unit.id(), unit);
    }

    /// Adds an internal connection unit → unit.
    ///
    /// # Errors
    /// [`CoreError::MissingReference`] for unknown units/streams,
    /// [`CoreError::InvalidFlowsheet`] for out-of-range ports or a
    /// self-connection.
    pub fn connect(
        &mut self,
        from_unit: UnitId,
        from_port: PortId,
        to_unit: UnitId,
        to_port: PortId,
        stream: StreamId,
    ) -> Result<()> {
        self.check_source(from_unit, from_port, stream)?;
        self.check_sink(to_unit, to_port, stream)?;
        if from_unit == to_unit {
            return Err(CoreError::InvalidFlowsheet(format!(
                "self-connection on unit {from_unit} is not allowed"
            )));
        }
        self.connections.push(Connection::internal(
            from_unit, from_port, to_unit, to_port, stream,
        ));
        Ok(())
    }

    /// Attaches a boundary feed to a unit inlet port.
    ///
    /// # Errors
    /// See [`Flowsheet::connect`].
    pub fn feed(&mut self, to_unit: UnitId, to_port: PortId, stream: StreamId) -> Result<()> {
        self.check_sink(to_unit, to_port, stream)?;
        self.connections
            .push(Connection::feed(to_unit, to_port, stream));
        Ok(())
    }

    /// Withdraws a product stream from a unit outlet port.
    ///
    /// # Errors
    /// See [`Flowsheet::connect`].
    pub fn withdraw(
        &mut self,
        from_unit: UnitId,
        from_port: PortId,
        stream: StreamId,
    ) -> Result<()> {
        self.check_source(from_unit, from_port, stream)?;
        self.connections
            .push(Connection::product(from_unit, from_port, stream));
        Ok(())
    }

    fn check_source(&self, unit: UnitId, port: PortId, stream: StreamId) -> Result<()> {
        let source = self
            .units
            .get(&unit)
            .ok_or_else(|| CoreError::MissingReference(format!("unit {unit}")))?;
        if port.value() as usize >= source.num_outlets() {
            return Err(CoreError::InvalidFlowsheet(format!(
                "unit {unit} ({}) has {} outlets; port {port} out of range",
                source.kind(),
                source.num_outlets()
            )));
        }
        self.check_stream(stream)
    }

    fn check_sink(&self, unit: UnitId, port: PortId, stream: StreamId) -> Result<()> {
        let sink = self
            .units
            .get(&unit)
            .ok_or_else(|| CoreError::MissingReference(format!("unit {unit}")))?;
        if port.value() as usize >= sink.num_inlets() {
            return Err(CoreError::InvalidFlowsheet(format!(
                "unit {unit} ({}) has {} inlets; port {port} out of range",
                sink.kind(),
                sink.num_inlets()
            )));
        }
        self.check_stream(stream)
    }

    fn check_stream(&self, stream: StreamId) -> Result<()> {
        if !self.streams.contains_key(&stream) {
            return Err(CoreError::MissingReference(format!("stream {stream}")));
        }
        Ok(())
    }

    /// All streams, ordered by id.
    pub fn streams(&self) -> impl Iterator<Item = &MaterialStream> {
        self.streams.values()
    }

    /// Lookup of a stream by id.
    #[must_use]
    pub fn stream(&self, id: StreamId) -> Option<&MaterialStream> {
        self.streams.get(&id)
    }

    /// Mutable lookup of a stream by id (for solvers to fill in results).
    pub fn stream_mut(&mut self, id: StreamId) -> Option<&mut MaterialStream> {
        self.streams.get_mut(&id)
    }

    /// All units, ordered by id.
    pub fn units(&self) -> impl Iterator<Item = &UnitOperation> {
        self.units.values()
    }

    /// Lookup of a unit by id.
    #[must_use]
    pub fn unit(&self, id: UnitId) -> Option<&UnitOperation> {
        self.units.get(&id)
    }

    /// All connections in insertion order.
    #[must_use]
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    /// Streams with no producing unit (boundary feeds plus unattached
    /// streams), ordered by id.
    #[must_use]
    pub fn inlet_streams(&self) -> Vec<StreamId> {
        let mut result: Vec<StreamId> = self
            .streams
            .keys()
            .copied()
            .filter(|s| {
                !self
                    .connections
                    .iter()
                    .any(|c| c.from.is_some() && c.stream == *s)
            })
            .collect();
        result.sort_unstable();
        result
    }

    /// Streams with no consuming unit (products plus unattached streams),
    /// ordered by id.
    #[must_use]
    pub fn outlet_streams(&self) -> Vec<StreamId> {
        let mut result: Vec<StreamId> = self
            .streams
            .keys()
            .copied()
            .filter(|s| {
                !self
                    .connections
                    .iter()
                    .any(|c| c.to.is_some() && c.stream == *s)
            })
            .collect();
        result.sort_unstable();
        result
    }

    /// Validates structural invariants: every connection references existing
    /// objects with in-range ports and at least one endpoint.
    ///
    /// # Errors
    /// Returns the first structural error found, if any.
    pub fn validate(&self) -> Result<()> {
        for c in &self.connections {
            if c.from.is_none() && c.to.is_none() {
                return Err(CoreError::InvalidFlowsheet(format!(
                    "connection on stream {} has neither endpoint",
                    c.stream
                )));
            }
            if let Some((u, p)) = c.from {
                self.check_source(u, p, c.stream)?;
            }
            if let Some((u, p)) = c.to {
                self.check_sink(u, p, c.stream)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::Composition;
    use crate::stream::{FlowRate, PhaseState};
    use crate::units::PumpConfig;

    fn feed_stream(id: u64) -> MaterialStream {
        MaterialStream::new(id, format!("stream-{id}"))
            .with_state(300.0, 101_325.0)
            .unwrap()
            .with_flow(FlowRate::Molar(10.0))
            .unwrap()
            .with_composition(Composition::from_mole_fractions(&[1.0]).unwrap())
            .with_phase(PhaseState::Liquid)
    }

    fn pump(id: u64) -> UnitOperation {
        UnitOperation::Pump {
            id: UnitId(id),
            config: PumpConfig {
                curve: [30.0, -100.0, -50.0],
                speed_rpm: 1750.0,
                efficiency: 0.7,
                pressure_rise: None,
            },
        }
    }

    #[test]
    fn build_simple_pfd() {
        let mut fs = Flowsheet::new(0, "pump to flash");
        for id in 0..3 {
            fs.add_stream(feed_stream(id));
        }
        fs.add_unit(pump(0));
        fs.add_unit(UnitOperation::Flash { id: UnitId(1) });

        fs.feed(UnitId(0), PortId(0), StreamId(0)).unwrap();
        fs.connect(UnitId(0), PortId(0), UnitId(1), PortId(0), StreamId(1))
            .unwrap();
        fs.withdraw(UnitId(1), PortId(0), StreamId(2)).unwrap();

        assert_eq!(fs.units().count(), 2);
        assert_eq!(fs.connections().len(), 3);
        assert!(fs.validate().is_ok());
        assert_eq!(fs.inlet_streams(), vec![StreamId(0)]);
        assert_eq!(fs.outlet_streams(), vec![StreamId(2)]);
    }

    #[test]
    fn connect_rejects_bad_ports() {
        let mut fs = Flowsheet::new(0, "x");
        fs.add_stream(feed_stream(0));
        fs.add_unit(pump(0));
        fs.add_unit(UnitOperation::Mixer {
            id: UnitId(1),
            inlets: 1,
        });
        // Pump has 1 outlet; port 1 is invalid.
        assert!(fs
            .connect(UnitId(0), PortId(1), UnitId(1), PortId(0), StreamId(0))
            .is_err());
        // Mixer has 1 inlet; port 1 is invalid.
        assert!(fs
            .connect(UnitId(1), PortId(0), UnitId(0), PortId(1), StreamId(0))
            .is_err());
        // Unknown stream.
        assert!(fs
            .connect(UnitId(1), PortId(0), UnitId(0), PortId(0), StreamId(99))
            .is_err());
        // Unknown unit.
        assert!(fs
            .connect(UnitId(7), PortId(0), UnitId(0), PortId(0), StreamId(0))
            .is_err());
    }

    #[test]
    fn self_connection_rejected() {
        let mut fs = Flowsheet::new(0, "x");
        fs.add_stream(feed_stream(0));
        fs.add_unit(UnitOperation::Mixer {
            id: UnitId(0),
            inlets: 2,
        });
        // Self connections are rejected: connect requires distinct units and
        // the mixer feeding itself is not representable.
        assert!(fs
            .connect(UnitId(0), PortId(0), UnitId(0), PortId(1), StreamId(0))
            .is_err());
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;
    use crate::composition::Composition;
    use crate::stream::{FlowRate, PhaseState};
    use crate::units::UnitOperation;

    fn sample_flowsheet() -> Flowsheet {
        let mut fs = Flowsheet::new(3, "serde sample");
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
        fs.add_unit(UnitOperation::Mixer {
            id: UnitId(1),
            inlets: 2,
        });
        fs.feed(UnitId(1), PortId(0), StreamId(1)).unwrap();
        fs
    }

    #[test]
    fn flowsheet_json_roundtrip() {
        let fs = sample_flowsheet();
        let json = serde_json::to_string(&fs).unwrap();
        let back: Flowsheet = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name(), "serde sample");
        assert_eq!(back.streams().count(), fs.streams().count());
        assert_eq!(back.units().count(), fs.units().count());
        assert_eq!(back.connections(), fs.connections());

        let original = fs.stream(StreamId(1)).unwrap();
        let restored = back.stream(StreamId(1)).unwrap();
        assert_eq!(original.id(), restored.id());
        assert_eq!(original.name(), restored.name());
        assert_eq!(original.temperature(), restored.temperature());
        assert_eq!(original.pressure(), restored.pressure());
        assert_eq!(original.flow_rate(), restored.flow_rate());
        assert_eq!(original.composition(), restored.composition());
        assert_eq!(original.phase(), restored.phase());
        assert_eq!(
            original.properties().is_complete(),
            restored.properties().is_complete()
        );
    }

    #[test]
    fn unit_operation_roundtrip() {
        let unit = UnitOperation::Splitter {
            id: UnitId(9),
            split_ratios: vec![0.8, 0.2],
        };
        let json = serde_json::to_string(&unit).unwrap();
        let back: UnitOperation = serde_json::from_str(&json).unwrap();
        assert_eq!(unit, back);
    }

    #[test]
    fn unset_fields_survive_json_without_nan() {
        let mut fs = Flowsheet::new(7, "bare");
        fs.add_stream(MaterialStream::new(1, "s1"));
        let json = serde_json::to_string(&fs).unwrap();
        assert!(
            !json.contains("NaN"),
            "JSON must not carry bare NaN: {json}"
        );
        let back: Flowsheet = serde_json::from_str(&json).unwrap();
        let s = back.stream(StreamId(1)).unwrap();
        assert!(s.temperature().is_nan());
        assert!(s.flow_rate().is_none());
        assert!(s.composition().is_none());
    }
}
