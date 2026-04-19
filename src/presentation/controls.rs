use egui::*;
use crate::presentation::theme::FluentTheme;

/// Estados de la simulación
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulationState {
    Paused,
    Running,
    Completed,
}

/// Controles de reproducción
#[derive(Clone, Debug)]
pub struct PlaybackControls {
    pub state: SimulationState,
    pub speed: f32,      // Factor de velocidad (0.25x a 4.0x)
    pub ticks_per_second: f32,  // Pasos por segundo (10 a 240)
    pub current_tick: u64,
    pub total_ticks: u64,
}

impl Default for PlaybackControls {
    fn default() -> Self {
        Self {
            state: SimulationState::Paused,
            speed: 1.0,
            ticks_per_second: 60.0,
            current_tick: 0,
            total_ticks: 10000,
        }
    }
}

impl PlaybackControls {
    pub fn effective_ticks_per_second(&self) -> f32 {
        self.ticks_per_second * self.speed
    }
    
    pub fn play(&mut self) {
        self.state = SimulationState::Running;
    }
    
    pub fn pause(&mut self) {
        self.state = SimulationState::Paused;
    }
    
    pub fn advance_tick(&mut self) {
        if self.current_tick < self.total_ticks {
            self.current_tick += 1;

            if self.current_tick >= self.total_ticks {
                self.state = SimulationState::Completed;
            }
        } else {
            self.state = SimulationState::Completed;
        }
    }
    
    pub fn reset(&mut self) {
        self.current_tick = 0;
        self.state = SimulationState::Paused;
    }
}

/// Panel de herramientas lateral
pub struct ToolPanel {
    pub selected_tool: SelectedTool,
    pub zoom_level: f32,
    pub grid_snap: bool,
    pub show_metrics: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectedTool {
    None,
    CreateNode,
    CreateEdge,
    Select,
    Move,
    Delete,
    Inspect,
}

impl Default for ToolPanel {
    fn default() -> Self {
        Self {
            selected_tool: SelectedTool::Select,
            zoom_level: 1.0,
            grid_snap: true,
            show_metrics: false,
        }
    }
}

/// Dibuja los controles de reproducción
pub fn draw_playback_controls(
    ui: &mut Ui,
    controls: &mut PlaybackControls,
    theme: &FluentTheme,
) {
    ui.horizontal(|ui| {
        // Botón Play/Pause
        let play_pause_text = if controls.state == SimulationState::Running {
            "⏸"
        } else {
            "▶"
        };
        
        if ui.button(
            RichText::new(play_pause_text)
                .size(16.0)
                .color(theme.text_primary)
        ).clicked() {
            if controls.state == SimulationState::Running {
                controls.pause();
            } else {
                controls.play();
            }
        }
        
        // Botón siguiente tick
        if ui.button(
            RichText::new("⏭")
                .size(16.0)
                .color(theme.text_primary)
        ).clicked() {
            controls.advance_tick();
        }
        
        // Botón reset
        if ui.button(
            RichText::new("⏹")
                .size(16.0)
                .color(theme.text_primary)
        ).clicked() {
            controls.reset();
        }
        
        ui.separator();
        
        // Indicador de ticks
        ui.label(
            RichText::new(format!("{} / {}", controls.current_tick, controls.total_ticks))
                .size(12.0)
                .color(theme.text_secondary)
        );
    });
    
    // Control de velocidad
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Speed:")
                .size(12.0)
                .color(theme.text_secondary)
        );
        
        ui.add(
            Slider::new(&mut controls.speed, 0.25..=4.0)
                .text("x")
                .step_by(0.25)
                .show_value(true)
        );
    });
}

