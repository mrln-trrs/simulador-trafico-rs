use traffic_simulator::{build_demo_simulation, Network, SimulationEvent};

fn node_label(network: &Network, node_id: usize) -> String {
    network
        .node(node_id)
        .map(|node| node.name.clone())
        .unwrap_or_else(|| format!("Nodo {}", node_id))
}

fn road_label(network: &Network, road_id: usize) -> String {
    network
        .road(road_id)
        .map(|road| road.name.clone())
        .unwrap_or_else(|| format!("Tramo {}", road_id))
}

fn print_header(simulation: &traffic_simulator::Simulation) {
    println!("Simulador de trafico en Rust");
    println!(
        "Red inicial: {} nodos, {} tramos",
        simulation.network.node_count(),
        simulation.network.road_count()
    );
    println!("Modelo: red de nodos, tramos con colas y semaforos por fases");
    println!();
}

fn print_event(simulation: &traffic_simulator::Simulation, event: SimulationEvent) {
    match event {
        SimulationEvent::Spawned {
            vehicle_id,
            name,
            origin,
            destination,
            route,
        } => {
            println!(
                "[tick {:02}] sale #{vehicle_id} {name} desde {} hacia {} por {}",
                simulation.tick(),
                node_label(&simulation.network, origin),
                node_label(&simulation.network, destination),
                simulation.network.describe_route(&route)
            );
        }
        SimulationEvent::SpawnFailed {
            name,
            origin,
            destination,
            reason,
        } => {
            println!(
                "[tick {:02}] no se pudo crear {name} ({}) -> ({}) : {}",
                simulation.tick(),
                node_label(&simulation.network, origin),
                node_label(&simulation.network, destination),
                reason
            );
        }
        SimulationEvent::SignalChanged { node_id, phase_name } => {
            println!(
                "[tick {:02}] semaforo en {} cambia a {}",
                simulation.tick(),
                node_label(&simulation.network, node_id),
                phase_name
            );
        }
        SimulationEvent::EnteredRoad {
            vehicle_id,
            road_id,
            lane_index,
        } => {
            println!(
                "[tick {:02}] vehiculo #{vehicle_id} entra en {} (carril {})",
                simulation.tick(),
                road_label(&simulation.network, road_id),
                lane_index + 1
            );
        }
        SimulationEvent::Rerouted {
            vehicle_id,
            node_id,
            avoided_road,
            new_route,
            reason,
        } => {
            println!(
                "[tick {:02}] vehiculo #{vehicle_id} reruta en {} evitando {} por {} ({})",
                simulation.tick(),
                node_label(&simulation.network, node_id),
                road_label(&simulation.network, avoided_road),
                simulation.network.describe_route(&new_route),
                reason
            );
        }
        SimulationEvent::EmergencyEnteredRoad {
            vehicle_id,
            road_id,
            lane_index,
            reason,
        } => {
            println!(
                "[tick {:02}] vehiculo #{vehicle_id} entra en {} (carril {}) por emergencia: {}",
                simulation.tick(),
                road_label(&simulation.network, road_id),
                lane_index + 1,
                reason
            );
        }
        SimulationEvent::ReachedNode { vehicle_id, node_id } => {
            println!(
                "[tick {:02}] vehiculo #{vehicle_id} llega a {}",
                simulation.tick(),
                node_label(&simulation.network, node_id)
            );
        }
        SimulationEvent::Completed {
            vehicle_id,
            destination,
            wait_time,
            travel_time,
        } => {
            println!(
                "[tick {:02}] vehiculo #{vehicle_id} termina en {} (espera {}, viaje {})",
                simulation.tick(),
                node_label(&simulation.network, destination),
                wait_time,
                travel_time
            );
        }
    }
}

fn print_tick_summary(simulation: &traffic_simulator::Simulation) {
    println!(
        "[tick {:02}] activos={}, completados={}",
        simulation.tick(),
        simulation.active_vehicle_count(),
        simulation.completed_vehicle_count()
    );
}

pub fn run() {
    let mut simulation = build_demo_simulation();

    print_header(&simulation);

    let max_ticks = 60;
    for _ in 0..max_ticks {
        let events = simulation.step();
        for event in events {
            print_event(&simulation, event);
        }

        if simulation.tick() % 5 == 0 {
            print_tick_summary(&simulation);
        }

        if simulation.is_idle() {
            break;
        }
    }

    let report = simulation.report();
    println!();
    println!("Resumen final: {report}");
}