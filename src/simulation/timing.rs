use super::Simulation;

impl Simulation {
    pub(super) fn increment_time_counters(&mut self) {
        let node_vehicle_ids = self
            .node_queues
            .values()
            .flat_map(|queue| queue.iter().copied())
            .collect::<Vec<_>>();
        for vehicle_id in node_vehicle_ids {
            if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
                if !vehicle.is_finished() {
                    if self.current_tick >= vehicle.queued_release_tick {
                        vehicle.total_wait_time += 1;
                        vehicle.waiting_ticks_at_node += 1;
                    }
                }
            }
        }

        let road_vehicle_ids = self
            .road_runtime
            .values()
            .flat_map(|runtime| runtime.lanes.iter().flat_map(|lane| lane.iter().copied()))
            .collect::<Vec<_>>();
        for vehicle_id in road_vehicle_ids {
            if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
                if !vehicle.is_finished() {
                    vehicle.total_travel_time += 1;
                }
            }
        }
    }
}