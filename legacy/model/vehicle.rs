use crate::model::ids::{NodeId, SegmentId, VehicleId};
use crate::model::state::{VehicleKind, VehicleState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VehicleRoute {
    pub segments: Vec<SegmentId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VehicleSpawn {
    pub id: VehicleId,
    pub release_tick: u64,
    pub origin: NodeId,
    pub destination: NodeId,
    pub kind: VehicleKind,
    pub speed_mps: f64,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: VehicleId,
    pub label: String,
    pub kind: VehicleKind,
    pub route: VehicleRoute,
    pub route_index: usize,
    pub progress: f64,
    pub speed_mps: f64,
    pub state: VehicleState,
    pub spawn_tick: u64,
    pub entered_tick: Option<u64>,
    pub arrival_tick: Option<u64>,
    pub updated_tick: u64,
    pub wait_ticks: u64,
    pub reroute_count: u32,
}

impl Vehicle {
    pub fn new(spawn: &VehicleSpawn, route: VehicleRoute, tick: u64) -> Self {
        Self {
            id: spawn.id,
            label: spawn.label.clone(),
            kind: spawn.kind,
            route,
            route_index: 0,
            progress: 0.0,
            speed_mps: spawn.speed_mps,
            state: VehicleState::Queued,
            spawn_tick: tick,
            entered_tick: Some(tick),
            arrival_tick: None,
            updated_tick: tick,
            wait_ticks: 0,
            reroute_count: 0,
        }
    }

    pub fn current_segment(&self) -> Option<SegmentId> {
        self.route.segments.get(self.route_index).copied()
    }

    pub fn next_segment(&self) -> Option<SegmentId> {
        self.route.segments.get(self.route_index + 1).copied()
    }
}
