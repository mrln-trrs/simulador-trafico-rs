use super::{events::SimulationEvent, Simulation};

impl Simulation {
    pub(super) fn advance_signals(&mut self, events: &mut Vec<SimulationEvent>) {
        let signal_nodes = self.signal_nodes.clone();

        for node_id in signal_nodes {
            let Some(node) = self.network.node(node_id) else {
                continue;
            };
            let Some(plan) = node.signal_plan.as_ref() else {
                continue;
            };
            let Some(state) = self.signals.get(&node_id) else {
                continue;
            };

            let phase_count = plan.phases.len();
            let phase_duration = plan.phases[state.phase_index].duration;

            let state = self
                .signals
                .get_mut(&node_id)
                .expect("signal state should exist for signal nodes");
            state.time_in_phase += 1;

            if state.time_in_phase >= phase_duration {
                state.time_in_phase = 0;
                state.phase_index = (state.phase_index + 1) % phase_count;
                let next_phase_name = plan.phases[state.phase_index].name.clone();
                events.push(SimulationEvent::SignalChanged {
                    node_id,
                    phase_name: next_phase_name,
                });
            }
        }
    }
}