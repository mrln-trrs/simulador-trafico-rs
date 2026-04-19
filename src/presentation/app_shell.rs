use crate::generation::fixtures::demo_scenario;
use crate::integration::commands::Command;
use crate::integration::snapshots::Snapshot;
use crate::presentation::canvas::{draw_snapshot, CanvasState};
use crate::presentation::view_model::ViewModel;
use crate::simulation::SimulationEngine;
use eframe::egui;

pub struct SimulatorApp {
    engine: SimulationEngine,
    canvas: CanvasState,
    last_snapshot: Snapshot,
    pending_steps: u32,
}

impl SimulatorApp {
    pub fn new(scenario: crate::model::Scenario) -> Self {
        let engine =
            SimulationEngine::new(scenario).expect("el escenario de demostración debe ser válido");
        let last_snapshot = engine.snapshot();
        Self {
            engine,
            canvas: CanvasState::default(),
            last_snapshot,
            pending_steps: 0,
        }
    }

    fn apply_command(&mut self, command: Command) {
        match command {
            Command::Play => self.engine.play(),
            Command::Pause => self.engine.pause(),
            Command::Step(amount) => self.pending_steps = self.pending_steps.saturating_add(amount),
            Command::Reset => self.engine.reset(),
            Command::LoadDemo => {
                self.engine = SimulationEngine::new(demo_scenario())
                    .expect("el escenario de demostración debe ser válido");
            }
        }
        self.last_snapshot = self.engine.snapshot();
    }
}

impl eframe::App for SimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.engine.is_running() {
            self.engine.advance_tick();
        } else if self.pending_steps > 0 {
            self.engine.advance_tick();
            self.pending_steps -= 1;
        }

        self.last_snapshot = self.engine.snapshot();
        let view_model = ViewModel::from_snapshot(self.last_snapshot.clone());

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Play").clicked() {
                    self.apply_command(Command::Play);
                }
                if ui.button("Pause").clicked() {
                    self.apply_command(Command::Pause);
                }
                if ui.button("Step").clicked() {
                    self.apply_command(Command::Step(1));
                }
                if ui.button("Reset").clicked() {
                    self.apply_command(Command::Reset);
                }
                ui.label(format!("Tick: {}", self.last_snapshot.tick));
                ui.label(format!("Vehículos: {}", self.last_snapshot.vehicles.len()));
            });
        });

        egui::SidePanel::right("inspector")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Estado");
                ui.label(format!("Escenario: {}", self.last_snapshot.scenario_name));
                ui.label(format!(
                    "Versión contrato: {}",
                    self.last_snapshot.contract_version
                ));
                ui.label(format!(
                    "Completados: {}",
                    self.last_snapshot.metrics.vehicles_completed
                ));
                ui.label(format!(
                    "Promedio viaje: {:.2}",
                    self.last_snapshot.metrics.average_travel_ticks()
                ));
                ui.separator();
                ui.label(format!(
                    "Pendientes de salida: {}",
                    self.last_snapshot.pending_spawns
                ));
                ui.label(format!("Nodos: {}", self.last_snapshot.nodes.len()));
                ui.label(format!("Tramos: {}", self.last_snapshot.segments.len()));
                ui.separator();
                ui.collapsing("Vehículos activos", |ui| {
                    for vehicle in &view_model.snapshot.vehicles {
                        ui.label(format!(
                            "{} - {:?} - {:.2} - {:?}",
                            vehicle.label, vehicle.state, vehicle.progress, vehicle.current_segment
                        ));
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            draw_snapshot(ui, &view_model.snapshot, &mut self.canvas);
        });
    }
}
