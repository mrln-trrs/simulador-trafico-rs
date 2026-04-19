use std::collections::BTreeMap;

use super::node::Node;
use super::road::RoadSegment;
use super::{routing, NodeId, RoadId};

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
        routing::shortest_route(self, start, goal)
    }

    pub fn shortest_route_filtered<F>(
        &self,
        start: NodeId,
        goal: NodeId,
        allow_road: F,
    ) -> Option<Vec<RoadId>>
    where
        F: FnMut(RoadId, &RoadSegment) -> bool,
    {
        routing::shortest_route_filtered(self, start, goal, allow_road)
    }
}