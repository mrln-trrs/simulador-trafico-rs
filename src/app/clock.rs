use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct FixedStepClock {
    last_instant: Instant,
    accumulator: Duration,
}

impl FixedStepClock {
    pub fn new(now: Instant) -> Self {
        Self {
            last_instant: now,
            accumulator: Duration::ZERO,
        }
    }


    pub fn reset(&mut self, now: Instant) {
        self.last_instant = now;
        self.accumulator = Duration::ZERO;
    }

    pub fn drain_steps(
        &mut self,
        now: Instant,
        running: bool,
        effective_ticks_per_second: f32,
    ) -> u32 {
        if !running {
            self.last_instant = now;
            return 0;
        }

        let Some(step_duration) = Self::step_duration(effective_ticks_per_second) else {
            self.last_instant = now;
            return 0;
        };

        let elapsed = now.saturating_duration_since(self.last_instant);
        self.last_instant = now;
        self.accumulator += elapsed;

        let mut steps = 0;
        while self.accumulator >= step_duration {
            self.accumulator -= step_duration;
            steps += 1;
        }

        steps
    }

    pub fn interpolation_alpha(
        &self,
        running: bool,
        effective_ticks_per_second: f32,
    ) -> f32 {
        if !running {
            return 0.0;
        }

        let Some(step_duration) = Self::step_duration(effective_ticks_per_second) else {
            return 0.0;
        };

        if step_duration.is_zero() {
            return 0.0;
        }

        (self.accumulator.as_secs_f64() / step_duration.as_secs_f64())
            .clamp(0.0, 1.0) as f32
    }

    fn step_duration(effective_ticks_per_second: f32) -> Option<Duration> {
        if !effective_ticks_per_second.is_finite() || effective_ticks_per_second <= 0.0 {
            return None;
        }

        Some(Duration::from_secs_f64(
            1.0 / effective_ticks_per_second as f64,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::FixedStepClock;
    use std::time::{Duration, Instant};

    #[test]
    fn drains_steps_from_real_time() {
        let start = Instant::now();
        let mut clock = FixedStepClock::new(start);

        let half_second_later = start + Duration::from_millis(500);
        assert_eq!(clock.drain_steps(half_second_later, true, 2.0), 1);

        let one_second_later = half_second_later + Duration::from_millis(500);
        assert_eq!(clock.drain_steps(one_second_later, true, 2.0), 1);
    }

    #[test]
    fn accumulates_leftover_time_between_frames() {
        let start = Instant::now();
        let mut clock = FixedStepClock::new(start);

        let later = start + Duration::from_millis(1100);
        assert_eq!(clock.drain_steps(later, true, 2.0), 2);

        let next = later + Duration::from_millis(400);
        assert_eq!(clock.drain_steps(next, true, 2.0), 1);
    }

    #[test]
    fn paused_clock_does_not_accumulate_time() {
        let start = Instant::now();
        let mut clock = FixedStepClock::new(start);

        let paused_time = start + Duration::from_secs(1);
        assert_eq!(clock.drain_steps(paused_time, false, 2.0), 0);

        let resumed_time = paused_time + Duration::from_millis(500);
        assert_eq!(clock.drain_steps(resumed_time, true, 2.0), 1);
    }

    #[test]
    fn exposes_partial_step_progress_for_rendering() {
        let start = Instant::now();
        let mut clock = FixedStepClock::new(start);

        let midway = start + Duration::from_millis(250);
        assert_eq!(clock.drain_steps(midway, true, 2.0), 0);

        let alpha = clock.interpolation_alpha(true, 2.0);
        assert!((alpha - 0.5).abs() < 1e-6);
    }
}