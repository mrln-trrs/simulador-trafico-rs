use crate::app::clock::FixedStepClock;
use crate::generation::fixtures::demo_scenario;
use crate::integration::commands::Command;
use crate::integration::snapshots::Snapshot;
use crate::presentation::canvas::{draw_snapshot, CanvasState};
use crate::presentation::view_model::ViewModel;
use crate::presentation::theme::FluentTheme;
use crate::presentation::controls::{PlaybackControls, ToolPanel, SimulationState};
use crate::simulation::SimulationEngine;
use egui::*;
use std::time::Instant;

pub struct SimulatorApp {
    engine: SimulationEngine,
    canvas: CanvasState,
    previous_snapshot: Snapshot,
    last_snapshot: Snapshot,
    theme: FluentTheme,
    playback: PlaybackControls,
    tools: ToolPanel,
    clock: FixedStepClock,
    ui_scale: f32,
}

impl SimulatorApp {
    pub fn new(scenario: crate::model::Scenario) -> Self {
        let now = Instant::now();
        let engine =
            SimulationEngine::new(scenario).expect("el escenario de demostración debe ser válido");
        let last_snapshot = engine.snapshot();
        
        Self {
            engine,
            canvas: CanvasState::default(),
            previous_snapshot: last_snapshot.clone(),
            last_snapshot,
            theme: FluentTheme::dark(),
            playback: PlaybackControls::default(),
            tools: ToolPanel::default(),
            clock: FixedStepClock::new(now),
            ui_scale: 1.0,
        }
    }

    fn play_simulation(&mut self) {
        self.playback.play();
        self.engine.play();
    }

    fn pause_simulation(&mut self) {
        self.playback.pause();
        self.engine.pause();
    }

    fn step_simulation(&mut self, steps: u32) {
        for _ in 0..steps {
            if self.playback.state == SimulationState::Completed {
                self.engine.pause();
                break;
            }

            self.previous_snapshot = self.last_snapshot.clone();
            self.playback.advance_tick();
            self.engine.advance_tick();
            self.last_snapshot = self.engine.snapshot();

            if self.playback.state == SimulationState::Completed {
                self.engine.pause();
                break;
            }
        }
    }

    fn reset_simulation(&mut self) {
        self.playback.reset();
        self.engine.reset();
        self.previous_snapshot = self.engine.snapshot();
        self.last_snapshot = self.previous_snapshot.clone();
        self.clock.reset(Instant::now());
    }

    fn apply_command(&mut self, command: Command) {
        match command {
            Command::Play => self.play_simulation(),
            Command::Pause => self.pause_simulation(),
            Command::Step(amount) => self.step_simulation(amount),
            Command::Reset => self.reset_simulation(),
            Command::LoadDemo => {
                self.engine = SimulationEngine::new(demo_scenario())
                    .expect("el escenario de demostración debe ser válido");
                self.playback.reset();
                self.previous_snapshot = self.engine.snapshot();
                self.last_snapshot = self.previous_snapshot.clone();
                self.clock.reset(Instant::now());
            }
        }
        self.last_snapshot = self.engine.snapshot();
    }
    
    fn update_simulation(&mut self, now: Instant) {
        let steps_due = self.clock.drain_steps(
            now,
            self.engine.is_running(),
            self.playback.effective_ticks_per_second(),
        );

        if steps_due > 0 {
            self.step_simulation(steps_due);
        }
    }
}

