use super::{events::SimulationEvent, Simulation};

impl Simulation {
    pub fn run_until_idle(&mut self, max_ticks: u32) -> Vec<SimulationEvent> {
        let mut events = Vec::new();
        for _ in 0..max_ticks {
            let step_events = self.step();
            events.extend(step_events);
            if self.is_idle() {
                break;
            }
        }
        events
    }

    pub fn step(&mut self) -> Vec<SimulationEvent> {
        let mut events = Vec::new();

        self.release_scheduled_spawns(&mut events);
        self.increment_time_counters();
        self.advance_signals(&mut events);
        self.advance_road_fronts(&mut events);
        self.release_node_queues(&mut events);

        self.current_tick += 1;
        events
    }
}