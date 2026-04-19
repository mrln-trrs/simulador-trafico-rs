use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

use crate::model::{NodeId, RoadId};

use super::Simulation;

#[derive(Clone, Copy, Debug)]
pub(super) struct RouteSearchOptions {
    pub(super) avoid_road: Option<RoadId>,
    pub(super) avoid_full_roads: bool,
    pub(super) respect_signals: bool,
}

impl Simulation {
    pub(super) fn compute_route_from(
        &self,
        start: NodeId,
        goal: NodeId,
        options: RouteSearchOptions,
    ) -> Option<Vec<RoadId>> {
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

            let Some(node) = self.network.node(node_id) else {
                continue;
            };

            for road_id in &node.outbound_roads {
                if options.avoid_road == Some(*road_id) {
                    continue;
                }

                let Some(road) = self.network.road(*road_id) else {
                    continue;
                };

                if options.avoid_full_roads && self.road_is_full(*road_id) {
                    continue;
                }

                if options.respect_signals && node_id == start && !self.can_depart(node_id, *road_id) {
                    continue;
                }

                let next_cost = cost.saturating_add(self.dynamic_road_cost(*road_id));
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

    pub(super) fn dynamic_road_cost(&self, road_id: RoadId) -> u32 {
        let Some(road) = self.network.road(road_id) else {
            return 1;
        };
        let Some(runtime) = self.road_runtime.get(&road_id) else {
            return road.travel_time_seconds();
        };

        let occupancy = runtime.lanes.iter().map(|lane| lane.len()).sum::<usize>() as u64;
        let capacity = road.total_capacity().max(1) as u64;
        let base = road.travel_time_seconds() as u64;
        let congestion_penalty = (occupancy.saturating_mul(base)) / capacity;
        base.saturating_add(congestion_penalty).min(u32::MAX as u64) as u32
    }

    pub(super) fn road_is_full(&self, road_id: RoadId) -> bool {
        let Some(road) = self.network.road(road_id) else {
            return true;
        };
        let Some(runtime) = self.road_runtime.get(&road_id) else {
            return true;
        };

        let occupancy = runtime.lanes.iter().map(|lane| lane.len()).sum::<usize>();
        occupancy >= road.total_capacity()
    }
}