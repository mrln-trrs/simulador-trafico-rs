use super::{Simulation, SimulationReport};

impl Simulation {
    pub fn active_vehicle_count(&self) -> usize {
        self.vehicles.values().filter(|vehicle| !vehicle.is_finished()).count()
    }

    pub fn completed_vehicle_count(&self) -> usize {
        self.vehicles.values().filter(|vehicle| vehicle.is_finished()).count()
    }

    pub fn is_idle(&self) -> bool {
        self.scheduled_spawns.is_empty()
            && self.vehicles.values().all(|vehicle| vehicle.is_finished())
            && self.node_queues.values().all(|queue| queue.is_empty())
            && self
                .road_runtime
                .values()
                .all(|runtime| runtime.lanes.iter().all(|lane| lane.is_empty()))
    }

    pub fn report(&self) -> SimulationReport {
        let completed = self
            .vehicles
            .values()
            .filter(|vehicle| vehicle.is_finished())
            .collect::<Vec<_>>();

        let vehicles_completed = completed.len();
        let (average_wait_time, average_travel_time) = if vehicles_completed == 0 {
            (0.0, 0.0)
        } else {
            let wait_sum: u32 = completed.iter().map(|vehicle| vehicle.total_wait_time).sum();
            let travel_sum: u32 = completed.iter().map(|vehicle| vehicle.total_travel_time).sum();
            (
                wait_sum as f64 / vehicles_completed as f64,
                travel_sum as f64 / vehicles_completed as f64,
            )
        };

        SimulationReport {
            ticks_run: self.current_tick,
            vehicles_total: self.vehicles.len(),
            vehicles_completed,
            vehicles_active: self.active_vehicle_count(),
            average_wait_time,
            average_travel_time,
        }
    }
}