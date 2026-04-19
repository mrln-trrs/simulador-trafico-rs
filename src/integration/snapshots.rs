use crate::integration::protocol::CONTRACT_VERSION;
use crate::model::graph::Point2;
use crate::model::ids::{NodeId, SegmentId, SignalId, VehicleId};
use crate::model::state::{NodeKind, SignalPhase, VehicleKind, VehicleState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub vehicles_spawned: u64,
    pub vehicles_completed: u64,
    pub total_travel_ticks: u64,
    pub total_wait_ticks: u64,
}

impl MetricsSnapshot {
    pub fn average_travel_ticks(&self) -> f64 {
        if self.vehicles_completed == 0 {
            0.0
        } else {
            self.total_travel_ticks as f64 / self.vehicles_completed as f64
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeSnapshot {
    pub id: NodeId,
    pub name: String,
    pub kind: NodeKind,
    pub position: Point2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SegmentSnapshot {
    pub id: SegmentId,
    pub name: String,
    pub from: NodeId,
    pub to: NodeId,
    pub lane_count: usize,
    pub length_m: f64,
    pub speed_limit_mps: f64,
    pub capacity: usize,
    pub occupancy: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VehicleSnapshot {
    pub id: VehicleId,
    pub label: String,
    pub kind: VehicleKind,
    pub state: VehicleState,
    pub route_index: usize,
    pub progress: f64,
    pub speed_mps: f64,
    pub current_segment: Option<SegmentId>,
    pub next_segment: Option<SegmentId>,
    pub wait_ticks: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignalSnapshot {
    pub id: SignalId,
    pub node: NodeId,
    pub phase: SignalPhase,
    pub phase_tick: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub contract_version: u32,
    pub tick: u64,
    pub running: bool,
    pub scenario_name: String,
    pub nodes: Vec<NodeSnapshot>,
    pub segments: Vec<SegmentSnapshot>,
    pub vehicles: Vec<VehicleSnapshot>,
    pub signals: Vec<SignalSnapshot>,
    pub metrics: MetricsSnapshot,
    pub pending_spawns: usize,
}

impl Snapshot {
    pub fn empty() -> Self {
        Self {
            contract_version: CONTRACT_VERSION,
            tick: 0,
            running: false,
            scenario_name: String::new(),
            nodes: Vec::new(),
            segments: Vec::new(),
            vehicles: Vec::new(),
            signals: Vec::new(),
            metrics: MetricsSnapshot {
                vehicles_spawned: 0,
                vehicles_completed: 0,
                total_travel_ticks: 0,
                total_wait_ticks: 0,
            },
            pending_spawns: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouteCostSnapshot {
    pub costs: BTreeMap<SegmentId, f64>,
}
