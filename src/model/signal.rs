use crate::model::ids::{NodeId, SignalId};
use crate::model::state::SignalPhase;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignalTiming {
    pub green_ticks: u32,
    pub yellow_ticks: u32,
    pub red_ticks: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrafficSignal {
    pub id: SignalId,
    pub node: NodeId,
    pub timing: SignalTiming,
    pub phase: SignalPhase,
    pub phase_tick: u32,
}

impl TrafficSignal {
    pub fn new(id: SignalId, node: NodeId, timing: SignalTiming) -> Self {
        Self {
            id,
            node,
            timing,
            phase: SignalPhase::Red,
            phase_tick: 0,
        }
    }
}
