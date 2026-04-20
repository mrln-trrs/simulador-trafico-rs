use egui::{Pos2, Rect, Vec2};

const DEFAULT_ZOOM: f32 = 48.0;
const MIN_ZOOM: f32 = 8.0;
const MAX_ZOOM: f32 = 400.0;

#[derive(Clone, Copy)]
pub(crate) struct GridViewport {
    pub(crate) pan: Vec2,
    pub(crate) zoom: f32,
}

impl Default for GridViewport {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: DEFAULT_ZOOM,
        }
    }
}

impl GridViewport {
    pub(crate) fn min_zoom(&self) -> f32 {
        MIN_ZOOM
    }

    pub(crate) fn max_zoom(&self) -> f32 {
        MAX_ZOOM
    }

    pub(crate) fn world_to_screen(&self, rect: Rect, world: Vec2) -> Pos2 {
        rect.center() + (world + self.pan) * self.zoom
    }

    pub(crate) fn screen_to_world(&self, rect: Rect, screen: Pos2) -> Vec2 {
        (screen - rect.center()) / self.zoom - self.pan
    }

    pub(crate) fn zoom_at(&mut self, rect: Rect, screen_point: Pos2, zoom_factor: f32) {
        let world_point = self.screen_to_world(rect, screen_point);
        self.zoom = (self.zoom * zoom_factor).clamp(MIN_ZOOM, MAX_ZOOM);
        self.pan = (screen_point - rect.center()) / self.zoom - world_point;
    }

    pub(crate) fn zoom_by(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    pub(crate) fn reset_zoom(&mut self) {
        self.zoom = DEFAULT_ZOOM;
    }

    pub(crate) fn center_on_origin(&mut self) {
        self.pan = Vec2::ZERO;
    }
}