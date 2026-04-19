use std::{collections::HashMap, sync::Arc};

use egui::{CentralPanel, Color32, Context, FontId, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};

const DEFAULT_ZOOM: f32 = 48.0;
const MIN_ZOOM: f32 = 8.0;
const MAX_ZOOM: f32 = 400.0;
const BASE_GRID_10M_ZOOM_THRESHOLD: f32 = 28.0;
const SUBGRID_10CM_ZOOM_THRESHOLD: f32 = 80.0;
const SUBGRID_1CM_ZOOM_THRESHOLD: f32 = 200.0;
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
pub struct SimuladorApp {
    viewport: GridViewport,
    cache: GridRenderCache,
}

#[derive(Default)]
struct GridRenderCache {
    label_galleys: HashMap<i64, Arc<egui::Galley>>,
    status_widths: Option<StatusBarWidths>,
}

#[derive(Clone, Copy)]
struct StatusBarWidths {
    cursor: f32,
    center: f32,
    zoom: f32,
    scale: f32,
    meters_per_pixel: f32,
    level: f32,
    view: f32,
}

#[derive(Clone, Copy)]
struct VisibleWorldBounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

#[derive(Clone, Copy)]
struct GridViewport {
    pan: Vec2,
    zoom: f32,
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
    fn world_to_screen(&self, rect: Rect, world: Vec2) -> Pos2 {
        rect.center() + (world + self.pan) * self.zoom
    }

    fn screen_to_world(&self, rect: Rect, screen: Pos2) -> Vec2 {
        (screen - rect.center()) / self.zoom - self.pan
    }

    fn zoom_at(&mut self, rect: Rect, screen_point: Pos2, zoom_factor: f32) {
        let world_point = self.screen_to_world(rect, screen_point);
        self.zoom = (self.zoom * zoom_factor).clamp(MIN_ZOOM, MAX_ZOOM);
        self.pan = (screen_point - rect.center()) / self.zoom - world_point;
    }
}

impl GridRenderCache {
    fn grid_label_galley(&mut self, painter: &Painter, value_meters: i64) -> Arc<egui::Galley> {
        if let Some(galley) = self.label_galleys.get(&value_meters) {
            return galley.clone();
        }

        if self.label_galleys.len() >= GRID_LABEL_CACHE_LIMIT {
            self.label_galleys.clear();
        }

        let galley = painter.layout_no_wrap(
            format!("{value_meters} m"),
            grid_label_font(),
            grid_label_color(),
        );
        self.label_galleys.insert(value_meters, galley.clone());
        galley
    }

