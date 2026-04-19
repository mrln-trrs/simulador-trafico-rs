use std::fmt;

use crate::model::{NodeId, RoadId, VehicleId};

#[derive(Debug, Clone)]
pub enum SimulationEvent {
    Spawned {
        vehicle_id: VehicleId,
        name: String,
        origin: NodeId,
        destination: NodeId,
        route: Vec<RoadId>,
    },
    SpawnFailed {
        name: String,
        origin: NodeId,
        destination: NodeId,
        reason: String,
    },
    SignalChanged {
        node_id: NodeId,
        phase_name: String,
    },
    EnteredRoad {
        vehicle_id: VehicleId,
        road_id: RoadId,
        lane_index: usize,
    },
    Rerouted {
        vehicle_id: VehicleId,
        node_id: NodeId,
        avoided_road: RoadId,
        new_route: Vec<RoadId>,
        reason: String,
    },
    EmergencyEnteredRoad {
        vehicle_id: VehicleId,
        road_id: RoadId,
        lane_index: usize,
        reason: String,
    },
    ReachedNode {
        vehicle_id: VehicleId,
        node_id: NodeId,
    },
    Completed {
        vehicle_id: VehicleId,
        destination: NodeId,
        wait_time: u32,
        travel_time: u32,
    },
}

#[derive(Debug, Clone)]
pub struct SimulationReport {
    pub ticks_run: u32,
    pub vehicles_total: usize,
    pub vehicles_completed: usize,
    pub vehicles_active: usize,
    pub average_wait_time: f64,
    pub average_travel_time: f64,
}

impl fmt::Display for SimulationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ticks={}, total={}, completed={}, active={}, wait_avg={:.2}, travel_avg={:.2}",
            self.ticks_run,
            self.vehicles_total,
            self.vehicles_completed,
            self.vehicles_active,
            self.average_wait_time,
            self.average_travel_time,
        )
    }
}