use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};
use std::fmt;

use crate::model::{Network, NodeId, NodeKind, RoadId, Vehicle, VehicleId, VehicleSpawn, VehicleStatus};

const DEFAULT_DEADLOCK_WAIT_THRESHOLD: u32 = 5;

#[derive(Clone, Copy, Debug)]
struct RouteSearchOptions {
    avoid_road: Option<RoadId>,
    avoid_full_roads: bool,
    respect_signals: bool,
}

#[derive(Debug, Clone)]
struct RoadRuntime {
    lanes: Vec<VecDeque<VehicleId>>,
}

impl RoadRuntime {
    fn new(lanes: usize) -> Self {
        Self {
            lanes: (0..lanes).map(|_| VecDeque::new()).collect(),
        }
    }
}

#[derive(Debug, Clone)]
struct SignalRuntime {
    phase_index: usize,
    time_in_phase: u32,
}

#[derive(Debug, Clone)]
pub enum SimulationEvent {
    Spawned {
        vehicle_id: VehicleId,
        name: String,
        origin: NodeId,
        destination: NodeId,
        route: Vec<RoadId>,
    },
    SpawnFailed {
        name: String,
        origin: NodeId,
        destination: NodeId,
        reason: String,
    },
    SignalChanged {
        node_id: NodeId,
        phase_name: String,
    },
    EnteredRoad {
        vehicle_id: VehicleId,
        road_id: RoadId,
        lane_index: usize,
    },
    Rerouted {
        vehicle_id: VehicleId,
        node_id: NodeId,
        avoided_road: RoadId,
        new_route: Vec<RoadId>,
        reason: String,
    },
    EmergencyEnteredRoad {
        vehicle_id: VehicleId,
        road_id: RoadId,
        lane_index: usize,
        reason: String,
    },
    ReachedNode {
        vehicle_id: VehicleId,
        node_id: NodeId,
    },
    Completed {
        vehicle_id: VehicleId,
        destination: NodeId,
        wait_time: u32,
        travel_time: u32,
    },
}

#[derive(Debug, Clone)]
pub struct SimulationReport {
    pub ticks_run: u32,
    pub vehicles_total: usize,
    pub vehicles_completed: usize,
    pub vehicles_active: usize,
    pub average_wait_time: f64,
    pub average_travel_time: f64,
}

impl fmt::Display for SimulationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ticks={}, total={}, completed={}, active={}, wait_avg={:.2}, travel_avg={:.2}",
            self.ticks_run,
            self.vehicles_total,
            self.vehicles_completed,
            self.vehicles_active,
            self.average_wait_time,
            self.average_travel_time,
        )
    }
}

pub struct Simulation {
    pub network: Network,
    vehicles: BTreeMap<VehicleId, Vehicle>,
    road_runtime: BTreeMap<RoadId, RoadRuntime>,
    node_queues: BTreeMap<NodeId, VecDeque<VehicleId>>,
    signals: BTreeMap<NodeId, SignalRuntime>,
    signal_nodes: Vec<NodeId>,
    scheduled_spawns: VecDeque<VehicleSpawn>,
    current_tick: u32,
    next_vehicle_id: VehicleId,
    deadlock_wait_threshold: u32,
}

impl Simulation {
    pub fn new(network: Network) -> Self {
        let road_runtime = network
            .roads()
            .map(|road| (road.id, RoadRuntime::new(road.lanes)))
            .collect::<BTreeMap<_, _>>();

        let signal_nodes = network
            .nodes()
            .filter(|node| node.signal_plan.is_some())
            .map(|node| node.id)
            .collect::<Vec<_>>();
        let mut signal_nodes = signal_nodes;
        signal_nodes.sort_unstable();

        let signals = network
            .nodes()
            .filter_map(|node| {
                node.signal_plan.as_ref().map(|_| {
                    (
                        node.id,
                        SignalRuntime {
                            phase_index: 0,
                            time_in_phase: 0,
                        },
                    )
                })
            })
            .collect::<BTreeMap<_, _>>();

        Self {
            network,
            vehicles: BTreeMap::new(),
            road_runtime,
            node_queues: BTreeMap::new(),
            signals,
            signal_nodes,
            scheduled_spawns: VecDeque::new(),
            current_tick: 0,
            next_vehicle_id: 1,
            deadlock_wait_threshold: DEFAULT_DEADLOCK_WAIT_THRESHOLD,
        }
    }

    pub fn set_deadlock_wait_threshold(&mut self, ticks: u32) {
        assert!(ticks > 0, "deadlock threshold must be positive");
        self.deadlock_wait_threshold = ticks;
    }

