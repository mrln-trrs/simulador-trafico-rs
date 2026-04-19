use std::cmp::Reverse;

use crate::model::{NodeId, NodeKind, RoadId, VehicleId, VehicleStatus};

use super::{
    events::SimulationEvent,
    routing::RouteSearchOptions,
    Simulation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct QueueCandidate {
    queue_index: usize,
    vehicle_id: VehicleId,
    next_road_id: Option<RoadId>,
    waiting_ticks: u32,
    priority_key: (u8, Reverse<u32>, usize, VehicleId),
}

impl Simulation {
    pub(super) fn release_node_queues(&mut self, events: &mut Vec<SimulationEvent>) {
        let mut node_ids = self.node_queues.keys().copied().collect::<Vec<_>>();
        node_ids.sort_unstable();

        for node_id in node_ids {
            loop {
                if let Some(candidate) = self.select_completion_candidate(node_id) {
                    self.complete_queue_vehicle(node_id, candidate, events);
                    continue;
                }

                let Some(candidate) = self.select_departable_candidate(node_id) else {
                    let Some(candidate) = self.select_deadlock_candidate(node_id) else {
                        break;
                    };

                    let Some(next_road_id) = candidate.next_road_id else {
                        break;
                    };

                    if self.try_deadlock_escape(
                        node_id,
                        candidate.queue_index,
                        candidate.vehicle_id,
                        next_road_id,
                        candidate.waiting_ticks,
                        events,
                    ) {
                        continue;
                    }

                    break;
                };

                let Some(next_road_id) = candidate.next_road_id else {
                    continue;
                };

                let lane_index = self
                    .find_lane_with_capacity(next_road_id)
                    .expect("selected candidate should have free capacity");

                self.node_queues
                    .get_mut(&node_id)
                    .expect("node queue should exist")
                    .remove(candidate.queue_index);
                self.enter_road(candidate.vehicle_id, next_road_id, lane_index, events);
            }
        }
    }

    fn complete_queue_vehicle(
        &mut self,
        node_id: NodeId,
        candidate: QueueCandidate,
        events: &mut Vec<SimulationEvent>,
    ) {
        self.node_queues
            .get_mut(&node_id)
            .expect("node queue should exist")
            .remove(candidate.queue_index);

        let vehicle = self
            .vehicles
            .get_mut(&candidate.vehicle_id)
            .expect("vehicle should exist while it is queued");
        vehicle.status = VehicleStatus::Finished;
        events.push(SimulationEvent::Completed {
            vehicle_id: candidate.vehicle_id,
            destination: vehicle.destination,
            wait_time: vehicle.total_wait_time,
            travel_time: vehicle.total_travel_time,
        });
    }

    fn queue_candidate(&self, queue_index: usize, vehicle_id: VehicleId) -> Option<QueueCandidate> {
        let vehicle = self.vehicles.get(&vehicle_id)?;
        if self.current_tick < vehicle.queued_release_tick || vehicle.waiting_ticks_at_node == 0 {
            return None;
        }

        Some(QueueCandidate {
            queue_index,
            vehicle_id,
            next_road_id: vehicle.route.get(vehicle.next_road_index).copied(),
            waiting_ticks: vehicle.waiting_ticks_at_node,
            priority_key: (
                vehicle.vehicle_type.priority_rank(),
                Reverse(vehicle.waiting_ticks_at_node),
                queue_index,
                vehicle_id,
            ),
        })
    }

    fn select_completion_candidate(&self, node_id: NodeId) -> Option<QueueCandidate> {
        let queue = self.node_queues.get(&node_id)?;

        queue
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(queue_index, vehicle_id)| {
                let candidate = self.queue_candidate(queue_index, vehicle_id)?;
                candidate.next_road_id.is_none().then_some(candidate)
            })
            .min_by_key(|candidate| candidate.priority_key)
    }

    fn select_departable_candidate(&self, node_id: NodeId) -> Option<QueueCandidate> {
        let queue = self.node_queues.get(&node_id)?;

        queue
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(queue_index, vehicle_id)| {
                let candidate = self.queue_candidate(queue_index, vehicle_id)?;
                let next_road_id = candidate.next_road_id?;

                if !self.can_depart(node_id, next_road_id) {
                    return None;
                }

                self.find_lane_with_capacity(next_road_id).map(|_| candidate)
            })
            .min_by_key(|candidate| candidate.priority_key)
    }

    fn select_deadlock_candidate(&self, node_id: NodeId) -> Option<QueueCandidate> {
        let queue = self.node_queues.get(&node_id)?;

        queue
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(queue_index, vehicle_id)| {
                let candidate = self.queue_candidate(queue_index, vehicle_id)?;
                if candidate.waiting_ticks < self.deadlock_wait_threshold {
                    return None;
                }

                if candidate.next_road_id.is_none() {
                    return None;
                }

                Some(candidate)
            })
            .min_by_key(|candidate| candidate.priority_key)
    }

    fn try_deadlock_escape(
        &mut self,
        node_id: NodeId,
        queue_index: usize,
        vehicle_id: VehicleId,
        avoided_road: RoadId,
        waiting_ticks: u32,
        events: &mut Vec<SimulationEvent>,
    ) -> bool {
        let Some(destination) = self.vehicles.get(&vehicle_id).map(|vehicle| vehicle.destination) else {
            return false;
        };

        let search_options = RouteSearchOptions {
            avoid_road: Some(avoided_road),
            avoid_full_roads: true,
            respect_signals: true,
        };

        let new_route = self
            .compute_route_from(node_id, destination, search_options)
            .or_else(|| {
                self.compute_route_from(
                    node_id,
                    destination,
                    RouteSearchOptions {
                        respect_signals: false,
                        ..search_options
                    },
                )
            });

        let Some(new_route) = new_route else {
            return self.force_emergency_release(
                node_id,
                queue_index,
                vehicle_id,
                avoided_road,
                waiting_ticks,
                events,
            );
        };

        if new_route.is_empty() {
            return false;
        }

        let next_road_id = new_route[0];

        {
            let vehicle = self
                .vehicles
                .get_mut(&vehicle_id)
                .expect("vehicle should exist while it is waiting at a node");
            vehicle.route = new_route.clone();
            vehicle.next_road_index = 0;
            vehicle.waiting_ticks_at_node = 0;
        }

        self.node_queues
            .get_mut(&node_id)
            .expect("node queue should exist")
            .remove(queue_index);

        events.push(SimulationEvent::Rerouted {
            vehicle_id,
            node_id,
            avoided_road,
            new_route: new_route.clone(),
            reason: format!("espera prolongada de {} ticks", waiting_ticks),
        });

        let lane_index = self
            .find_lane_for_emergency_release(next_road_id)
            .unwrap_or(0);
        self.enter_road_emergency(
            vehicle_id,
            next_road_id,
            lane_index,
            "ruta de emergencia".to_string(),
            events,
        );
        true
    }

    fn force_emergency_release(
        &mut self,
        node_id: NodeId,
        queue_index: usize,
        vehicle_id: VehicleId,
        avoided_road: RoadId,
        waiting_ticks: u32,
        events: &mut Vec<SimulationEvent>,
    ) -> bool {
        let Some(next_road_id) = self
            .vehicles
            .get(&vehicle_id)
            .and_then(|vehicle| vehicle.route.get(vehicle.next_road_index).copied())
        else {
            return false;
        };

        {
            let vehicle = self
                .vehicles
                .get_mut(&vehicle_id)
                .expect("vehicle should exist while it is waiting at a node");
            vehicle.waiting_ticks_at_node = 0;
        }

        self.node_queues
            .get_mut(&node_id)
            .expect("node queue should exist")
            .remove(queue_index);

        events.push(SimulationEvent::Rerouted {
            vehicle_id,
            node_id,
            avoided_road,
            new_route: self
                .vehicles
                .get(&vehicle_id)
                .map(|vehicle| vehicle.route.clone())
                .unwrap_or_default(),
            reason: format!("liberación forzada tras {} ticks", waiting_ticks),
        });

        let lane_index = self
            .find_lane_for_emergency_release(next_road_id)
            .unwrap_or(0);
        self.enter_road_emergency(
            vehicle_id,
            next_road_id,
            lane_index,
            "liberación de emergencia por atasco".to_string(),
            events,
        );
        true
    }

    pub(super) fn can_depart(&self, node_id: NodeId, road_id: RoadId) -> bool {
        let Some(node) = self.network.node(node_id) else {
            return false;
        };

        if !node.outbound_roads.contains(&road_id) {
            return false;
        }

        match node.kind {
            NodeKind::TrafficLight => {
                let Some(plan) = node.signal_plan.as_ref() else {
                    return true;
                };
                let Some(state) = self.signals.get(&node_id) else {
                    return true;
                };
                let phase = &plan.phases[state.phase_index];
                phase.allowed_roads.is_empty() || phase.allowed_roads.contains(&road_id)
            }
            _ => true,
        }
    }
}