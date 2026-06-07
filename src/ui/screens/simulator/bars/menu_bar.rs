use egui::{menu, Context, TopBottomPanel};
use crate::ui::screens::simulator::SimuladorApp;

pub(crate) fn draw_menu_bar(ctx: &Context, app: &mut SimuladorApp) {
    TopBottomPanel::top("menu_bar")
        .show_separator_line(false)
        .show(ctx, |ui| {
            crate::ui::screens::simulator::windows::settings_window::apply_local_scale(ui, app.ui_zoom * app.text_scale);
            menu::bar(ui, |ui| {
                if ui.selectable_label(app.show_settings_window, "⚙ Configuración").clicked() {
                    app.show_settings_window = !app.show_settings_window;
                }
            });
        });
}