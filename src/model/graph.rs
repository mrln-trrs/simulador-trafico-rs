use crate::model::ids::{NodeId, SegmentId};
use crate::model::state::NodeKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub name: String,
    pub kind: NodeKind,
    pub position: Point2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoadSegment {
    pub id: SegmentId,
    pub name: String,
    pub from: NodeId,
    pub to: NodeId,
    pub lane_count: usize,
    pub length_m: f64,
    pub speed_limit_mps: f64,
    pub capacity: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub segments: Vec<RoadSegment>,
    pub outgoing: BTreeMap<NodeId, Vec<SegmentId>>,
}

impl Graph {
    pub fn add_node(
        &mut self,
        name: impl Into<String>,
        kind: NodeKind,
        position: Point2,
    ) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            id,
            name: name.into(),
            kind,
            position,
        });
        id
    }

    pub fn add_segment(
        &mut self,
        name: impl Into<String>,
        from: NodeId,
        to: NodeId,
        lane_count: usize,
        length_m: f64,
        speed_limit_mps: f64,
        capacity: usize,
    ) -> SegmentId {
        let id = SegmentId(self.segments.len());
        self.segments.push(RoadSegment {
            id,
            name: name.into(),
            from,
            to,
            lane_count,
            length_m,
            speed_limit_mps,
            capacity,
        });
        self.outgoing.entry(from).or_default().push(id);
        id
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }

    pub fn segment(&self, id: SegmentId) -> Option<&RoadSegment> {
        self.segments.get(id.0)
    }

    pub fn outgoing_segments(&self, id: NodeId) -> &[SegmentId] {
        self.outgoing.get(&id).map_or(&[], Vec::as_slice)
    }
}
