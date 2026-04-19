use crate::model::graph::Graph;
use crate::model::ids::NodeId;
use crate::model::scenario::Scenario;
use crate::model::state::NodeKind;
use std::collections::{BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
}

pub fn validate_scenario(scenario: &Scenario) -> Result<(), Vec<ValidationIssue>> {
    let mut issues = Vec::new();
    validate_graph(&scenario.graph, &mut issues);

    for spawn in &scenario.spawns {
        if scenario.graph.node(spawn.origin).is_none() {
            issues.push(ValidationIssue {
                code: "spawn-origin-missing".into(),
                message: format!("El origen {} no existe.", spawn.origin),
            });
        }
        if scenario.graph.node(spawn.destination).is_none() {
            issues.push(ValidationIssue {
                code: "spawn-destination-missing".into(),
                message: format!("El destino {} no existe.", spawn.destination),
            });
        }
        if spawn.release_tick > scenario.duration_ticks {
            issues.push(ValidationIssue {
                code: "spawn-release-out-of-range".into(),
                message: format!(
                    "El spawn {:?} queda fuera de la duración del escenario.",
                    spawn.id
                ),
            });
        }
        if !has_path(&scenario.graph, spawn.origin, spawn.destination) {
            issues.push(ValidationIssue {
                code: "spawn-unroutable".into(),
                message: format!(
                    "No existe ruta desde {} hasta {}.",
                    spawn.origin, spawn.destination
                ),
            });
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

fn validate_graph(graph: &Graph, issues: &mut Vec<ValidationIssue>) {
    if graph.nodes.is_empty() {
        issues.push(ValidationIssue {
            code: "graph-empty".into(),
            message: "La red no contiene nodos.".into(),
        });
    }

    for node in &graph.nodes {
        if matches!(node.kind, NodeKind::Source) && graph.outgoing_segments(node.id).is_empty() {
            issues.push(ValidationIssue {
                code: "source-without-outgoing".into(),
                message: format!("El nodo fuente {} no tiene tramos de salida.", node.id),
            });
        }
    }

    for segment in &graph.segments {
        if graph.node(segment.from).is_none() {
            issues.push(ValidationIssue {
                code: "segment-missing-origin".into(),
                message: format!("El tramo {} referencia un origen inexistente.", segment.id),
            });
        }
        if graph.node(segment.to).is_none() {
            issues.push(ValidationIssue {
                code: "segment-missing-destination".into(),
                message: format!("El tramo {} referencia un destino inexistente.", segment.id),
            });
        }
        if segment.length_m <= 0.0 {
            issues.push(ValidationIssue {
                code: "segment-invalid-length".into(),
                message: format!("El tramo {} tiene longitud no valida.", segment.id),
            });
        }
        if segment.capacity == 0 {
            issues.push(ValidationIssue {
                code: "segment-zero-capacity".into(),
                message: format!("El tramo {} tiene capacidad cero.", segment.id),
            });
        }
    }
}

fn has_path(graph: &Graph, origin: NodeId, destination: NodeId) -> bool {
    if origin == destination {
        return true;
    }

    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([origin]);
    visited.insert(origin);

    while let Some(node) = queue.pop_front() {
        for segment_id in graph.outgoing_segments(node) {
            if let Some(segment) = graph.segment(*segment_id) {
                if segment.to == destination {
                    return true;
                }
                if visited.insert(segment.to) {
                    queue.push_back(segment.to);
                }
            }
        }
    }

    false
}
