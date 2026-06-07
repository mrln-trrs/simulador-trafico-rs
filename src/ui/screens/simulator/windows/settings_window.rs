use egui::{TextStyle, FontId, FontFamily};

pub fn apply_local_scale(ui: &mut egui::Ui, scale: f32) {
    let mut style = ui.style().as_ref().clone();
    style.text_styles.insert(TextStyle::Small, FontId::new(9.0 * scale, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Body, FontId::new(12.5 * scale, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Button, FontId::new(12.5 * scale, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Heading, FontId::new(18.0 * scale, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Monospace, FontId::new(12.0 * scale, FontFamily::Monospace));
    style.spacing.item_spacing *= scale;
    style.spacing.button_padding *= scale;
    ui.set_style(style);
}
