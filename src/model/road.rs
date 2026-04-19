use super::{NodeId, RoadId};

#[derive(Clone, Debug)]
pub struct RoadSegment {
    pub id: RoadId,
    pub name: String,
    pub from: NodeId,
    pub to: NodeId,
    pub length_m: f64,
    pub lanes: usize,
    pub speed_limit_kmh: f64,
    pub capacity_per_lane: usize,
}

impl RoadSegment {
    pub fn new(
        id: RoadId,
        name: impl Into<String>,
        from: NodeId,
        to: NodeId,
        length_m: f64,
        lanes: usize,
        speed_limit_kmh: f64,
        capacity_per_lane: usize,
    ) -> Self {
        assert!(length_m > 0.0, "road length must be positive");
        assert!(lanes > 0, "roads need at least one lane");
        assert!(speed_limit_kmh > 0.0, "speed limits must be positive");
        assert!(capacity_per_lane > 0, "lane capacity must be positive");
        Self {
            id,
            name: name.into(),
            from,
            to,
            length_m,
            lanes,
            speed_limit_kmh,
            capacity_per_lane,
        }
    }

    pub fn travel_time_seconds(&self) -> u32 {
        let meters_per_second = self.speed_limit_kmh * 1000.0 / 3600.0;
        let seconds = (self.length_m / meters_per_second).ceil() as u32;
        seconds.max(1)
    }

    pub fn total_capacity(&self) -> usize {
        self.lanes * self.capacity_per_lane
    }
}