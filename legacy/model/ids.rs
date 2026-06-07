use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            Eq,
            Hash,
            Ord,
            PartialEq,
            PartialOrd,
            Serialize,
            Deserialize,
        )]
        pub struct $name(pub usize);

        impl From<usize> for $name {
            fn from(value: usize) -> Self {
                Self(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

define_id!(NodeId);
define_id!(SegmentId);
define_id!(LaneId);
define_id!(VehicleId);
define_id!(SignalId);
