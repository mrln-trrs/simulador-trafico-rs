use egui::{
    Align, Button, Color32, Context, Frame, Id, Layout, Margin, SidePanel,
    Stroke, TopBottomPanel, Ui, Vec2,
};
use serde::{Deserialize, Serialize};

/// Representa los lados en los que se puede acoplar el menú lateral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidebarPosition {
    Left,
    Right,
    Top,
    Bottom,
}

/// Estructura de datos para cada elemento de la barra de herramientas.
#[derive(Debug, Clone)]
pub struct SidebarItem<T: Clone + PartialEq> {
    /// Valor único asociado al ítem (para detectar clics y estado activo).
    pub value: T,
    /// Icono (puede ser un emoji, una letra o un icono de fuente).
    pub icon: String,
    /// Etiqueta descriptiva que se muestra al expandir.
    pub label: String,
    /// Texto de ayuda flotante (tooltip).
    pub tooltip: String,
}

pub struct Sidebar<'a, T: Clone + PartialEq> {
    id: Id,
    position: SidebarPosition,
    items: &'a [SidebarItem<T>],
}

impl<'a, T: Clone + PartialEq> Sidebar<'a, T> {
    pub fn new(id: impl std::hash::Hash, position: SidebarPosition, items: &'a [SidebarItem<T>]) -> Self {
        Self {
            id: Id::new(id),
            position,
            items,
        }
    }

    /// Dibuja el panel y retorna `Some(value)` si el usuario hace clic en alguna opción.
    pub fn show(
        self,
        ctx: &Context,
        is_expanded: &mut bool,
        selected_value: &mut Option<T>,
        scale: f32,
    ) -> Option<T> {
        let mut clicked_val = None;

        // Estilo premium oscuro y minimalista
        let frame = Frame {
            fill: Color32::from_rgb(20, 22, 26),
            inner_margin: Margin::same(8.0),
            outer_margin: Margin::ZERO,
            rounding: egui::Rounding::ZERO,
            shadow: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(32, 35, 42)),
        };

        match self.position {
            SidebarPosition::Left => {
                let width = if *is_expanded { 160.0 } else { 48.0 };
                SidePanel::left(self.id)
                    .resizable(false)
                    .exact_width(width)
                    .frame(frame)
                    .show(ctx, |ui| {
                        crate::ui::screens::simulator::windows::settings_window::apply_local_scale(ui, scale);
                        clicked_val = self.draw_vertical_content(ui, is_expanded, selected_value);
                    });
            }
            SidebarPosition::Right => {
                let width = if *is_expanded { 160.0 } else { 48.0 };
                SidePanel::right(self.id)
                    .resizable(false)
                    .exact_width(width)
                    .frame(frame)
                    .show(ctx, |ui| {
                        crate::ui::screens::simulator::windows::settings_window::apply_local_scale(ui, scale);
                        clicked_val = self.draw_vertical_content(ui, is_expanded, selected_value);
                    });
            }
            SidebarPosition::Top => {
                TopBottomPanel::top(self.id)
                    .show_separator_line(false)
                    .frame(frame)
                    .show(ctx, |ui| {
                        crate::ui::screens::simulator::windows::settings_window::apply_local_scale(ui, scale);
                        clicked_val = self.draw_horizontal_content(ui, is_expanded, selected_value);
                    });
            }
            SidebarPosition::Bottom => {
                TopBottomPanel::bottom(self.id)
                    .show_separator_line(false)
                    .frame(frame)
                    .show(ctx, |ui| {
                        crate::ui::screens::simulator::windows::settings_window::apply_local_scale(ui, scale);
                        clicked_val = self.draw_horizontal_content(ui, is_expanded, selected_value);
                    });
            }
        }

        clicked_val
    }

    /// Dibuja el menú vertical (Left / Right)
    fn draw_vertical_content(
        &self,
        ui: &mut Ui,
        is_expanded: &mut bool,
        selected_value: &mut Option<T>,
    ) -> Option<T> {
        let mut clicked_val = None;

        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
            // Elementos de la barra
            for item in self.items {
                let is_selected = selected_value.as_ref() == Some(&item.value);
                
                let text = if *is_expanded {
                    format!("{}  {}", item.icon, item.label)
                } else {
                    item.icon.clone()
                };

                let mut btn = Button::new(text).min_size(Vec2::new(0.0, 36.0));
                
                if is_selected {
                    btn = btn.fill(Color32::from_rgb(59, 130, 246)); // Azul premium
                } else {
                    btn = btn.fill(Color32::TRANSPARENT);
                }

                let response = ui.add(btn).on_hover_text(&item.tooltip);
                if response.clicked() {
                    if is_selected {
                        *selected_value = None;
                    } else {
                        *selected_value = Some(item.value.clone());
                        clicked_val = Some(item.value.clone());
                    }
                }
            }

            // Espaciador flexible para empujar el botón de colapsar al fondo
            let space_left = ui.available_size().y - 40.0;
            if space_left > 0.0 {
                ui.allocate_space(Vec2::new(ui.available_size().x, space_left));
            }

            // Botón para colapsar/expandir
            let toggle_text = if *is_expanded { "◀ Colapsar" } else { "▶" };
            if ui.button(toggle_text).clicked() {
                *is_expanded = !*is_expanded;
            }
        });

        clicked_val
    }

    /// Dibuja el menú horizontal (Top / Bottom)
    fn draw_horizontal_content(
        &self,
        ui: &mut Ui,
        is_expanded: &mut bool,
        selected_value: &mut Option<T>,
    ) -> Option<T> {
        let mut clicked_val = None;

        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            // Elementos de la barra
            for item in self.items {
                let is_selected = selected_value.as_ref() == Some(&item.value);

                let text = if *is_expanded {
                    format!("{}  {}", item.icon, item.label)
                } else {
                    item.icon.clone()
                };

                let mut btn = Button::new(text).min_size(Vec2::new(0.0, 32.0));
                if is_selected {
                    btn = btn.fill(Color32::from_rgb(59, 130, 246));
                } else {
                    btn = btn.fill(Color32::TRANSPARENT);
                }

                let response = ui.add(btn).on_hover_text(&item.tooltip);
                if response.clicked() {
                    if is_selected {
                        *selected_value = None;
                    } else {
                        *selected_value = Some(item.value.clone());
                        clicked_val = Some(item.value.clone());
                    }
                }
            }

            // Espaciador flexible y botón de colapsar a la derecha
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let toggle_text = if *is_expanded { "▲ Colapsar" } else { "▼" };
                if ui.button(toggle_text).clicked() {
                    *is_expanded = !*is_expanded;
                }
            });
        });

        clicked_val
    }
}
