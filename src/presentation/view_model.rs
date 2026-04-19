use crate::integration::snapshots::Snapshot;

#[derive(Clone, Debug)]
pub struct ViewModel {
    pub snapshot: Snapshot,
}

impl ViewModel {
    pub fn from_snapshot(snapshot: Snapshot) -> Self {
        Self { snapshot }
    }
}
