pub const DEFAULT_GAP_RATIO: f64 = 0.05;

pub fn normalize_progress(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}
