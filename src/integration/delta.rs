use crate::integration::events::SimulationEvent;
use crate::integration::snapshots::{MetricsSnapshot, SignalSnapshot, VehicleSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapshotDelta {
    pub tick: u64,
    pub vehicles: Vec<VehicleSnapshot>,
    pub signals: Vec<SignalSnapshot>,
    pub metrics: MetricsSnapshot,
    pub events: Vec<SimulationEvent>,
}
