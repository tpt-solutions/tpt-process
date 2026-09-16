//! Flow splitter: divides one feed into outlet streams by fixed ratios.

use tpt_proc_core::{FlowRate, MaterialStream};

use crate::{UnitBehavior, UnitError};

/// Divides the feed stream among two or more outlets.
///
/// Every outlet carries the feed's state (T, P, composition, phase); flows
/// are scaled by the split ratios. Ratios are normalized on construction,
/// so e.g. `[2, 1, 1]` produces 50%/25%/25% outlets.
#[derive(Clone, Debug, PartialEq)]
pub struct Splitter {
    ratios: Vec<f64>,
}

impl Splitter {
    /// Creates a splitter from split ratios (any positive, finite values;
    /// normalized internally).
    pub fn new(ratios: Vec<f64>) -> Self {
        let total: f64 = ratios.iter().sum();
        let normalized = if total > 0.0 {
            ratios.iter().map(|r| r / total).collect()
        } else {
            ratios
        };
        Self { ratios: normalized }
    }

    /// Creates a splitter, validating the ratios first.
    ///
    /// # Errors
    /// [`UnitError::Unsupported`] when fewer than two ratios are given, a
    /// ratio is non-finite/negative, or the ratios sum to zero.
    pub fn validated(ratios: Vec<f64>) -> Result<Self, UnitError> {
        if ratios.len() < 2 {
            return Err(UnitError::Unsupported(
                "a splitter needs at least two outlets".into(),
            ));
        }
        if ratios.iter().any(|r| !r.is_finite() || *r < 0.0) {
            return Err(UnitError::Unsupported(
                "split ratios must be finite and non-negative".into(),
            ));
        }
        if ratios.iter().sum::<f64>() <= 0.0 {
            return Err(UnitError::Unsupported("split ratios must sum > 0".into()));
        }
        Ok(Self::new(ratios))
    }

    /// The (normalized) split ratios.
    #[must_use]
    pub fn ratios(&self) -> &[f64] {
        &self.ratios
    }
}

impl UnitBehavior for Splitter {
    fn kind(&self) -> &'static str {
        "splitter"
    }

    fn num_inlets(&self) -> usize {
        1
    }

    fn num_outlets(&self) -> usize {
        self.ratios.len()
    }

    fn solve(&self, inlets: &[MaterialStream]) -> Result<Vec<MaterialStream>, UnitError> {
        if inlets.len() != 1 {
            return Err(UnitError::WrongInletCount {
                expected: 1,
                got: inlets.len(),
            });
        }
        let feed = &inlets[0];
        let total = feed
            .total_flow()
            .map_err(|e| UnitError::MissingStreamData(e.to_string()))?;
        let composition = feed.composition().cloned().ok_or_else(|| {
            UnitError::MissingStreamData(format!("feed {} lacks composition", feed.id()))
        })?;

        let mut outlets = Vec::with_capacity(self.ratios.len());
        for (port, ratio) in self.ratios.iter().enumerate() {
            let scaled_flow = match feed.flow_rate() {
                Some(FlowRate::Mass(_)) => Some(FlowRate::Mass(total * ratio)),
                Some(FlowRate::Molar(_)) => Some(FlowRate::Molar(total * ratio)),
                Some(FlowRate::Volumetric(_)) => Some(FlowRate::Volumetric(total * ratio)),
                None => None,
            };
            let mut outlet = MaterialStream::new(
                outlet_id(feed.id().value(), port),
                format!("{}-split{port}", feed.name()),
            )
            .with_state(feed.temperature(), feed.pressure())
            .map_err(|e| UnitError::MissingStreamData(e.to_string()))?
            .with_composition(composition.clone());
            if let Some(flow) = scaled_flow {
                outlet = outlet
                    .with_flow(flow)
                    .map_err(|e| UnitError::MissingStreamData(e.to_string()))?;
            }
            if let Some(phase) = feed.phase() {
                outlet = outlet.with_phase(phase);
            }
            outlets.push(outlet);
        }
        Ok(outlets)
    }
}

/// Synthetic id for a splitter outlet (namespaced above any feed id).
fn outlet_id(feed: u64, port: usize) -> u64 {
    feed * 100 + 5000 + port as u64
}
