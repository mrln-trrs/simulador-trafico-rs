use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    TickAdvanced,
    VehicleSpawned,
    VehicleMoved,
    VehicleQueued,
    VehicleArrived,
    VehicleRerouted,
    SignalChanged,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationEvent {
    pub tick: u64,
    pub kind: EventKind,
    pub entity: Option<String>,
    pub message: String,
}
