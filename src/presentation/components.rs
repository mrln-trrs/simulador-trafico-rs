use egui::*;
use crate::presentation::theme::{FluentTheme, ComponentState, spacing, rounding, typography};

/// Botón Fluent minimalista
pub fn fluent_button(
    ui: &mut Ui,
    text: &str,
    theme: &FluentTheme,
    state: ComponentState,
) -> bool {
    let button = Button::new(
        RichText::new(text)
            .size(typography::BODY)
            .color(theme.text_primary)
    )
    .fill(match state {
        ComponentState::Hovered => theme.interactive_hover,
        ComponentState::Pressed => theme.interactive_pressed,
        ComponentState::Disabled => theme.interactive_disabled,
        _ => theme.surface_variant,
    })
    .stroke(Stroke::new(1.0, theme.outline_variant))
    .rounding(rounding::medium())
    .min_size(Vec2::new(80.0, 36.0));
    
    ui.add_enabled(
        !matches!(state, ComponentState::Disabled),
        button
    ).clicked()
}

/// Botón pequeño (herramientas)
pub fn fluent_icon_button(
    ui: &mut Ui,
    icon: &str,
    tooltip: &str,
    theme: &FluentTheme,
    state: ComponentState,
) -> bool {
    let size = 32.0;
    let button = Button::new(
        RichText::new(icon)
            .size(16.0)
            .color(theme.text_primary)
    )
    .fill(match state {
        ComponentState::Hovered => theme.interactive_hover,
        ComponentState::Pressed => theme.interactive_pressed,
        ComponentState::Disabled => theme.interactive_disabled,
        _ => Color32::TRANSPARENT,
    })
    .stroke(Stroke::new(0.5, theme.outline_variant))
    .rounding(rounding::small())
    .min_size(Vec2::new(size, size));
    
    ui.add_enabled(
        !matches!(state, ComponentState::Disabled),
        button
    )
    .on_hover_text(tooltip)
    .clicked()
}

/// Campo de texto Fluent
pub fn fluent_text_input(
    ui: &mut Ui,
    label: &str,
    value: &mut String,
    theme: &FluentTheme,
) -> Response {
    ui.vertical(|ui| {
        ui.label(
            RichText::new(label)
                .size(typography::BODY)
                .color(theme.text_secondary)
        );
        
        let response = ui.text_edit_singleline(value);
        let painter = ui.painter();
        
        // Dibuja un borde sutil
        painter.rect_stroke(
            response.rect,
            rounding::small(),
            Stroke::new(1.0, theme.outline_variant)
        );
        
        response
    }).inner
}

/// Slider minimalista
pub fn fluent_slider(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    theme: &FluentTheme,
) {
    ui.vertical(|ui| {
        ui.label(
            RichText::new(label)
                .size(typography::BODY)
                .color(theme.text_secondary)
        );
        
        ui.add(
            Slider::new(value, range)
                .show_value(true)
        );
    });
}

/// Panel minimalista con borde
pub fn fluent_panel<R>(
    ui: &mut Ui,
    title: Option<&str>,
    theme: &FluentTheme,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    Frame::none()
        .fill(theme.surface)
        .stroke(Stroke::new(1.0, theme.outline_variant))
        .rounding(rounding::medium())
        .inner_margin(spacing::normal())
        .show(ui, |ui| {
            if let Some(title) = title {
                ui.label(
                    RichText::new(title)
                        .size(typography::SUBTITLE)
                        .color(theme.text_primary)
                        .strong()
                );
                ui.separator();
            }
            
            add_contents(ui)
        })
        .inner
}

/// Badge/etiqueta pequeña
pub fn fluent_badge(
    ui: &mut Ui,
    text: &str,
    color: Color32,
    theme: &FluentTheme,
) {
    Frame::none()
        .fill(color.gamma_multiply(0.2))
        .stroke(Stroke::new(1.0, color))
        .rounding(rounding::circle())
        .inner_margin(Margin { left: 8.0, right: 8.0, top: 4.0, bottom: 4.0 })
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(typography::CAPTION)
                    .color(color)
            );
        });
}

/// Card/tarjeta con contenido
pub fn fluent_card<R>(
    ui: &mut Ui,
    theme: &FluentTheme,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    Frame::none()
        .fill(theme.surface_variant)
        .stroke(Stroke::new(0.5, theme.outline_variant))
        .rounding(rounding::medium())
        .inner_margin(egui::Margin::same(spacing::LARGE))
        .show(ui, |ui| {
            add_contents(ui)
        })
        .inner
}

/// Separador minimalista
pub fn fluent_separator(ui: &mut Ui, theme: &FluentTheme) {
    let (_id, rect) = ui.allocate_space(Vec2::new(ui.available_width(), 2.0));
    let painter = ui.painter();
    let y = rect.center().y;
    painter.hline(rect.x_range(), y, Stroke::new(0.5, theme.outline_variant));
    ui.add_space(spacing::MEDIUM);
}

/// Campo numérico
pub fn fluent_number_input(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    theme: &FluentTheme,
) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .size(typography::BODY)
                .color(theme.text_secondary)
        );
        
        let mut text = value.to_string();
        ui.text_edit_singleline(&mut text);
        if let Ok(v) = text.parse::<f32>() {
            *value = v;
        }
    });
}

/// Toggle/checkbox minimalista
pub fn fluent_toggle(
    ui: &mut Ui,
    label: &str,
    checked: &mut bool,
    theme: &FluentTheme,
) -> Response {
    ui.horizontal(|ui| {
        let response = ui.checkbox(checked, "");
        ui.label(
            RichText::new(label)
                .size(typography::BODY)
                .color(theme.text_primary)
        );
        response
    }).inner
}

/// Indicador de estado con texto
pub fn fluent_status(
    ui: &mut Ui,
    label: &str,
    color: Color32,
    theme: &FluentTheme,
) {
    ui.horizontal(|ui| {
        let painter = ui.painter();
        let pos = ui.cursor().center();
        painter.circle_filled(pos, 3.0, color);
        ui.add_space(6.0);
        ui.label(
            RichText::new(label)
                .size(typography::BODY)
                .color(theme.text_secondary)
        );
    });
}
