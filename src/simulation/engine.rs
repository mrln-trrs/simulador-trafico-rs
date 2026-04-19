use crate::integration::delta::SnapshotDelta;
use crate::integration::events::{EventKind, SimulationEvent};
use crate::integration::snapshots::{
    NodeSnapshot, SegmentSnapshot, SignalSnapshot, Snapshot, VehicleSnapshot,
};
use crate::model::ids::{NodeId, SegmentId, SignalId, VehicleId};
use crate::model::scenario::Scenario;
use crate::model::signal::TrafficSignal;
use crate::model::state::{SignalPhase, VehicleState};
use crate::model::vehicle::{Vehicle, VehicleRoute, VehicleSpawn};
use crate::simulation::movement::travel_progress;
use crate::simulation::routing::shortest_route;
use crate::simulation::tick::{normalize_progress, DEFAULT_GAP_RATIO};
use crate::simulation::validation::validate_engine_scenario;
use crate::simulation::SimulationMetrics;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Debug)]
pub struct SimulationEngine {
    scenario: Scenario,
    tick: u64,
    running: bool,
    vehicles: BTreeMap<VehicleId, Vehicle>,
    segment_queues: BTreeMap<SegmentId, VecDeque<VehicleId>>,
    signals: BTreeMap<SignalId, TrafficSignal>,
    pending_spawns: VecDeque<VehicleSpawn>,
    metrics: SimulationMetrics,
    events: Vec<SimulationEvent>,
    reroute_limit: u32,
}

