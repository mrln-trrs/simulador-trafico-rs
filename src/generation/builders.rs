use crate::model::graph::{Graph, Point2};
use crate::model::scenario::Scenario;
use crate::model::signal::TrafficSignal;
use crate::model::state::{NodeKind, VehicleKind};
use crate::model::{VehicleId, VehicleSpawn};

pub struct ScenarioBuilder {
    name: String,
    seed: u64,
    duration_ticks: u64,
    graph: Graph,
    spawns: Vec<VehicleSpawn>,
    signals: Vec<TrafficSignal>,
}

impl ScenarioBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            seed: 1,
            duration_ticks: 120,
            graph: Graph::default(),
            spawns: Vec::new(),
            signals: Vec::new(),
        }
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn duration_ticks(mut self, duration_ticks: u64) -> Self {
        self.duration_ticks = duration_ticks;
        self
    }

    pub fn node(mut self, name: impl Into<String>, kind: NodeKind, x: f64, y: f64) -> Self {
        self.graph.add_node(name, kind, Point2 { x, y });
        self
    }

    pub fn segment(
        mut self,
        name: impl Into<String>,
        from: usize,
        to: usize,
        lane_count: usize,
        length_m: f64,
        speed_limit_mps: f64,
        capacity: usize,
    ) -> Self {
        self.graph.add_segment(
            name,
            from.into(),
            to.into(),
            lane_count,
            length_m,
            speed_limit_mps,
            capacity,
        );
        self
    }

    pub fn spawn(
        mut self,
        release_tick: u64,
        origin: usize,
        destination: usize,
        kind: VehicleKind,
        speed_mps: f64,
        label: impl Into<String>,
    ) -> Self {
        let id = VehicleId(self.spawns.len());
        self.spawns.push(VehicleSpawn {
            id,
            release_tick,
            origin: origin.into(),
            destination: destination.into(),
            kind,
            speed_mps,
            label: label.into(),
        });
        self
    }

    pub fn signal(mut self, signal: TrafficSignal) -> Self {
        self.signals.push(signal);
        self
    }

    pub fn build(self) -> Scenario {
        Scenario {
            name: self.name,
            seed: self.seed,
            duration_ticks: self.duration_ticks,
            graph: self.graph,
            spawns: self.spawns,
            signals: self.signals,
        }
    }
}
