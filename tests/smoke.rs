use simulador_trafico::generation::ScenarioBuilder;
use simulador_trafico::model::state::{NodeKind, VehicleKind};
use simulador_trafico::persistence::{load_scenario, save_scenario};
use simulador_trafico::simulation::SimulationEngine;

fn simple_scenario() -> simulador_trafico::model::Scenario {
    ScenarioBuilder::new("Escenario simple")
        .duration_ticks(40)
        .node("Origen", NodeKind::Source, 0.0, 0.0)
        .node("Intermedio", NodeKind::Intersection, 120.0, 0.0)
        .node("Destino", NodeKind::Sink, 240.0, 0.0)
        .segment("A-B", 0, 1, 1, 120.0, 30.0, 4)
        .segment("B-C", 1, 2, 1, 120.0, 30.0, 4)
        .spawn(0, 0, 2, VehicleKind::Car, 30.0, "Auto")
        .build()
}

#[test]
fn demo_scenario_is_valid() {
    let scenario = simulador_trafico::generation::demo_scenario();
    assert!(simulador_trafico::model::validate_scenario(&scenario).is_ok());
}

#[test]
fn engine_completes_simple_trip() {
    let scenario = simple_scenario();
    let mut engine = SimulationEngine::new(scenario).expect("el escenario debe ser válido");

    engine.advance_many(20);

    let snapshot = engine.snapshot();
    assert_eq!(snapshot.metrics.vehicles_spawned, 1);
    assert_eq!(snapshot.metrics.vehicles_completed, 1);
    assert!(snapshot.vehicles.is_empty());
}

#[test]
fn scenario_roundtrip_json() {
    let scenario = simple_scenario();
    let dir = tempfile::tempdir().expect("directorio temporal");
    let path = dir.path().join("scenario.json");

    save_scenario(&path, &scenario).expect("guardar escenario");
    let loaded = load_scenario(&path).expect("cargar escenario");

    assert_eq!(scenario.name, loaded.name);
    assert_eq!(scenario.graph.nodes.len(), loaded.graph.nodes.len());
    assert_eq!(scenario.graph.segments.len(), loaded.graph.segments.len());
    assert_eq!(scenario.spawns.len(), loaded.spawns.len());
}
