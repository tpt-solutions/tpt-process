//! Bridge from `tpt-process` PFDs into `tpt-construction` site and
//! project models (`tpt-c-model`).
//!
//! Process units become construction `Element`s (category
//! `"ProcessEquipment"`) carrying a `Placement` property set with their
//! PFD coordinates, grouped under one `Site` in a `Project`.
//!
//! # Example
//!
//! ```
//! use tpt_proc_construction::plant_project;
//! use tpt_proc_pfd::{Node, Pfd};
//!
//! let mut pfd = Pfd::new("distillation train");
//! pfd.add_node(Node::new("u1", "Feed pump", 40.0, 100.0));
//! pfd.add_node(Node::new("u2", "Column C-101", 240.0, 100.0));
//! pfd.connect("u1", "u2", "S1");
//!
//! let project = plant_project(&pfd);
//! assert_eq!(project.element_count(), 2);
//! assert_eq!(project.sites.len(), 1);
//! ```

#![forbid(unsafe_code)]

use tpt_c_core::ProjectId;
use tpt_c_ids::IdFactory;
use tpt_c_model::{Element, Project, PropertySet, PropertyValue, Site};
use tpt_proc_pfd::Pfd;

/// The element category assigned to every exported process unit.
pub const PROCESS_EQUIPMENT_CATEGORY: &str = "ProcessEquipment";

/// Exports a PFD as a `tpt-construction` project: one site named after
/// the diagram, one process-equipment element per node (with a
/// `Placement` property set carrying the x/y coordinates), and the site
/// referencing all exported element ids.
///
/// Element ids are *deterministic* (UUIDv5 derived from the PFD node id),
/// so re-exporting an unchanged diagram is idempotent and downstream
/// diffs stay meaningful.
#[must_use]
pub fn plant_project(pfd: &Pfd) -> Project {
    let project_id = ProjectId::from_uuid(IdFactory::deterministic(&format!(
        "project-{}",
        pfd.title()
    )));
    let mut project = Project::new(project_id, format!("process-{}", pfd.title()));
    let mut site = Site {
        id: "process-site".to_string(),
        name: pfd.title().to_string(),
        buildings: Vec::new(),
        elements: Vec::new(),
    };

    for node in pfd.nodes() {
        let id = IdFactory::deterministic_element(&node.id);
        let placement = PropertySet::new("Placement")
            .with("x", PropertyValue::Number(node.x))
            .with("y", PropertyValue::Number(node.y));
        let element = Element::new(id, node.label.clone(), PROCESS_EQUIPMENT_CATEGORY)
            .with_property_set(placement);
        project.add_element(element);
        site.elements.push(id);
    }

    project.sites.push(site);
    project
}

/// Exports a PFD as a project plus the serialized JSON of the project
/// (convenience for downstream `tpt-c-*` tooling).
///
/// # Errors
/// Propagates the model's serialization errors.
pub fn plant_project_json(pfd: &Pfd) -> Result<(Project, String), serde_json::Error> {
    let project = plant_project(pfd);
    let json = serde_json::to_string_pretty(&project)?;
    Ok((project, json))
}

/// The stream connectivity of a PFD as (from label, stream, to label)
/// triples — useful when the construction side needs line lists.
#[must_use]
pub fn line_list(pfd: &Pfd) -> Vec<(String, String, String)> {
    pfd.edges()
        .iter()
        .map(|e| (e.from.clone(), e.label.clone(), e.to.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_proc_pfd::Node;

    fn demo() -> Pfd {
        let mut pfd = Pfd::new("Distillation Area 100");
        pfd.add_node(Node::new("u1", "Feed pump P-101", 40.0, 100.0));
        pfd.add_node(Node::new("u2", "Column C-101", 240.0, 100.0));
        pfd.add_node(Node::new("u3", "Reboiler E-102", 440.0, 180.0));
        pfd.connect("u1", "u2", "S1");
        pfd.connect("u2", "u3", "S2");
        pfd
    }

    #[test]
    fn project_has_site_and_equipment() {
        let project = plant_project(&demo());
        assert_eq!(project.element_count(), 3);
        assert_eq!(project.sites.len(), 1);
        assert_eq!(project.sites[0].elements.len(), 3);
        assert_eq!(project.sites[0].name, "Distillation Area 100");
    }

    #[test]
    fn elements_carry_placement_and_category() {
        let project = plant_project(&demo());
        let count = project
            .elements_of_category(PROCESS_EQUIPMENT_CATEGORY)
            .count();
        assert_eq!(count, 3);
        for element in &project.elements {
            let placement = element
                .property_sets
                .iter()
                .find(|ps| ps.name == "Placement")
                .expect("placement set");
            assert_eq!(placement.properties.len(), 2);
        }
    }

    #[test]
    fn line_list_matches_edges() {
        let lines = line_list(&demo());
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0],
            ("u1".to_string(), "S1".to_string(), "u2".to_string())
        );
    }

    #[test]
    fn json_export_works() {
        let (_, json) = plant_project_json(&demo()).unwrap();
        assert!(json.contains("ProcessEquipment"));
    }
}
