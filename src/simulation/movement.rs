pub fn travel_progress(speed_mps: f64, length_m: f64) -> f64 {
    if length_m <= 0.0 {
        1.0
    } else {
        (speed_mps / length_m).clamp(0.0, 1.0)
    }
}
