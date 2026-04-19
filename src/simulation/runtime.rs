use std::collections::VecDeque;

use crate::model::VehicleId;

#[derive(Debug, Clone)]
pub(super) struct RoadRuntime {
    pub(super) lanes: Vec<VecDeque<VehicleId>>,
}

impl RoadRuntime {
    pub(super) fn new(lanes: usize) -> Self {
        Self {
            lanes: (0..lanes).map(|_| VecDeque::new()).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct SignalRuntime {
    pub(super) phase_index: usize,
    pub(super) time_in_phase: u32,
}