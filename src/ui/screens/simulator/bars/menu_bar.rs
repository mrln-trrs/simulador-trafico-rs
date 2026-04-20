use egui::{menu, Context, TopBottomPanel};

use super::super::canvas::viewport::GridViewport;

const MANUAL_ZOOM_STEP: f32 = 1.15;

pub(crate) fn draw_menu_bar(ctx: &Context, viewport: &mut GridViewport) {
    TopBottomPanel::top("menu_bar")
        .show_separator_line(false)
        .show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("Vista", |ui| {
                    if ui.button("Zoom +").clicked() {
                        viewport.zoom_by(MANUAL_ZOOM_STEP);
                        ui.close_menu();
                        ctx.request_repaint();
                    }

                    if ui.button("Zoom -").clicked() {
                        viewport.zoom_by(1.0 / MANUAL_ZOOM_STEP);
                        ui.close_menu();
                        ctx.request_repaint();
                    }

                    ui.separator();

                    if ui.button("Restablecer zoom").clicked() {
                        viewport.reset_zoom();
                        ui.close_menu();
                        ctx.request_repaint();
                    }

                    if ui.button("Centrar vista al origen 0,0").clicked() {
                        viewport.center_on_origin();
                        ui.close_menu();
                        ctx.request_repaint();
                    }
                });
            });
        });
}