impl SimulationEngine {
    pub fn new(scenario: Scenario) -> Result<Self, Vec<crate::model::ValidationIssue>> {
        validate_engine_scenario(&scenario)?;

        let mut signals = BTreeMap::new();
        for signal in &scenario.signals {
            signals.insert(signal.id, signal.clone());
        }

        let mut pending_spawns: VecDeque<_> = scenario.spawns.iter().cloned().collect();
        pending_spawns
            .make_contiguous()
            .sort_by_key(|spawn| (spawn.release_tick, spawn.id));

        let mut segment_queues = BTreeMap::new();
        for segment in &scenario.graph.segments {
            segment_queues.insert(segment.id, VecDeque::new());
        }

        Ok(Self {
            scenario,
            tick: 0,
            running: true,
            vehicles: BTreeMap::new(),
            segment_queues,
            signals,
            pending_spawns,
            metrics: SimulationMetrics::default(),
            events: Vec::new(),
            reroute_limit: 3,
        })
    }

    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn play(&mut self) {
        self.running = true;
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    pub fn reset(&mut self) {
        if let Ok(engine) = Self::new(self.scenario.clone()) {
            *self = engine;
        }
    }

    pub fn advance_tick(&mut self) -> Vec<SimulationEvent> {
        self.events.clear();
        self.advance_signals();
        self.spawn_due_vehicles();
        self.move_vehicles();
        self.tick += 1;
        self.events.push(SimulationEvent {
            tick: self.tick,
            kind: EventKind::TickAdvanced,
            entity: None,
            message: format!("Avance al tick {}", self.tick),
        });
        self.events.clone()
    }

    pub fn advance_many(&mut self, ticks: u32) -> Vec<SimulationEvent> {
        let mut all = Vec::new();
        for _ in 0..ticks {
            all.extend(self.advance_tick());
        }
        all
    }

    pub fn snapshot(&self) -> Snapshot {
        let nodes = self
            .scenario
            .graph
            .nodes
            .iter()
            .map(|node| NodeSnapshot {
                id: node.id,
                name: node.name.clone(),
                kind: node.kind,
                position: node.position,
            })
            .collect();

        let segments = self
            .scenario
            .graph
            .segments
            .iter()
            .map(|segment| SegmentSnapshot {
                id: segment.id,
                name: segment.name.clone(),
                from: segment.from,
                to: segment.to,
                lane_count: segment.lane_count,
                length_m: segment.length_m,
                speed_limit_mps: segment.speed_limit_mps,
                capacity: segment.capacity,
                occupancy: self
                    .segment_queues
                    .get(&segment.id)
                    .map_or(0, VecDeque::len),
            })
            .collect();

        let vehicles = self
            .vehicles
            .values()
            .map(|vehicle| VehicleSnapshot {
                id: vehicle.id,
                label: vehicle.label.clone(),
                kind: vehicle.kind,
                state: vehicle.state,
                route_index: vehicle.route_index,
                progress: vehicle.progress,
                speed_mps: vehicle.speed_mps,
                current_segment: vehicle.current_segment(),
                next_segment: vehicle.next_segment(),
                wait_ticks: vehicle.wait_ticks,
            })
            .collect();

        let signals = self
            .signals
            .values()
            .map(|signal| SignalSnapshot {
                id: signal.id,
                node: signal.node,
                phase: signal.phase,
                phase_tick: signal.phase_tick,
            })
            .collect();

        Snapshot {
            contract_version: crate::integration::CONTRACT_VERSION,
            tick: self.tick,
            running: self.running,
            scenario_name: self.scenario.name.clone(),
            nodes,
            segments,
            vehicles,
            signals,
            metrics: self.metrics.snapshot(),
            pending_spawns: self.pending_spawns.len(),
        }
    }

    pub fn delta(&self) -> SnapshotDelta {
        SnapshotDelta {
            tick: self.tick,
            vehicles: self.snapshot().vehicles,
            signals: self.snapshot().signals,
            metrics: self.metrics.snapshot(),
            events: self.events.clone(),
        }
    }

    fn advance_signals(&mut self) {
        for signal in self.signals.values_mut() {
            signal.phase_tick += 1;
            let limit = match signal.phase {
                SignalPhase::Green => signal.timing.green_ticks,
                SignalPhase::Yellow => signal.timing.yellow_ticks,
                SignalPhase::Red => signal.timing.red_ticks,
            }
            .max(1);

            if signal.phase_tick >= limit {
                signal.phase_tick = 0;
                signal.phase = match signal.phase {
                    SignalPhase::Red => SignalPhase::Green,
                    SignalPhase::Green => SignalPhase::Yellow,
                    SignalPhase::Yellow => SignalPhase::Red,
                };
                self.events.push(SimulationEvent {
                    tick: self.tick,
                    kind: EventKind::SignalChanged,
                    entity: Some(signal.id.0.to_string()),
                    message: format!("Semáforo {} cambió a {:?}", signal.id, signal.phase),
                });
            }
        }
    }

    fn spawn_due_vehicles(&mut self) {
        let mut still_waiting = VecDeque::new();
        while let Some(spawn) = self.pending_spawns.pop_front() {
            if spawn.release_tick > self.tick {
                still_waiting.push_back(spawn);
                continue;
            }

            let occupancy = self.segment_occupancy_map();
            let route = shortest_route(
                &self.scenario.graph,
                spawn.origin,
                spawn.destination,
                &occupancy,
            )
            .unwrap_or_default();

            if route.is_empty() {
                self.events.push(SimulationEvent {
                    tick: self.tick,
                    kind: EventKind::VehicleQueued,
                    entity: Some(spawn.id.0.to_string()),
                    message: format!("No se encontró ruta para {}", spawn.label),
                });
                still_waiting.push_back(spawn);
                continue;
            }

            if let Some(first_segment) = route.first().copied() {
                if self
                    .segment_queues
                    .get(&first_segment)
                    .map_or(0, VecDeque::len)
                    >= self
                        .scenario
                        .graph
                        .segment(first_segment)
                        .map_or(0, |segment| segment.capacity)
                {
                    still_waiting.push_back(spawn);
                    continue;
                }

                let vehicle = Vehicle::new(&spawn, VehicleRoute { segments: route }, self.tick);
                self.segment_queues
                    .entry(first_segment)
                    .or_default()
                    .push_back(vehicle.id);
                self.metrics.vehicles_spawned += 1;
                self.events.push(SimulationEvent {
                    tick: self.tick,
                    kind: EventKind::VehicleSpawned,
                    entity: Some(vehicle.id.0.to_string()),
                    message: format!("Vehículo {} ingresó a la red", vehicle.label),
                });
                self.vehicles.insert(vehicle.id, vehicle);
            }
        }
        self.pending_spawns = still_waiting;
    }

    fn move_vehicles(&mut self) {
        let segment_ids: Vec<SegmentId> = self
            .scenario
            .graph
            .segments
            .iter()
            .map(|segment| segment.id)
            .collect();

        for segment_id in segment_ids {
            let Some(segment) = self.scenario.graph.segment(segment_id).cloned() else {
                continue;
            };
            let ids = self
                .segment_queues
                .get(&segment_id)
                .cloned()
                .unwrap_or_default();
            let mut previous_front_progress: Option<f64> = None;
            let mut removals = Vec::new();

            for vehicle_id in ids {
                let Some(snapshot_vehicle) = self.vehicles.get(&vehicle_id).cloned() else {
                    continue;
                };
                if snapshot_vehicle.updated_tick == self.tick {
                    continue;
                }

                let delta = travel_progress(snapshot_vehicle.speed_mps, segment.length_m);
                let desired = normalize_progress(snapshot_vehicle.progress + delta);
                let cap = previous_front_progress
                    .map_or(1.0, |progress| (progress - DEFAULT_GAP_RATIO).max(0.0));
                let target = desired.min(cap);

                if target < 1.0 {
                    let moved = target > snapshot_vehicle.progress;
                    if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
                        vehicle.progress = target;
                        vehicle.state = if moved {
                            VehicleState::Moving
                        } else {
                            VehicleState::Queued
                        };
                        vehicle.updated_tick = self.tick;
                        vehicle.wait_ticks = if moved { 0 } else { vehicle.wait_ticks + 1 };
                    }
                    previous_front_progress = Some(target);
                    self.events.push(SimulationEvent {
                        tick: self.tick,
                        kind: EventKind::VehicleMoved,
                        entity: Some(vehicle_id.0.to_string()),
                        message: format!(
                            "{} avanzó sobre {}",
                            snapshot_vehicle.label, segment.name
                        ),
                    });
                    continue;
                }

                let Some(next_segment) = snapshot_vehicle.next_segment() else {
                    if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
                        vehicle.state = VehicleState::Arrived;
                        vehicle.progress = 1.0;
                        vehicle.arrival_tick = Some(self.tick);
                        vehicle.updated_tick = self.tick;
                    }
                    self.metrics.vehicles_completed += 1;
                    self.metrics.total_travel_ticks +=
                        self.tick.saturating_sub(snapshot_vehicle.spawn_tick);
                    self.metrics.total_wait_ticks += snapshot_vehicle.wait_ticks;
                    removals.push(vehicle_id);
                    previous_front_progress = Some(1.0);
                    self.events.push(SimulationEvent {
                        tick: self.tick,
                        kind: EventKind::VehicleArrived,
                        entity: Some(vehicle_id.0.to_string()),
                        message: format!("{} llegó a destino", snapshot_vehicle.label),
                    });
                    continue;
                };

                let can_enter = self.can_enter_next_segment(segment.to, next_segment);
                if !can_enter {
                    let red_signal = self.node_has_red_signal(segment.to);
                    if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
                        vehicle.state = if red_signal {
                            VehicleState::WaitingForSignal
                        } else {
                            VehicleState::Queued
                        };
                        vehicle.progress = 1.0;
                        vehicle.updated_tick = self.tick;
                        vehicle.wait_ticks += 1;
                    }
                    previous_front_progress = Some(1.0);
                    self.events.push(SimulationEvent {
                        tick: self.tick,
                        kind: EventKind::VehicleQueued,
                        entity: Some(vehicle_id.0.to_string()),
                        message: format!("{} espera en {}", snapshot_vehicle.label, segment.name),
                    });

                    let mut rerouted = false;
                    if snapshot_vehicle.wait_ticks + 1 >= 3
                        && snapshot_vehicle.reroute_count < self.reroute_limit
                    {
                        if let Some(destination_segment) =
                            snapshot_vehicle.route.segments.last().copied()
                        {
                            if let Some(destination_node) = self
                                .scenario
                                .graph
                                .segment(destination_segment)
                                .map(|segment| segment.to)
                            {
                                let occupancy = self.segment_occupancy_map();
                                if let Some(new_suffix) = shortest_route(
                                    &self.scenario.graph,
                                    segment.to,
                                    destination_node,
                                    &occupancy,
                                ) {
                                    if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
                                        vehicle.route.segments.truncate(vehicle.route_index + 1);
                                        vehicle.route.segments.extend(new_suffix);
                                        vehicle.reroute_count += 1;
                                        rerouted = true;
                                    }
                                }
                            }
                        }
                    }
                    if rerouted {
                        self.events.push(SimulationEvent {
                            tick: self.tick,
                            kind: EventKind::VehicleRerouted,
                            entity: Some(vehicle_id.0.to_string()),
                            message: format!("{} recalculó su ruta", snapshot_vehicle.label),
                        });
                    }
                    continue;
                }