    fn status_widths(&mut self, painter: &Painter) -> StatusBarWidths {
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
    fn from_rect(viewport: &GridViewport, rect: Rect) -> Self {
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

impl eframe::App for SimuladorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
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
                    self.viewport.zoom = (self.viewport.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
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

        egui::TopBottomPanel::bottom("status_bar")
            .show_separator_line(false)
            .show(ctx, |ui| {
                draw_status_bar(ui, &self.viewport, viewport_rect, pointer_world, &mut self.cache);
            });
    }
}

fn draw_infinite_grid(painter: &Painter, rect: Rect, viewport: &GridViewport, cache: &mut GridRenderCache) {
    let base_step = if viewport.zoom <= BASE_GRID_10M_ZOOM_THRESHOLD {
        10.0
    } else {
        1.0
    };

    let visible_world = VisibleWorldBounds::from_rect(viewport, rect);

    draw_step_grid(
        painter,
        rect,
        viewport,
        visible_world,
        cache,
        base_step,
        Color32::from_gray(86),
        1.0,
        10.0,
        true,
    );

    if viewport.zoom >= SUBGRID_10CM_ZOOM_THRESHOLD {
        draw_step_grid(painter, rect, viewport, visible_world, cache, 0.1, Color32::from_gray(64), 0.5, 10.0, false);
    }

    if viewport.zoom >= SUBGRID_1CM_ZOOM_THRESHOLD {
        draw_step_grid(painter, rect, viewport, visible_world, cache, 0.01, Color32::from_gray(54), 0.35, 10.0, false);
    }
}

fn draw_step_grid(
    painter: &Painter,
    rect: Rect,
    viewport: &GridViewport,
    visible_world: VisibleWorldBounds,
    cache: &mut GridRenderCache,
    step: f32,
    color: Color32,
    stroke_width: f32,
    major_every: f32,
    show_labels: bool,
) {
    let start_x = (visible_world.min_x / step).floor() as i32 - 1;
    let end_x = (visible_world.max_x / step).ceil() as i32 + 1;
    let start_y = (visible_world.min_y / step).floor() as i32 - 1;
    let end_y = (visible_world.max_y / step).ceil() as i32 + 1;
    let zoom = viewport.zoom;
    let x_delta = step * zoom;
    let y_delta = step * zoom;
    let screen_origin = rect.center() + viewport.pan * zoom;
    let screen_x_start = screen_origin.x + start_x as f32 * x_delta;
    let screen_y_start = screen_origin.y + start_y as f32 * y_delta;
    let major_every_i32 = show_labels.then_some(major_every.max(1.0) as i32);

    let mut shapes = Vec::with_capacity(
        (end_x - start_x + 1).max(0) as usize + (end_y - start_y + 1).max(0) as usize,
    );
    let mut labels = Vec::new();
    let vertical_range = rect.y_range();
    let horizontal_range = rect.x_range();

    let mut screen_x = screen_x_start;
    for ix in start_x..=end_x {
        let is_major = major_every_i32.is_some_and(|every| ix.rem_euclid(every) == 0);

        let line_color = if is_major {
            Color32::from_gray(112)
        } else {
            color
        };
        let line_width = if is_major { stroke_width * 1.4 } else { stroke_width };

        shapes.push(Shape::line_segment(
            [
                Pos2::new(screen_x, vertical_range.min),
                Pos2::new(screen_x, vertical_range.max),
            ],
            Stroke::new(line_width, line_color),
        ));

        if show_labels && is_major {
            let label = cache.grid_label_galley(painter, (ix as f32 * step).round() as i64);
            labels.push((Pos2::new(screen_x + 3.0, rect.top() + 3.0), label));
        }

        screen_x += x_delta;
    }

    let mut screen_y = screen_y_start;
    for iy in start_y..=end_y {
        let is_major = major_every_i32.is_some_and(|every| iy.rem_euclid(every) == 0);

        let line_color = if is_major {
            Color32::from_gray(112)
        } else {
            color
        };
        let line_width = if is_major { stroke_width * 1.4 } else { stroke_width };

        shapes.push(Shape::line_segment(
            [
                Pos2::new(horizontal_range.min, screen_y),
                Pos2::new(horizontal_range.max, screen_y),
            ],
            Stroke::new(line_width, line_color),
        ));

        if show_labels && is_major {
            let label = cache.grid_label_galley(painter, (iy as f32 * step).round() as i64);
            labels.push((Pos2::new(rect.left() + 4.0, screen_y + 2.0), label));
        }

        screen_y += y_delta;
    }

    painter.extend(shapes);
    for (position, label) in labels {
        painter.galley(position, label, grid_label_color());
    }

    let origin = viewport.world_to_screen(rect, Vec2::ZERO);
    if rect.expand(4.0).contains(origin) {
        painter.circle_filled(origin, 3.5, Color32::from_rgb(220, 120, 80));
    }
}

fn draw_status_bar(
    ui: &mut Ui,
    viewport: &GridViewport,
    rect: Rect,
    pointer_world: Option<Vec2>,
    cache: &mut GridRenderCache,
) {
    let center_world = if rect.is_positive() {
        viewport.screen_to_world(rect, rect.center())
    } else {
        Vec2::ZERO
    };
    let meters_per_pixel = if viewport.zoom > 0.0 { 1.0 / viewport.zoom } else { 0.0 };
    let visible_width_m = if rect.width() > 0.0 { rect.width() / viewport.zoom } else { 0.0 };
    let visible_height_m = if rect.height() > 0.0 { rect.height() / viewport.zoom } else { 0.0 };
    let painter = ui.painter().clone();
    let widths = cache.status_widths(&painter);

    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

        status_entry(
            ui,
            &painter,
            format!("| Posición cursor: {} |", format_world_pos(pointer_world)),
            widths.cursor,
        );
        status_entry(
            ui,
            &painter,
            format!("| Centro de vista: {} |", format_world_pos(Some(center_world))),
            widths.center,
        );
        status_entry(ui, &painter, format!("| Zoom: {:.1} px/m |", viewport.zoom), widths.zoom);
        status_entry(
            ui,
            &painter,
            format!("| Escala: 1 px = {:.4} m |", meters_per_pixel),
            widths.scale,
        );
        status_entry(
            ui,
            &painter,
            format!("| Metro/píxel: 1 m = {:.1} px |", viewport.zoom),
            widths.meters_per_pixel,
        );
        status_entry(
            ui,
            &painter,
            format!("| Nivel: {} |", grid_level_label(viewport.zoom)),
            widths.level,
        );
        status_entry(
            ui,
            &painter,
            format!("| Vista: {:.1} m × {:.1} m |", visible_width_m, visible_height_m),
            widths.view,
        );
    });
    ui.add_space(6.0);
}

fn status_entry(ui: &mut Ui, painter: &Painter, text: String, min_width: f32) {
    let font_id = status_font();
    let text_color = status_text_color();
    let text_galley = painter.layout_no_wrap(text, font_id, text_color);

    let size = Vec2::new(
        min_width.max(text_galley.size().x),
        text_galley.size().y,
    );
    let (_id, rect) = ui.allocate_space(size);
    painter.galley(rect.min, text_galley, text_color);
}

fn format_world_pos(position: Option<Vec2>) -> String {
    match position {
        Some(position) => format!("{:.2} m, {:.2} m", position.x, position.y),
        None => String::from("fuera del plano"),
    }
}

fn grid_level_label(zoom: f32) -> &'static str {
    if zoom >= SUBGRID_1CM_ZOOM_THRESHOLD {
        "Subgrilla 1 cm"
    } else if zoom >= SUBGRID_10CM_ZOOM_THRESHOLD {
        "Subgrilla 10 cm"
    } else if zoom <= BASE_GRID_10M_ZOOM_THRESHOLD {
        "Grilla base 10 m"
    } else {
        "Grilla base 1 m"
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

fn status_font() -> FontId {
    FontId::proportional(STATUS_FONT_SIZE)
}

fn grid_label_color() -> Color32 {
    Color32::from_gray(188)
}

fn status_text_color() -> Color32 {
    Color32::from_rgb(235, 238, 242)
}