use crate::model::{RoadId, VehicleId, VehicleStatus};

use super::{events::SimulationEvent, Simulation};

impl Simulation {
    pub(super) fn advance_road_fronts(&mut self, events: &mut Vec<SimulationEvent>) {
        let road_ids = self.road_runtime.keys().copied().collect::<Vec<_>>();

        for road_id in road_ids {
            let lane_count = self
                .road_runtime
                .get(&road_id)
                .map(|runtime| runtime.lanes.len())
                .unwrap_or(0);

            for lane_index in 0..lane_count {
                let Some(vehicle_id) = self
                    .road_runtime
                    .get(&road_id)
                    .and_then(|runtime| runtime.lanes[lane_index].front().copied())
                else {
                    continue;
                };

                let Some(road) = self.network.road(road_id) else {
                    continue;
                };

                let ready_to_exit = {
                    let vehicle = self
                        .vehicles
                        .get_mut(&vehicle_id)
                        .expect("vehicle should exist while it is on a road");
                    match &mut vehicle.status {
                        VehicleStatus::OnRoad {
                            road_id: current_road,
                            lane_index: current_lane,
                            remaining_time,
                        } if *current_road == road_id && *current_lane == lane_index => {
                            if *remaining_time > 0 {
                                *remaining_time -= 1;
                            }
                            *remaining_time == 0
                        }
                        _ => false,
                    }
                };

                if !ready_to_exit {
                    continue;
                }

                self.road_runtime
                    .get_mut(&road_id)
                    .expect("road runtime should exist")
                    .lanes[lane_index]
                    .pop_front();

                let reached_node = road.to;
                let next_road_index = self
                    .vehicles
                    .get(&vehicle_id)
                    .expect("vehicle should exist")
                    .next_road_index;

                if next_road_index >= self.vehicles.get(&vehicle_id).expect("vehicle should exist").route.len() {
                    let vehicle = self
                        .vehicles
                        .get_mut(&vehicle_id)
                        .expect("vehicle should exist");
                    vehicle.status = VehicleStatus::Finished;
                    events.push(SimulationEvent::Completed {
                        vehicle_id,
                        destination: vehicle.destination,
                        wait_time: vehicle.total_wait_time,
                        travel_time: vehicle.total_travel_time,
                    });
                } else {
                    let vehicle = self
                        .vehicles
                        .get_mut(&vehicle_id)
                        .expect("vehicle should exist");
                    vehicle.status = VehicleStatus::WaitingAtNode(reached_node);
                    vehicle.queued_release_tick = self.current_tick + 1;
                    vehicle.waiting_ticks_at_node = 0;
                    self.node_queues
                        .entry(reached_node)
                        .or_default()
                        .push_back(vehicle_id);
                    events.push(SimulationEvent::ReachedNode {
                        vehicle_id,
                        node_id: reached_node,
                    });
                }
            }
        }
    }

    pub(super) fn find_lane_with_capacity(&self, road_id: RoadId) -> Option<usize> {
        let road = self.network.road(road_id)?;
        let runtime = self.road_runtime.get(&road_id)?;

        runtime
            .lanes
            .iter()
            .enumerate()
            .filter(|(_, lane)| lane.len() < road.capacity_per_lane)
            .min_by_key(|(lane_index, lane)| (lane.len(), *lane_index))
            .map(|(lane_index, _)| lane_index)
    }

    pub(super) fn find_lane_for_emergency_release(&self, road_id: RoadId) -> Option<usize> {
        let runtime = self.road_runtime.get(&road_id)?;

        runtime
            .lanes
            .iter()
            .enumerate()
            .min_by_key(|(lane_index, lane)| (lane.len(), *lane_index))
            .map(|(lane_index, _)| lane_index)
    }

    pub(super) fn enter_road(
        &mut self,
        vehicle_id: VehicleId,
        road_id: RoadId,
        lane_index: usize,
        events: &mut Vec<SimulationEvent>,
    ) {
        let travel_time = self
            .network
            .road(road_id)
            .expect("road should exist")
            .travel_time_seconds();

        self.road_runtime
            .get_mut(&road_id)
            .expect("road runtime should exist")
            .lanes[lane_index]
            .push_back(vehicle_id);

        let vehicle = self
            .vehicles
            .get_mut(&vehicle_id)
            .expect("vehicle should exist");
        vehicle.next_road_index += 1;
        vehicle.status = VehicleStatus::OnRoad {
            road_id,
            lane_index,
            remaining_time: travel_time,
        };

        events.push(SimulationEvent::EnteredRoad {
            vehicle_id,
            road_id,
            lane_index,
        });
    }

    pub(super) fn enter_road_emergency(
        &mut self,
        vehicle_id: VehicleId,
        road_id: RoadId,
        lane_index: usize,
        reason: String,
        events: &mut Vec<SimulationEvent>,
    ) {
        let travel_time = self
            .network
            .road(road_id)
            .expect("road should exist")
            .travel_time_seconds();

        self.road_runtime
            .get_mut(&road_id)
            .expect("road runtime should exist")
            .lanes[lane_index]
            .push_back(vehicle_id);

        let vehicle = self
            .vehicles
            .get_mut(&vehicle_id)
            .expect("vehicle should exist");
        vehicle.next_road_index += 1;
        vehicle.status = VehicleStatus::OnRoad {
            road_id,
            lane_index,
            remaining_time: travel_time,
        };

        events.push(SimulationEvent::EmergencyEnteredRoad {
            vehicle_id,
            road_id,
            lane_index,
            reason,
        });
    }
}