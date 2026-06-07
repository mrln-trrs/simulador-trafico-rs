use crate::model::graph::Graph;
use crate::model::ids::SignalId;
use crate::model::signal::TrafficSignal;
use crate::model::vehicle::VehicleSpawn;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub seed: u64,
    pub duration_ticks: u64,
    pub graph: Graph,
    pub spawns: Vec<VehicleSpawn>,
    pub signals: Vec<TrafficSignal>,
}

impl Scenario {
    pub fn signal_ids(&self) -> Vec<SignalId> {
        self.signals.iter().map(|signal| signal.id).collect()
    }
}
