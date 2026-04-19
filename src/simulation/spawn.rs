use crate::model::{Vehicle, VehicleSpawn};

use super::{events::SimulationEvent, routing::RouteSearchOptions, Simulation};

impl Simulation {
    pub fn schedule_spawn(&mut self, spawn: VehicleSpawn) {
        self.scheduled_spawns.push_back(spawn);
        self.scheduled_spawns
            .make_contiguous()
            .sort_by_key(|spawn| spawn.departure_tick);
    }

    pub(super) fn release_scheduled_spawns(&mut self, events: &mut Vec<SimulationEvent>) {
        while self
            .scheduled_spawns
            .front()
            .map(|spawn| spawn.departure_tick <= self.current_tick)
            .unwrap_or(false)
        {
            let spawn = self.scheduled_spawns.pop_front().expect("spawn should exist");
            self.spawn_vehicle(spawn, events);
        }
    }

    fn spawn_vehicle(&mut self, spawn: VehicleSpawn, events: &mut Vec<SimulationEvent>) {
        let VehicleSpawn {
            departure_tick: _,
            vehicle_type,
            origin,
            destination,
            name,
        } = spawn;

        let Some(route) = self.compute_route_from(
            origin,
            destination,
            RouteSearchOptions {
                avoid_road: None,
                avoid_full_roads: false,
                respect_signals: false,
            },
        ) else {
            events.push(SimulationEvent::SpawnFailed {
                name,
                origin,
                destination,
                reason: "no route found".to_string(),
            });
            return;
        };

        let vehicle_id = self.next_vehicle_id;
        self.next_vehicle_id += 1;

        let vehicle = Vehicle::with_type(
            vehicle_id,
            name.clone(),
            origin,
            destination,
            route.clone(),
            vehicle_type,
        );
        self.vehicles.insert(vehicle_id, vehicle);
        if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
            vehicle.queued_release_tick = self.current_tick + 1;
        }
        self.node_queues.entry(origin).or_default().push_back(vehicle_id);

        events.push(SimulationEvent::Spawned {
            vehicle_id,
            name,
            origin,
            destination,
            route,
        });
    }
}