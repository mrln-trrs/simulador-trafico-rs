use std::{collections::HashMap, sync::Arc};

use egui::{Color32, FontId, Painter, Rect};

use super::viewport::GridViewport;

const GRID_LABEL_CACHE_LIMIT: usize = 256;
const GRID_LABEL_FONT_SIZE: f32 = 10.0;
const STATUS_FONT_SIZE: f32 = 12.0;

const STATUS_CURSOR_TEMPLATE: &str = "| Posición cursor: -1000.00 m, -1000.00 m |";
const STATUS_CENTER_TEMPLATE: &str = "| Centro de vista: -1000.00 m, -1000.00 m |";
const STATUS_ZOOM_TEMPLATE: &str = "| Zoom: 400.0 px/m |";
const STATUS_SCALE_TEMPLATE: &str = "| Escala: 1 px = -1000.0000 m |";
const STATUS_METERS_PER_PIXEL_TEMPLATE: &str = "| Metro/píxel: 1 m = 400.0 px |";
const STATUS_LEVEL_TEMPLATE: &str = "| Nivel: Subgrilla 10 cm |";
const STATUS_VIEW_TEMPLATE: &str = "| Vista: 9999.9 m × 9999.9 m |";

#[derive(Default)]
pub(crate) struct GridRenderCache {
    label_galleys: HashMap<i64, Arc<egui::Galley>>,
    status_widths: Option<StatusBarWidths>,
}

#[derive(Clone, Copy)]
pub(crate) struct VisibleWorldBounds {
    pub(crate) min_x: f32,
    pub(crate) max_x: f32,
    pub(crate) min_y: f32,
    pub(crate) max_y: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct StatusBarWidths {
    pub(crate) cursor: f32,
    pub(crate) center: f32,
    pub(crate) zoom: f32,
    pub(crate) scale: f32,
    pub(crate) meters_per_pixel: f32,
    pub(crate) level: f32,
    pub(crate) view: f32,
}

impl GridRenderCache {
    pub(crate) fn grid_label_galley(&mut self, painter: &Painter, value_meters: i64, scale: f32) -> Arc<egui::Galley> {
        if let Some(galley) = self.label_galleys.get(&value_meters) {
            return galley.clone();
        }

        if self.label_galleys.len() >= GRID_LABEL_CACHE_LIMIT {
            self.label_galleys.clear();
        }

        let font_size = GRID_LABEL_FONT_SIZE * scale;
        let galley = painter.layout_no_wrap(
            format!("{value_meters} m"),
            FontId::proportional(font_size),
            grid_label_color(),
        );
        self.label_galleys.insert(value_meters, galley.clone());
        galley
    }

    pub(crate) fn status_widths(&mut self, painter: &Painter) -> StatusBarWidths {
        *self.status_widths.get_or_insert_with(|| StatusBarWidths::measure(painter))
    }
}

impl StatusBarWidths {
    fn measure(painter: &Painter) -> Self {
        Self {
            cursor: measure_template_width(painter, STATUS_CURSOR_TEMPLATE),
            center: measure_template_width(painter, STATUS_CENTER_TEMPLATE),
            zoom: measure_template_width(painter, STATUS_ZOOM_TEMPLATE),
            scale: measure_template_width(painter, STATUS_SCALE_TEMPLATE),
            meters_per_pixel: measure_template_width(painter, STATUS_METERS_PER_PIXEL_TEMPLATE),
            level: measure_template_width(painter, STATUS_LEVEL_TEMPLATE),
            view: measure_template_width(painter, STATUS_VIEW_TEMPLATE),
        }
    }
}

impl VisibleWorldBounds {
    pub(crate) fn from_rect(viewport: &GridViewport, rect: Rect) -> Self {
        let top_left = viewport.screen_to_world(rect, rect.left_top());
        let bottom_right = viewport.screen_to_world(rect, rect.right_bottom());

        Self {
            min_x: top_left.x.min(bottom_right.x),
            max_x: top_left.x.max(bottom_right.x),
            min_y: top_left.y.min(bottom_right.y),
            max_y: top_left.y.max(bottom_right.y),
        }
    }
}

fn measure_template_width(painter: &Painter, template: &str) -> f32 {
    painter
        .layout_no_wrap(template.to_owned(), status_font(), status_text_color())
        .size()
        .x
}

fn grid_label_font() -> FontId {
    FontId::proportional(GRID_LABEL_FONT_SIZE)
}

pub(crate) fn status_font() -> FontId {
    FontId::proportional(STATUS_FONT_SIZE)
}

pub(crate) fn grid_label_color() -> Color32 {
    Color32::from_gray(188)
}

pub(crate) fn status_text_color() -> Color32 {
    Color32::from_rgb(235, 238, 242)
}