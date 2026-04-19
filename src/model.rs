use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

pub type NodeId = usize;
pub type RoadId = usize;
pub type VehicleId = usize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum NodeKind {
    Entry,
    Exit,
    Intersection,
    TrafficLight,
    Stop,
    Roundabout,
    Merge,
    Split,
}

#[derive(Clone, Debug)]
pub struct SignalPhase {
    pub name: String,
    pub allowed_roads: Vec<RoadId>,
    pub duration: u32,
}

impl SignalPhase {
    pub fn new(name: impl Into<String>, duration: u32, allowed_roads: Vec<RoadId>) -> Self {
        assert!(duration > 0, "signal phases must last at least one tick");
        Self {
            name: name.into(),
            allowed_roads,
            duration,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SignalPlan {
    pub phases: Vec<SignalPhase>,
}

impl SignalPlan {
    pub fn new(phases: Vec<SignalPhase>) -> Self {
        assert!(!phases.is_empty(), "signal plans need at least one phase");
        Self { phases }
    }
}

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

#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub name: String,
    pub kind: NodeKind,
    pub signal_plan: Option<SignalPlan>,
    pub inbound_roads: Vec<RoadId>,
    pub outbound_roads: Vec<RoadId>,
}

impl Node {
    pub fn new(id: NodeId, name: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            signal_plan: None,
            inbound_roads: Vec::new(),
            outbound_roads: Vec::new(),
        }
    }

    pub fn with_signal_plan(mut self, signal_plan: SignalPlan) -> Self {
        self.signal_plan = Some(signal_plan);
        self
    }
}

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

#[derive(Clone, Debug, Default)]
pub struct Network {
    nodes: BTreeMap<NodeId, Node>,
    roads: BTreeMap<RoadId, RoadSegment>,
}

impl Network {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn add_road(&mut self, road: RoadSegment) {
        assert!(
            self.nodes.contains_key(&road.from),
            "add the source node before adding road {}",
            road.id
        );
        assert!(
            self.nodes.contains_key(&road.to),
            "add the destination node before adding road {}",
            road.id
        );

        let road_id = road.id;
        let from = road.from;
        let to = road.to;

        self.nodes
            .get_mut(&from)
            .expect("source node should exist")
            .outbound_roads
            .push(road_id);
        self.nodes
            .get_mut(&to)
            .expect("destination node should exist")
            .inbound_roads
            .push(road_id);
        self.roads.insert(road_id, road);
    }

    pub fn node(&self, node_id: NodeId) -> Option<&Node> {
        self.nodes.get(&node_id)
    }

    pub fn road(&self, road_id: RoadId) -> Option<&RoadSegment> {
        self.roads.get(&road_id)
    }

    pub fn node_mut(&mut self, node_id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&node_id)
    }

    pub fn road_mut(&mut self, road_id: RoadId) -> Option<&mut RoadSegment> {
        self.roads.get_mut(&road_id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn road_count(&self) -> usize {
        self.roads.len()
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn roads(&self) -> impl Iterator<Item = &RoadSegment> {
        self.roads.values()
    }

    pub fn describe_route(&self, route: &[RoadId]) -> String {
        if route.is_empty() {
            return "sin tramos".to_string();
        }

        route
            .iter()
            .map(|road_id| {
                self.roads
                    .get(road_id)
                    .map(|road| road.name.as_str())
                    .unwrap_or("tramo desconocido")
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(" -> ")
    }

    pub fn shortest_route(&self, start: NodeId, goal: NodeId) -> Option<Vec<RoadId>> {
        self.shortest_route_filtered(start, goal, |_, _| true)
    }

    pub fn shortest_route_filtered<F>(
        &self,
        start: NodeId,
        goal: NodeId,
        mut allow_road: F,
    ) -> Option<Vec<RoadId>>
    where
        F: FnMut(RoadId, &RoadSegment) -> bool,
    {
        if start == goal {
            return Some(Vec::new());
        }

        let mut frontier: BinaryHeap<(Reverse<u32>, NodeId)> = BinaryHeap::new();
        let mut best_costs: BTreeMap<NodeId, u32> = BTreeMap::new();
        let mut came_from: BTreeMap<NodeId, (NodeId, RoadId)> = BTreeMap::new();

        frontier.push((Reverse(0), start));
        best_costs.insert(start, 0);

        while let Some((Reverse(cost), node_id)) = frontier.pop() {
            if node_id == goal {
                break;
            }

            if cost > *best_costs.get(&node_id).unwrap_or(&u32::MAX) {
                continue;
            }

            let Some(node) = self.nodes.get(&node_id) else {
                continue;
            };

            for road_id in &node.outbound_roads {
                let Some(road) = self.roads.get(road_id) else {
                    continue;
                };

                if !allow_road(*road_id, road) {
                    continue;
                }

                let next_cost = cost.saturating_add(road.travel_time_seconds());
                if next_cost < *best_costs.get(&road.to).unwrap_or(&u32::MAX) {
                    best_costs.insert(road.to, next_cost);
                    came_from.insert(road.to, (node_id, road.id));
                    frontier.push((Reverse(next_cost), road.to));
                }
            }
        }

        if !best_costs.contains_key(&goal) {
            return None;
        }

        let mut route = Vec::new();
        let mut current = goal;
        while current != start {
            let (previous, road_id) = came_from.get(&current).copied()?;
            route.push(road_id);
            current = previous;
        }

        route.reverse();
        Some(route)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortest_route_prefers_faster_path() {
        let mut network = Network::new();
        network.add_node(Node::new(1, "Origen", NodeKind::Entry));
        network.add_node(Node::new(2, "Intermedio", NodeKind::Intersection));
        network.add_node(Node::new(3, "Destino", NodeKind::Exit));

        network.add_road(RoadSegment::new(10, "Via A", 1, 2, 100.0, 1, 50.0, 1));
        network.add_road(RoadSegment::new(11, "Via B", 2, 3, 100.0, 1, 50.0, 1));
        network.add_road(RoadSegment::new(12, "Via directa", 1, 3, 300.0, 1, 50.0, 1));

        let route = network.shortest_route(1, 3).expect("route should exist");
        assert_eq!(route, vec![10, 11]);
        assert_eq!(network.describe_route(&route), "Via A -> Via B");
    }
}