                if let Some(queue) = self.segment_queues.get_mut(&segment_id) {
                    queue.retain(|id| *id != vehicle_id);
                }
                self.segment_queues
                    .entry(next_segment)
                    .or_default()
                    .push_back(vehicle_id);
                if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
                    vehicle.route_index += 1;
                    vehicle.progress = 0.0;
                    vehicle.state = VehicleState::Queued;
                    vehicle.updated_tick = self.tick;
                    vehicle.wait_ticks = 0;
                }
                previous_front_progress = Some(1.0);
                self.events.push(SimulationEvent {
                    tick: self.tick,
                    kind: EventKind::VehicleMoved,
                    entity: Some(vehicle_id.0.to_string()),
                    message: format!("{} pasó a {}", snapshot_vehicle.label, next_segment.0),
                });
            }

            if !removals.is_empty() {
                if let Some(queue) = self.segment_queues.get_mut(&segment_id) {
                    queue.retain(|id| !removals.contains(id));
                }
                for vehicle_id in removals {
                    self.vehicles.remove(&vehicle_id);
                }
            }
        }
    }

    fn can_enter_next_segment(&self, current_node: NodeId, next_segment: SegmentId) -> bool {
        let Some(segment) = self.scenario.graph.segment(next_segment) else {
            return false;
        };
        self.segment_queues
            .get(&next_segment)
            .map_or(0, VecDeque::len)
            < segment.capacity
            && !self.node_has_red_signal(current_node)
    }

    fn node_has_red_signal(&self, node_id: NodeId) -> bool {
        self.signals
            .values()
            .find(|signal| signal.node == node_id)
            .map(|signal| matches!(signal.phase, SignalPhase::Red | SignalPhase::Yellow))
            .unwrap_or(false)
    }

    fn segment_occupancy_map(&self) -> BTreeMap<SegmentId, usize> {
        self.segment_queues
            .iter()
            .map(|(segment_id, queue)| (*segment_id, queue.len()))
            .collect()
    }
}
