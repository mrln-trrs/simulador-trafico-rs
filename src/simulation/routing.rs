use crate::model::graph::{Graph, RoadSegment};
use crate::model::ids::{NodeId, SegmentId};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};

#[derive(Clone, Copy, Debug, PartialEq)]
struct QueueState {
    cost: f64,
    node: NodeId,
}

impl Eq for QueueState {}

impl Ord for QueueState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .total_cmp(&self.cost)
            .then_with(|| other.node.cmp(&self.node))
    }
}

impl PartialOrd for QueueState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn segment_cost(segment: &RoadSegment, occupancy: usize) -> f64 {
    let capacity = segment.capacity.max(1) as f64;
    let occupancy_ratio = occupancy as f64 / capacity;
    let travel_time = segment.length_m / segment.speed_limit_mps.max(0.1);
    travel_time * (1.0 + occupancy_ratio * 0.75)
}

pub fn shortest_route(
    graph: &Graph,
    origin: NodeId,
    destination: NodeId,
    occupancy: &BTreeMap<SegmentId, usize>,
) -> Option<Vec<SegmentId>> {
    if origin == destination {
        return Some(Vec::new());
    }

    let mut distances: BTreeMap<NodeId, f64> = BTreeMap::new();
    let mut previous: BTreeMap<NodeId, (NodeId, SegmentId)> = BTreeMap::new();
    let mut heap = BinaryHeap::new();

    distances.insert(origin, 0.0);
    heap.push(QueueState {
        cost: 0.0,
        node: origin,
    });

    while let Some(QueueState { cost, node }) = heap.pop() {
        if node == destination {
            break;
        }

        if cost > *distances.get(&node).unwrap_or(&f64::INFINITY) {
            continue;
        }

        for segment_id in graph.outgoing_segments(node) {
            if let Some(segment) = graph.segment(*segment_id) {
                let next_cost =
                    cost + segment_cost(segment, *occupancy.get(segment_id).unwrap_or(&0));
                let entry = distances.entry(segment.to).or_insert(f64::INFINITY);
                if next_cost < *entry {
                    *entry = next_cost;
                    previous.insert(segment.to, (node, *segment_id));
                    heap.push(QueueState {
                        cost: next_cost,
                        node: segment.to,
                    });
                }
            }
        }
    }

    if !previous.contains_key(&destination) {
        return None;
    }

    let mut route = Vec::new();
    let mut current = destination;
    while current != origin {
        let (parent, segment) = previous.get(&current).copied()?;
        route.push(segment);
        current = parent;
    }
    route.reverse();
    Some(route)
}
