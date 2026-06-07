use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    Source,
    Intersection,
    Signal,
    Sink,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignalPhase {
    Red,
    Yellow,
    Green,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum VehicleKind {
    Car,
    Bus,
    Truck,
    Emergency,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum VehicleState {
    WaitingToSpawn,
    Queued,
    Moving,
    WaitingForSignal,
    Arrived,
}