/// Dibuja el panel de herramientas
pub fn draw_tool_panel(
    ui: &mut Ui,
    panel: &mut ToolPanel,
    theme: &FluentTheme,
) {
    ui.vertical(|ui| {
        ui.label(
            RichText::new("Herramientas")
                .size(14.0)
                .color(theme.text_primary)
                .strong()
        );
        ui.separator();
        
        // Herramientas de creación y edición
        let tools = [
            (SelectedTool::Select, "🔍", "Seleccionar"),
            (SelectedTool::CreateNode, "●", "Crear nodo"),
            (SelectedTool::CreateEdge, "⟶", "Crear tramo"),
            (SelectedTool::Move, "✤", "Mover"),
            (SelectedTool::Delete, "⌫", "Eliminar"),
            (SelectedTool::Inspect, "ℹ", "Inspeccionar"),
        ];
        
        for (tool, icon, tooltip) in &tools {
            let selected = panel.selected_tool == *tool;
            
            let button_response = ui.add(
                Button::new(
                    RichText::new(icon.to_string())
                        .size(16.0)
                        .color(if selected { theme.primary } else { theme.text_secondary })
                )
                .fill(if selected { theme.surface } else { theme.background })
                .stroke(if selected {
                    Stroke::new(2.0, theme.primary)
                } else {
                    Stroke::new(1.0, theme.outline_variant)
                })
                .min_size(Vec2::new(40.0, 40.0))
            );
            
            if button_response.clicked() {
                panel.selected_tool = *tool;
            }
            
            button_response.on_hover_text(*tooltip);
        }
        
        ui.separator();
        
        // Opciones de vista
        ui.label(
            RichText::new("Vista")
                .size(12.0)
                .color(theme.text_secondary)
        );
        
        ui.add(
            Slider::new(&mut panel.zoom_level, 0.25..=4.0)
                .text("Zoom")
                .step_by(0.25)
                .show_value(true)
        );
        
        ui.checkbox(&mut panel.grid_snap, "Snap a grid");
        ui.checkbox(&mut panel.show_metrics, "Mostrar métricas");
    });
}

/// Panel de propiedades de elemento seleccionado
pub fn draw_properties_panel(
    ui: &mut Ui,
    selected_id: Option<&str>,
    theme: &FluentTheme,
) {
    Frame::none()
        .fill(theme.surface)
        .stroke(Stroke::new(1.0, theme.outline_variant))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            if let Some(id) = selected_id {
                ui.label(
                    RichText::new("Propiedades")
                        .size(14.0)
                        .color(theme.text_primary)
                        .strong()
                );
                ui.separator();
                
                ui.label(
                    RichText::new(format!("ID: {}", id))
                        .size(11.0)
                        .color(theme.text_secondary)
                );
                
                // Placeholder para edición dinámica
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Posición")
                        .size(12.0)
                        .color(theme.text_primary)
                        .strong()
                );
                
                ui.horizontal(|ui| {
                    ui.label("X:");
                    let mut x = 0.0;
                    ui.text_edit_singleline(&mut x.to_string());
                });
                
                ui.horizontal(|ui| {
                    ui.label("Y:");
                    let mut y = 0.0;
                    ui.text_edit_singleline(&mut y.to_string());
                });
            } else {
                ui.label(
                    RichText::new("Ningún elemento seleccionado")
                        .size(12.0)
                        .color(theme.text_tertiary)
                        .italics()
                );
            }
        });
}

/// Panel de estado y métricas
pub fn draw_metrics_panel(
    ui: &mut Ui,
    theme: &FluentTheme,
) {
    Frame::none()
        .fill(theme.surface_variant)
        .stroke(Stroke::new(0.5, theme.outline_variant))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new("Métricas")
                    .size(12.0)
                    .color(theme.text_primary)
                    .strong()
            );
            
            ui.separator();
            
            // Indicadores de estado
            ui.horizontal(|ui| {
                ui.colored_label(theme.success, "●");
                ui.label(
                    RichText::new("Vehículos: 42")
                        .size(11.0)
                        .color(theme.text_secondary)
                );
            });
            
            ui.horizontal(|ui| {
                ui.colored_label(theme.warning, "●");
                ui.label(
                    RichText::new("Congestión: Media")
                        .size(11.0)
                        .color(theme.text_secondary)
                );
            });
            
            ui.horizontal(|ui| {
                ui.colored_label(theme.info, "●");
                ui.label(
                    RichText::new("FPS: 60")
                        .size(11.0)
                        .color(theme.text_secondary)
                );
            });
        });
}
