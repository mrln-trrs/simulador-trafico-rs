mod bars;
mod canvas;
mod components;
mod state;

use egui::{CentralPanel, Color32, Context, Rect, Sense, TopBottomPanel};

use self::bars::menu_bar::draw_menu_bar;
use self::bars::status_bar::draw_status_bar;
use self::canvas::grid::draw_infinite_grid;
use self::canvas::render_cache::GridRenderCache;
use self::canvas::viewport::GridViewport;
use self::state::window_state::SavedWindowState;

#[derive(Default)]
pub struct SimuladorApp {
    window_state: SavedWindowState,
    viewport: GridViewport,
    cache: GridRenderCache,
}

impl SimuladorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let window_state = SavedWindowState::load(cc.storage);
        window_state.apply_to_context(&cc.egui_ctx);

        Self {
            window_state,
            ..Default::default()
        }
    }
}

impl eframe::App for SimuladorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.window_state.sync_from_context(ctx);

        draw_menu_bar(ctx, &mut self.viewport);

        let mut pointer_world = None;
        let mut viewport_rect = Rect::NOTHING;

        CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size_before_wrap();
            let (rect, response) = ui.allocate_exact_size(available_size, Sense::drag());
            let painter = ui.painter_at(rect);
            viewport_rect = rect;
            let mut viewport_changed = false;

            let (zoom_delta, hover_pos) = ui.input(|input| (input.zoom_delta(), input.pointer.hover_pos()));

            if response.dragged() {
                self.viewport.pan += response.drag_delta() / self.viewport.zoom;
                viewport_changed = true;
            }

            if (zoom_delta - 1.0).abs() > f32::EPSILON {
                if let Some(pointer_pos) = hover_pos {
                    self.viewport.zoom_at(rect, pointer_pos, zoom_delta);
                } else {
                    self.viewport.zoom = (self.viewport.zoom * zoom_delta).clamp(self.viewport.min_zoom(), self.viewport.max_zoom());
                }
                viewport_changed = true;
            }

            if viewport_changed {
                ctx.request_repaint();
            }

            pointer_world = hover_pos.and_then(|pointer_pos| {
                rect.contains(pointer_pos)
                    .then(|| self.viewport.screen_to_world(rect, pointer_pos))
            });

            painter.rect_filled(rect, 0.0, Color32::from_rgb(16, 18, 22));
            draw_infinite_grid(&painter, rect, &self.viewport, &mut self.cache);
        });

        TopBottomPanel::bottom("status_bar")
            .show_separator_line(false)
            .show(ctx, |ui| {
                draw_status_bar(ui, &self.viewport, viewport_rect, pointer_world, &mut self.cache);
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.window_state.save(storage);
    }
}