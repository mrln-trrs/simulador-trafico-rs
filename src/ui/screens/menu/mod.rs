use egui::{Context, DragValue, Layout, Align, TopBottomPanel, CentralPanel};

pub struct MenuApp {
    pub temp_ui_zoom: f32,
    pub temp_text_scale: f32,
    pub initialized: bool,
}

impl MenuApp {
    pub fn new() -> Self {
        Self {
            temp_ui_zoom: 1.0,
            temp_text_scale: 1.0,
            initialized: false,
        }
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        simulator_app: &mut crate::ui::screens::simulator::SimuladorApp,
        window: &egui_multiwin::winit::window::Window,
        rr: &mut crate::app::runtime::tracked_window::RedrawResponse,
    ) {
        if !self.initialized {
            self.temp_ui_zoom = simulator_app.ui_zoom;
            self.temp_text_scale = simulator_app.text_scale;
            self.initialized = true;
        }

        let size = window.inner_size();
        if let Ok(pos) = window.outer_position() {
            simulator_app.settings_window_size = Some([size.width, size.height]);
            simulator_app.settings_window_pos = Some([pos.x, pos.y]);
        }

        TopBottomPanel::bottom("settings_buttons").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Aplicar").clicked() {
                        simulator_app.ui_zoom = self.temp_ui_zoom;
                        simulator_app.text_scale = self.temp_text_scale;
                        if let Some(ref sim_ctx) = simulator_app.egui_ctx {
                            sim_ctx.request_repaint();
                        }
                        window.request_redraw();
                    }
                    if ui.button("Cancelar").clicked() {
                        rr.quit = true;
                    }
                    if ui.button("Restablecer").clicked() {
                        self.temp_ui_zoom = 1.0;
                        self.temp_text_scale = 1.0;
                    }
                });
            });
            ui.add_space(6.0);
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Configuración de Interfaz");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Zoom UI:");
                    if ui.button("-").clicked() {
                        self.temp_ui_zoom = (self.temp_ui_zoom - 0.1).max(0.5);
                    }
                    let mut zoom_val = self.temp_ui_zoom;
                    if ui.add(DragValue::new(&mut zoom_val).speed(0.01).clamp_range(0.5..=3.0)).changed() {
                        self.temp_ui_zoom = zoom_val;
                    }
                    if ui.button("+").clicked() {
                        self.temp_ui_zoom = (self.temp_ui_zoom + 0.1).min(3.0);
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Texto:");
                    if ui.button("-").clicked() {
                        self.temp_text_scale = (self.temp_text_scale - 0.1).max(0.5);
                    }
                    let mut scale_val = self.temp_text_scale;
                    if ui.add(DragValue::new(&mut scale_val).speed(0.01).clamp_range(0.5..=3.0)).changed() {
                        self.temp_text_scale = scale_val;
                    }
                    if ui.button("+").clicked() {
                        self.temp_text_scale = (self.temp_text_scale + 0.1).min(3.0);
                    }
                });
            });
        });
    }
}
