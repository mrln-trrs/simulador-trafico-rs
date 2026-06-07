use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Play,
    Pause,
    Step(u32),
    Reset,
    LoadDemo,
}
