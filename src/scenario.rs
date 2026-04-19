use crate::generation::build_demo_schedule;
use crate::model::{Network, Node, NodeKind, RoadSegment, SignalPhase, SignalPlan};
use crate::simulation::Simulation;

pub fn build_demo_network() -> Network {
    let mut network = Network::new();

    network.add_node(Node::new(1, "Acceso Norte", NodeKind::Entry));
    network.add_node(
        Node::new(2, "Semaforo Central", NodeKind::TrafficLight).with_signal_plan(
            SignalPlan::new(vec![
                SignalPhase::new("Paso principal", 4, vec![2]),
                SignalPhase::new("Paso lateral", 2, vec![3]),
            ]),
        ),
    );
    network.add_node(Node::new(3, "Desvio Este", NodeKind::Intersection));
    network.add_node(Node::new(4, "Destino Principal", NodeKind::Exit));

    network.add_road(RoadSegment::new(1, "Avenida Norte", 1, 2, 420.0, 2, 45.0, 3));
    network.add_road(RoadSegment::new(2, "Salida Principal", 2, 4, 600.0, 1, 50.0, 2));
    network.add_road(RoadSegment::new(3, "Desvio Este", 2, 3, 260.0, 1, 35.0, 2));
    network.add_road(RoadSegment::new(4, "Conector Este", 3, 4, 450.0, 2, 50.0, 3));
    network.add_road(RoadSegment::new(5, "Ruta Lateral", 1, 3, 520.0, 1, 40.0, 2));

    network
}

pub fn build_demo_simulation() -> Simulation {
    let mut simulation = Simulation::new(build_demo_network());
    for spawn in build_demo_schedule() {
        simulation.schedule_spawn(spawn);
    }
    simulation
}
