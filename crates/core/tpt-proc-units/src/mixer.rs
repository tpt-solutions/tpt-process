//! Adiabatic mixer: merges two or more streams on a common flow basis.

use tpt_proc_core::{Composition, CompositionBasis, FlowRate, MaterialStream};

use crate::{UnitBehavior, UnitError};

/// Adiabatic merge of two or more inlets into one outlet.
///
/// Mixing model (no thermodynamics required):
/// - the outlet flow is the sum of inlet flows (all inlets must share one
///   basis: molar, mass, or volumetric);
/// - the outlet composition is the flow-weighted mixture of the inlet
///   compositions (which must share component count and basis);
/// - the outlet temperature is the flow-weighted arithmetic mean of the
///   inlet temperatures (a proxy for the true adiabatic mixing temperature
///   — equal heat capacities assumed — until the flash-based mixer replaces
///   it);
/// - the outlet pressure is the minimum inlet pressure (conservative
///   maximum-mixing-pressure rule);
/// - the outlet phase is left unset: phase equilibrium is a
///   property-package question, filled in by the solver.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mixer;

impl UnitBehavior for Mixer {
    fn kind(&self) -> &'static str {
        "mixer"
    }

    fn num_inlets(&self) -> usize {
        2
    }

    fn num_outlets(&self) -> usize {
        1
    }

    fn solve(&self, inlets: &[MaterialStream]) -> Result<Vec<MaterialStream>, UnitError> {
        if inlets.len() < 2 {
            return Err(UnitError::WrongInletCount {
                expected: 2,
                got: inlets.len(),
            });
        }

        let basis_flow = inlet_flow_basis(inlets)?;
        let mut total_flow = 0.0;
        let mut weighted_temperature = 0.0;
        let mut min_pressure = f64::INFINITY;
        let mut fractions = vec![0.0; first_composition(inlets)?.len()];
        let mut fraction_basis: Option<CompositionBasis> = None;

        for inlet in inlets {
            let flow = inlet
                .total_flow()
                .map_err(|e| UnitError::MissingStreamData(e.to_string()))?;
            if !inlet.temperature().is_finite() || !inlet.pressure().is_finite() {
                return Err(UnitError::MissingStreamData(format!(
                    "inlet {} lacks T/P state",
                    inlet.id()
                )));
            }
            let composition = inlet.composition().ok_or_else(|| {
                UnitError::MissingStreamData(format!("inlet {} lacks composition", inlet.id()))
            })?;
            if composition.len() != fractions.len() {
                return Err(UnitError::Unsupported(
                    "inlet compositions disagree on component count".into(),
                ));
            }
            let this_basis = composition.basis();
            match fraction_basis {
                None => fraction_basis = Some(this_basis),
                Some(b) if b == this_basis => {}
                Some(_) => {
                    return Err(UnitError::Unsupported(
                        "inlet compositions disagree on basis".into(),
                    ))
                }
            }
            total_flow += flow;
            weighted_temperature += flow * inlet.temperature();
            min_pressure = min_pressure.min(inlet.pressure());
            for (accumulator, x) in fractions.iter_mut().zip(composition.as_slice()) {
                *accumulator += flow * x;
            }
        }

        let normalized: Vec<f64> = fractions.iter().map(|f| f / total_flow).collect();
        let composition = Composition::from_parts(
            &normalized,
            fraction_basis.unwrap_or(CompositionBasis::Mole),
        )
        .map_err(|e| UnitError::MissingStreamData(e.to_string()))?;

        let outlet = MaterialStream::new(
            outlet_id(inlets),
            format!("mixed-{}", inlets[0].id().value()),
        )
        .with_state(weighted_temperature / total_flow, min_pressure)
        .map_err(|e| UnitError::MissingStreamData(e.to_string()))?
        .with_flow(scaled(basis_flow, total_flow))
        .map_err(|e| UnitError::MissingStreamData(e.to_string()))?
        .with_composition(composition);

        Ok(vec![outlet])
    }
}

fn first_composition(inlets: &[MaterialStream]) -> Result<&Composition, UnitError> {
    inlets[0].composition().ok_or_else(|| {
        UnitError::MissingStreamData(format!("inlet {} lacks composition", inlets[0].id()))
    })
}

fn inlet_flow_basis(inlets: &[MaterialStream]) -> Result<FlowRate, UnitError> {
    let first = inlets[0].flow_rate().ok_or_else(|| {
        UnitError::MissingStreamData(format!("inlet {} lacks flow", inlets[0].id()))
    })?;
    for inlet in inlets {
        let Some(other) = inlet.flow_rate() else {
            return Err(UnitError::MissingStreamData(format!(
                "inlet {} lacks flow",
                inlet.id()
            )));
        };
        if std::mem::discriminant(&first) != std::mem::discriminant(&other) {
            return Err(UnitError::Unsupported(
                "inlet flows disagree on basis; convert to a common basis first".into(),
            ));
        }
    }
    Ok(first)
}

fn scaled(basis: FlowRate, value: f64) -> FlowRate {
    match basis {
        FlowRate::Mass(_) => FlowRate::Mass(value),
        FlowRate::Molar(_) => FlowRate::Molar(value),
        FlowRate::Volumetric(_) => FlowRate::Volumetric(value),
    }
}

/// Synthetic id for a mixer outlet (namespaced above any inlet id).
fn outlet_id(inlets: &[MaterialStream]) -> u64 {
    inlets.iter().map(|s| s.id().value()).max().unwrap_or(0) + 1000
}
