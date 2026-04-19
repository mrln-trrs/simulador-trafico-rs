pub mod network;
pub mod node;
pub mod road;
pub mod routing;
pub mod vehicle;

pub type NodeId = usize;
pub type RoadId = usize;
pub type VehicleId = usize;

pub use network::Network;
pub use node::{Node, NodeKind, SignalPhase, SignalPlan};
pub use road::RoadSegment;
pub use vehicle::{Vehicle, VehicleSpawn, VehicleStatus, VehicleType};