    pub fn schedule_spawn(&mut self, spawn: VehicleSpawn) {
        self.scheduled_spawns.push_back(spawn);
        self.scheduled_spawns
            .make_contiguous()
            .sort_by_key(|spawn| spawn.departure_tick);
    }

    pub fn tick(&self) -> u32 {
        self.current_tick
    }

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

    pub fn run_until_idle(&mut self, max_ticks: u32) -> Vec<SimulationEvent> {
        let mut events = Vec::new();
        for _ in 0..max_ticks {
            let step_events = self.step();
            events.extend(step_events);
            if self.is_idle() {
                break;
            }
        }
        events
    }

    pub fn step(&mut self) -> Vec<SimulationEvent> {
        let mut events = Vec::new();

        self.release_scheduled_spawns(&mut events);
        self.increment_time_counters();
        self.advance_signals(&mut events);
        self.advance_road_fronts(&mut events);
        self.release_node_queues(&mut events);

        self.current_tick += 1;
        events
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

    fn release_scheduled_spawns(&mut self, events: &mut Vec<SimulationEvent>) {
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

        let vehicle = Vehicle::new(vehicle_id, name.clone(), origin, destination, route.clone());
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

    fn increment_time_counters(&mut self) {
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

    fn advance_signals(&mut self, events: &mut Vec<SimulationEvent>) {
        let signal_nodes = self.signal_nodes.clone();

        for node_id in signal_nodes {
            let Some(node) = self.network.node(node_id) else {
                continue;
            };
            let Some(plan) = node.signal_plan.as_ref() else {
                continue;
            };
            let Some(state) = self.signals.get(&node_id) else {
                continue;
            };

            let phase_count = plan.phases.len();
            let phase_duration = plan.phases[state.phase_index].duration;

            let state = self
                .signals
                .get_mut(&node_id)
                .expect("signal state should exist for signal nodes");
            state.time_in_phase += 1;

            if state.time_in_phase >= phase_duration {
                state.time_in_phase = 0;
                state.phase_index = (state.phase_index + 1) % phase_count;
                let next_phase_name = plan.phases[state.phase_index].name.clone();
                events.push(SimulationEvent::SignalChanged {
                    node_id,
                    phase_name: next_phase_name,
                });
            }
        }
    }

    fn advance_road_fronts(&mut self, events: &mut Vec<SimulationEvent>) {
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

    fn release_node_queues(&mut self, events: &mut Vec<SimulationEvent>) {
        let mut node_ids = self.node_queues.keys().copied().collect::<Vec<_>>();
        node_ids.sort_unstable();

        for node_id in node_ids {
            loop {
                let Some(vehicle_id) = self
                    .node_queues
                    .get(&node_id)
                    .and_then(|queue| queue.front().copied())
                else {
                    break;
                };

                let waiting_ticks = self
                    .vehicles
                    .get(&vehicle_id)
                    .map(|vehicle| vehicle.waiting_ticks_at_node)
                    .unwrap_or(0);

                let queued_release_tick = self
                    .vehicles
                    .get(&vehicle_id)
                    .map(|vehicle| vehicle.queued_release_tick)
                    .unwrap_or(0);

                if self.current_tick < queued_release_tick {
                    break;
                }

                if waiting_ticks == 0 {
                    break;
                }

                let maybe_next_road = {
                    let vehicle = self
                        .vehicles
                        .get(&vehicle_id)
                        .expect("vehicle should exist while it is queued");
                    if vehicle.next_road_index >= vehicle.route.len() {
                        None
                    } else {
                        Some(vehicle.route[vehicle.next_road_index])
                    }
                };

                let Some(next_road_id) = maybe_next_road else {
                    self.node_queues
                        .get_mut(&node_id)
                        .expect("node queue should exist")
                        .pop_front();
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
                    continue;
                };

                if waiting_ticks >= self.deadlock_wait_threshold {
                    if self.try_deadlock_escape(node_id, vehicle_id, next_road_id, waiting_ticks, events) {
                        continue;
                    }
                }

                if !self.can_depart(node_id, next_road_id) {
                    break;
                }

                let Some(lane_index) = self.find_lane_with_capacity(next_road_id) else {
                    break;
                };

                self.node_queues
                    .get_mut(&node_id)
                    .expect("node queue should exist")
                    .pop_front();
                self.enter_road(vehicle_id, next_road_id, lane_index, events);
            }
        }
    }

    fn try_deadlock_escape(
        &mut self,
        node_id: NodeId,
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
            return self.force_emergency_release(node_id, vehicle_id, avoided_road, waiting_ticks, events);
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
            .pop_front();

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
            .pop_front();

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

    fn can_depart(&self, node_id: NodeId, road_id: RoadId) -> bool {
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

    fn compute_route_from(
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

    fn dynamic_road_cost(&self, road_id: RoadId) -> u32 {
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

    fn road_is_full(&self, road_id: RoadId) -> bool {
        let Some(road) = self.network.road(road_id) else {
            return true;
        };
        let Some(runtime) = self.road_runtime.get(&road_id) else {
            return true;
        };

        let occupancy = runtime.lanes.iter().map(|lane| lane.len()).sum::<usize>();
        occupancy >= road.total_capacity()
    }

    fn find_lane_with_capacity(&self, road_id: RoadId) -> Option<usize> {
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

    fn find_lane_for_emergency_release(&self, road_id: RoadId) -> Option<usize> {
        let runtime = self.road_runtime.get(&road_id)?;

        runtime
            .lanes
            .iter()
            .enumerate()
            .min_by_key(|(lane_index, lane)| (lane.len(), *lane_index))
            .map(|(lane_index, _)| lane_index)
    }

    fn enter_road(
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

    fn enter_road_emergency(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Node, NodeKind, RoadSegment, VehicleSpawn};
    use std::collections::VecDeque;

    #[test]
    fn vehicle_completes_a_simple_route() {
        let mut network = Network::new();
        network.add_node(Node::new(1, "Origen", NodeKind::Entry));
        network.add_node(Node::new(2, "Destino", NodeKind::Exit));
        network.add_road(RoadSegment::new(1, "Camino", 1, 2, 60.0, 1, 60.0, 1));

        let mut simulation = Simulation::new(network);
        simulation.schedule_spawn(VehicleSpawn::new(0, "Auto 1", 1, 2));

        simulation.run_until_idle(10);

        assert_eq!(simulation.completed_vehicle_count(), 1);
        assert!(simulation.is_idle());
    }

    #[test]
    fn vehicle_waits_at_least_one_tick_before_departing_a_node() {
        let mut network = Network::new();
        network.add_node(Node::new(1, "Origen", NodeKind::Entry));
        network.add_node(Node::new(2, "Destino", NodeKind::Exit));
        network.add_road(RoadSegment::new(1, "Camino", 1, 2, 60.0, 1, 60.0, 1));

        let mut simulation = Simulation::new(network);
        simulation.schedule_spawn(VehicleSpawn::new(0, "Auto 1", 1, 2));

        let first_tick_events = simulation.step();
        assert!(first_tick_events.iter().any(|event| matches!(event, SimulationEvent::Spawned { .. })));
        assert_eq!(simulation.active_vehicle_count(), 1);
        assert_eq!(simulation.completed_vehicle_count(), 0);

        let second_tick_events = simulation.step();
        assert!(second_tick_events.iter().any(|event| matches!(event, SimulationEvent::EnteredRoad { .. })));
    }

    #[test]
    fn deadlock_threshold_triggers_emergency_escape() {
        let mut network = Network::new();
        network.add_node(Node::new(1, "Origen", NodeKind::Entry));
        network.add_node(Node::new(2, "Interseccion", NodeKind::Intersection));
        network.add_node(Node::new(3, "Destino", NodeKind::Exit));
        network.add_node(Node::new(4, "Desvio", NodeKind::Intersection));

        network.add_road(RoadSegment::new(1, "Principal", 1, 2, 60.0, 1, 30.0, 1));
        network.add_road(RoadSegment::new(2, "Atascada", 2, 3, 60.0, 1, 30.0, 1));
        network.add_road(RoadSegment::new(3, "Desvio", 2, 4, 60.0, 1, 30.0, 1));
        network.add_road(RoadSegment::new(4, "Retorno", 4, 3, 60.0, 1, 30.0, 1));

        let mut simulation = Simulation::new(network);
        simulation.set_deadlock_wait_threshold(2);

        let mut vehicle = Vehicle::new(1, "Auto 1", 1, 3, vec![2]);
        vehicle.status = VehicleStatus::WaitingAtNode(2);
        vehicle.waiting_ticks_at_node = 2;
        vehicle.queued_release_tick = 0;

        simulation.vehicles.insert(1, vehicle);
        simulation.node_queues.insert(2, VecDeque::from([1]));

        if let Some(runtime) = simulation.road_runtime.get_mut(&2) {
            runtime.lanes[0].push_back(99);
        }

        let mut events = Vec::new();
        simulation.release_node_queues(&mut events);

        assert!(events.iter().any(|event| matches!(event, SimulationEvent::Rerouted { .. })));
        assert!(events.iter().any(|event| matches!(event, SimulationEvent::EmergencyEnteredRoad { .. })));

        let vehicle = simulation.vehicles.get(&1).expect("vehicle should still exist");
        assert!(matches!(vehicle.status, VehicleStatus::OnRoad { road_id: 3, .. }));
    }
}
