use crate::generation::builders::ScenarioBuilder;
use crate::model::signal::{SignalTiming, TrafficSignal};
use crate::model::state::{NodeKind, VehicleKind};
use crate::model::{NodeId, SignalId};

pub fn demo_scenario() -> crate::model::Scenario {
    ScenarioBuilder::new("Escenario de demostración")
        .seed(42)
        .duration_ticks(180)
        .node("Entrada", NodeKind::Source, 60.0, 180.0)
        .node("Cruce", NodeKind::Signal, 250.0, 180.0)
        .node("Salida", NodeKind::Sink, 420.0, 120.0)
        .node("Desvío", NodeKind::Intersection, 420.0, 240.0)
        .segment("A-B", 0, 1, 1, 190.0, 18.0, 4)
        .segment("B-C", 1, 2, 1, 180.0, 16.0, 4)
        .segment("B-D", 1, 3, 1, 180.0, 16.0, 4)
        .spawn(0, 0, 2, VehicleKind::Car, 18.0, "Auto 1")
        .spawn(4, 0, 2, VehicleKind::Bus, 15.0, "Bus 1")
        .spawn(8, 0, 3, VehicleKind::Truck, 14.0, "Camión 1")
        .signal(TrafficSignal::new(
            SignalId(0),
            NodeId(1),
            SignalTiming {
                green_ticks: 4,
                yellow_ticks: 1,
                red_ticks: 3,
            },
        ))
        .build()
}
