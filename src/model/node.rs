use super::{NodeId, RoadId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum NodeKind {
    Entry,
    Exit,
    Intersection,
    TrafficLight,
    Stop,
    Roundabout,
    Merge,
    Split,
}

#[derive(Clone, Debug)]
pub struct SignalPhase {
    pub name: String,
    pub allowed_roads: Vec<RoadId>,
    pub duration: u32,
}

impl SignalPhase {
    pub fn new(name: impl Into<String>, duration: u32, allowed_roads: Vec<RoadId>) -> Self {
        assert!(duration > 0, "signal phases must last at least one tick");
        Self {
            name: name.into(),
            allowed_roads,
            duration,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SignalPlan {
    pub phases: Vec<SignalPhase>,
}

impl SignalPlan {
    pub fn new(phases: Vec<SignalPhase>) -> Self {
        assert!(!phases.is_empty(), "signal plans need at least one phase");
        Self { phases }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub name: String,
    pub kind: NodeKind,
    pub signal_plan: Option<SignalPlan>,
    pub inbound_roads: Vec<RoadId>,
    pub outbound_roads: Vec<RoadId>,
}

impl Node {
    pub fn new(id: NodeId, name: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            signal_plan: None,
            inbound_roads: Vec::new(),
            outbound_roads: Vec::new(),
        }
    }

    pub fn with_signal_plan(mut self, signal_plan: SignalPlan) -> Self {
        self.signal_plan = Some(signal_plan);
        self
    }
}