use crate::integration::snapshots::Snapshot;

#[derive(Clone, Debug)]
pub struct ViewModel {
    pub previous_snapshot: Snapshot,
    pub snapshot: Snapshot,
    pub interpolation_alpha: f32,
}

impl ViewModel {
    pub fn from_snapshots(
        previous_snapshot: Snapshot,
        snapshot: Snapshot,
        interpolation_alpha: f32,
    ) -> Self {
        Self {
            previous_snapshot,
            snapshot,
            interpolation_alpha,
        }
    }
}
