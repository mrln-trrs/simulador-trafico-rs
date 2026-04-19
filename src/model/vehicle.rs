use super::{NodeId, RoadId, VehicleId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VehicleType {
    Emergency,
    Bus,
    Car,
    Truck,
    Motorcycle,
}

impl VehicleType {
    pub fn priority_rank(self) -> u8 {
        match self {
            VehicleType::Emergency => 0,
            VehicleType::Bus => 1,
            VehicleType::Car => 2,
            VehicleType::Truck => 3,
            VehicleType::Motorcycle => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VehicleStatus {
    WaitingAtNode(NodeId),
    OnRoad {
        road_id: RoadId,
        lane_index: usize,
        remaining_time: u32,
    },
    Finished,
}

#[derive(Clone, Debug)]
pub struct Vehicle {
    pub id: VehicleId,
    pub name: String,
    pub vehicle_type: VehicleType,
    pub origin: NodeId,
    pub destination: NodeId,
    pub route: Vec<RoadId>,
    pub next_road_index: usize,
    pub status: VehicleStatus,
    pub total_wait_time: u32,
    pub total_travel_time: u32,
    pub waiting_ticks_at_node: u32,
    pub queued_release_tick: u32,
}

impl Vehicle {
    pub fn new(
        id: VehicleId,
        name: impl Into<String>,
        origin: NodeId,
        destination: NodeId,
        route: Vec<RoadId>,
    ) -> Self {
        Self::with_type(id, name, origin, destination, route, VehicleType::Car)
    }

    pub fn with_type(
        id: VehicleId,
        name: impl Into<String>,
        origin: NodeId,
        destination: NodeId,
        route: Vec<RoadId>,
        vehicle_type: VehicleType,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            vehicle_type,
            origin,
            destination,
            route,
            next_road_index: 0,
            status: VehicleStatus::WaitingAtNode(origin),
            total_wait_time: 0,
            total_travel_time: 0,
            waiting_ticks_at_node: 0,
            queued_release_tick: 0,
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.status, VehicleStatus::Finished)
    }

    pub fn current_node(&self) -> Option<NodeId> {
        match self.status {
            VehicleStatus::WaitingAtNode(node_id) => Some(node_id),
            VehicleStatus::OnRoad { .. } | VehicleStatus::Finished => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VehicleSpawn {
    pub departure_tick: u32,
    pub vehicle_type: VehicleType,
    pub origin: NodeId,
    pub destination: NodeId,
    pub name: String,
}

impl VehicleSpawn {
    pub fn new(
        departure_tick: u32,
        name: impl Into<String>,
        origin: NodeId,
        destination: NodeId,
    ) -> Self {
        Self::with_type(departure_tick, name, origin, destination, VehicleType::Car)
    }

    pub fn with_type(
        departure_tick: u32,
        name: impl Into<String>,
        origin: NodeId,
        destination: NodeId,
        vehicle_type: VehicleType,
    ) -> Self {
        Self {
            departure_tick,
            vehicle_type,
            origin,
            destination,
            name: name.into(),
        }
    }
}