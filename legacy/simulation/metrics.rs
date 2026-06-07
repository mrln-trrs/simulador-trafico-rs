use crate::integration::snapshots::MetricsSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub vehicles_spawned: u64,
    pub vehicles_completed: u64,
    pub total_travel_ticks: u64,
    pub total_wait_ticks: u64,
}

impl SimulationMetrics {
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            vehicles_spawned: self.vehicles_spawned,
            vehicles_completed: self.vehicles_completed,
            total_travel_ticks: self.total_travel_ticks,
            total_wait_ticks: self.total_wait_ticks,
        }
    }
}