impl eframe::App for SimulatorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Aplicar escala global de UI
        {
            let mut style = (*ctx.style()).clone();
            style.text_styles.insert(
                TextStyle::Body,
                FontId::new(12.0 * self.ui_scale, FontFamily::Proportional),
            );
            style.text_styles.insert(
                TextStyle::Button,
                FontId::new(12.0 * self.ui_scale, FontFamily::Proportional),
            );
            style.text_styles.insert(
                TextStyle::Heading,
                FontId::new(18.0 * self.ui_scale, FontFamily::Proportional),
            );
            ctx.set_style(style);
        }
        
        // Actualiza la simulación
        self.update_simulation(Instant::now());

        let view_model = ViewModel::from_snapshots(
            self.previous_snapshot.clone(),
            self.last_snapshot.clone(),
            self.clock.interpolation_alpha(
                self.engine.is_running(),
                self.playback.effective_ticks_per_second(),
            ),
        );

        // ===== BARRA SUPERIOR =====
        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("▶ Simulador de Tráfico")
                            .size(16.0 * self.ui_scale)
                            .color(self.theme.primary)
                            .strong()
                    );
                    
                    ui.separator();
                    
                    // Controles básicos
                    if ui.button("▶ Play").clicked() {
                        self.apply_command(Command::Play);
                    }
                    
                    if ui.button("⏸ Pause").clicked() {
                        self.apply_command(Command::Pause);
                    }
                    
                    if ui.button("⏭ Siguiente").clicked() {
                        self.apply_command(Command::Step(1));
                    }
                    
                    if ui.button("⏹ Reset").clicked() {
                        self.apply_command(Command::Reset);
                    }
                    
                    ui.separator();
                    
                    ui.label(
                        RichText::new(format!("Tick: {}", self.playback.current_tick))
                            .color(self.theme.text_secondary)
                            .size(12.0 * self.ui_scale)
                    );
                    
                    // Espaciador
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Control de zoom UI
                        ui.label(
                            RichText::new("UI:")
                                .size(11.0 * self.ui_scale)
                                .color(self.theme.text_secondary)
                        );
                        
                        // Botón Reset
                        if ui.button(
                            RichText::new("↺")
                                .size(12.0 * self.ui_scale)
                        ).clicked() {
                            self.ui_scale = 1.0;
                        }
                        
                        // Zoom -
                        if ui.button(
                            RichText::new("−")
                                .size(14.0 * self.ui_scale)
                        ).clicked() {
                            self.ui_scale = (self.ui_scale - 0.1).max(0.5);
                        }
                        
                        // Mostrar escala actual
                        ui.label(
                            RichText::new(format!("{:.0}%", self.ui_scale * 100.0))
                                .size(10.0 * self.ui_scale)
                                .color(self.theme.text_secondary)
                        );
                        
                        // Zoom +
                        if ui.button(
                            RichText::new("+")
                                .size(14.0 * self.ui_scale)
                        ).clicked() {
                            self.ui_scale = (self.ui_scale + 0.1).min(2.0);
                        }
                    });
                });
            });
        });

        // ===== PANEL IZQUIERDO (Herramientas) =====
        SidePanel::left("tools")
            .resizable(true)
            .default_width(120.0)
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Herramientas").size(12.0).strong());
                        ui.separator();
                        
                        // Herramientas basicas
                        for (_idx, (icon, name)) in [
                            ("🔍", "Seleccionar"),
                            ("●", "Nodo"),
                            ("⟶", "Tramo"),
                            ("✤", "Mover"),
                            ("⌫", "Eliminar"),
                        ].iter().enumerate() {
                            if ui.button(format!("{} {}", icon, name)).clicked() {
                                // Acción de herramienta
                            }
                        }
                        
                        ui.separator();
                        
                        ui.label(RichText::new("Opciones").size(11.0).strong());
                        ui.checkbox(&mut self.tools.grid_snap, "Snap a grid");
                        ui.checkbox(&mut self.tools.show_metrics, "Métricas");
                        
                        ui.separator();
                        
                        ui.label(RichText::new("Velocidad").size(11.0).strong());
                        ui.add(
                            Slider::new(&mut self.playback.speed, 0.25..=4.0)
                                .step_by(0.25)
                        );
                        
                        ui.separator();
                        
                        ui.label(RichText::new("Ticks/Segundo").size(11.0).strong());
                        ui.add(
                            Slider::new(&mut self.playback.ticks_per_second, 10.0..=240.0)
                                .step_by(5.0)
                        );
                        ui.label(
                            RichText::new(format!("{:.0} TPS", self.playback.ticks_per_second))
                                .size(10.0)
                                .color(self.theme.text_secondary)
                        );
                    });
            });

        // ===== PANEL DERECHO (Inspector) =====
        SidePanel::right("inspector")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Estado").size(12.0).strong());
                        ui.separator();
                        
                        // Estado general
                        let state_text = match self.playback.state {
                            SimulationState::Running => "En ejecución",
                            SimulationState::Paused => "Pausada",
                            SimulationState::Completed => "Completada",
                        };
                        
                        let state_color = match self.playback.state {
                            SimulationState::Running => self.theme.success,
                            SimulationState::Paused => self.theme.warning,
                            SimulationState::Completed => self.theme.info,
                        };
                        
                        ui.horizontal(|ui| {
                            ui.colored_label(state_color, "●");
                            ui.label(RichText::new(state_text).size(11.0).color(self.theme.text_secondary));
                        });
                        
                        ui.separator();
                        
                        ui.label(
                            RichText::new(format!("Tick: {}", self.last_snapshot.tick))
                                .size(10.0)
                                .color(self.theme.text_secondary)
                        );
                        
                        ui.label(
                            RichText::new(format!("Vehículos: {}", self.last_snapshot.vehicles.len()))
                                .size(10.0)
                                .color(self.theme.text_secondary)
                        );
                        
                        ui.label(
                            RichText::new(format!("Completados: {}", 
                                self.last_snapshot.metrics.vehicles_completed))
                                .size(10.0)
                                .color(self.theme.text_secondary)
                        );
                        
                        ui.label(
                            RichText::new(format!("Promedio: {:.1}", 
                                self.last_snapshot.metrics.average_travel_ticks()))
                                .size(10.0)
                                .color(self.theme.text_secondary)
                        );
                        
                        ui.label(
                            RichText::new(format!("Nodos: {}", 
                                self.last_snapshot.nodes.len()))
                                .size(10.0)
                                .color(self.theme.text_secondary)
                        );
                        
                        ui.label(
                            RichText::new(format!("Tramos: {}", 
                                self.last_snapshot.segments.len()))
                                .size(10.0)
                                .color(self.theme.text_secondary)
                        );
                        
                        ui.separator();
                        
                        CollapsingHeader::new(
                            RichText::new("Vehículos")
                                .size(11.0)
                                .color(self.theme.text_primary)
                        )
                        .default_open(false)
                        .show(ui, |ui| {
                            for vehicle in &view_model.snapshot.vehicles {
                                Frame::none()
                                    .inner_margin(Margin::same(4.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(&vehicle.label)
                                                .size(9.0)
                                                .color(self.theme.primary)
                                        );
                                        ui.label(
                                            RichText::new(format!("Progreso: {:.1}%", 
                                                vehicle.progress * 100.0))
                                                .size(8.0)
                                                .color(self.theme.text_tertiary)
                                        );
                                    });
                            }
                        });
                    });
            });

        // ===== CANVAS CENTRAL =====
        CentralPanel::default().show(ctx, |ui| {
            draw_snapshot(ui, &view_model, &mut self.canvas, &self.theme);
        });
        
        // Solicita repaint para animación fluida
        ctx.request_repaint();
    }
}

