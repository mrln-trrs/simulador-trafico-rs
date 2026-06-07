use egui::{Context, Pos2, Vec2, ViewportCommand};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub(crate) struct SavedWindowState {
    maximized: bool,
    windowed_position: Option<[f32; 2]>,
    windowed_inner_size: Option<[f32; 2]>,
}

impl SavedWindowState {
    pub(crate) fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        storage
            .and_then(|storage| eframe::get_value(storage, "simulador_window_state"))
            .unwrap_or_default()
    }

    pub(crate) fn apply_to_context(self, ctx: &Context) {
        if self.maximized {
            ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
            return;
        }

        if let Some(position) = self.windowed_position {
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::new(position[0], position[1])));
        }

        if let Some(inner_size) = self.windowed_inner_size {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(inner_size[0], inner_size[1])));
        }
    }

    pub(crate) fn sync_from_context(&mut self, ctx: &Context) {
        // Solo sincronizar el estado si es el viewport raíz (ventana principal)
        let is_root = ctx.viewport_id() == egui::ViewportId::ROOT;
        if !is_root {
            return;
        }

        let (maximized, inner_rect, outer_rect) = ctx.input(|input| {
            let viewport = input.viewport();
            (viewport.maximized, viewport.inner_rect, viewport.outer_rect)
        });

        if let Some(maximized) = maximized {
            self.maximized = maximized;
        }

        if self.maximized {
            return;
        }

        if let (Some(inner_rect), Some(outer_rect)) = (inner_rect, outer_rect) {
            self.windowed_position = Some([outer_rect.min.x, outer_rect.min.y]);
            self.windowed_inner_size = Some([inner_rect.width(), inner_rect.height()]);
        }
    }

    pub(crate) fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "simulador_window_state", self);
    }
}