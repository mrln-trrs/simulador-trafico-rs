use crate::model::ids::{LaneId, SegmentId, VehicleId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lane {
    pub id: LaneId,
    pub segment: SegmentId,
    pub index: usize,
    pub capacity: usize,
    pub occupied: Vec<VehicleId>,
}

impl Lane {
    pub fn available_space(&self) -> usize {
        self.capacity.saturating_sub(self.occupied.len())
    }
}
