use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

use super::network::Network;
use super::road::RoadSegment;
use super::{NodeId, RoadId};

pub fn shortest_route(network: &Network, start: NodeId, goal: NodeId) -> Option<Vec<RoadId>> {
    shortest_route_filtered(network, start, goal, |_, _| true)
}

pub fn shortest_route_filtered<F>(
    network: &Network,
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

        let Some(node) = network.node(node_id) else {
            continue;
        };

        for road_id in &node.outbound_roads {
            let Some(road) = network.road(*road_id) else {
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

#[cfg(test)]
mod tests {
    use crate::model::{Network, Node, NodeKind, RoadSegment};

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