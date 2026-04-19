use egui::{Align2, CentralPanel, Color32, Context, FontId, Painter, Pos2, Rect, Sense, Stroke, Vec2};

const DEFAULT_ZOOM: f32 = 48.0;
const MIN_ZOOM: f32 = 8.0;
const MAX_ZOOM: f32 = 400.0;
const SUBGRID_10CM_ZOOM_THRESHOLD: f32 = 60.0;
const SUBGRID_1CM_ZOOM_THRESHOLD: f32 = 120.0;

#[derive(Default)]
pub struct SimuladorApp {
    viewport: GridViewport,
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

impl eframe::App for SimuladorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size_before_wrap();
            let (rect, response) = ui.allocate_exact_size(available_size, Sense::drag());
            let painter = ui.painter_at(rect);

            if response.dragged() {
                self.viewport.pan += response.drag_delta() / self.viewport.zoom;
                ctx.request_repaint();
            }

            let zoom_delta = ui.input(|input| input.zoom_delta());
            if (zoom_delta - 1.0).abs() > f32::EPSILON {
                if let Some(pointer_pos) = ui.input(|input| input.pointer.hover_pos()) {
                    self.viewport.zoom_at(rect, pointer_pos, zoom_delta);
                } else {
                    self.viewport.zoom = (self.viewport.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
                }
                ctx.request_repaint();
            }

            painter.rect_filled(rect, 0.0, Color32::from_rgb(16, 18, 22));
            draw_infinite_grid(&painter, rect, &self.viewport);
            draw_grid_legend(&painter, rect, &self.viewport);
        });
    }
}

fn draw_infinite_grid(painter: &Painter, rect: Rect, viewport: &GridViewport) {
    draw_step_grid(painter, rect, viewport, 1.0, Color32::from_gray(86), 1.0, 10.0, true);

    if viewport.zoom >= SUBGRID_10CM_ZOOM_THRESHOLD {
        draw_step_grid(painter, rect, viewport, 0.1, Color32::from_gray(64), 0.5, 10.0, false);
    }

    if viewport.zoom >= SUBGRID_1CM_ZOOM_THRESHOLD {
        draw_step_grid(painter, rect, viewport, 0.01, Color32::from_gray(54), 0.35, 10.0, false);
    }
}

fn draw_step_grid(
    painter: &Painter,
    rect: Rect,
    viewport: &GridViewport,
    step: f32,
    color: Color32,
    stroke_width: f32,
    major_every: f32,
    show_labels: bool,
) {
    let top_left = viewport.screen_to_world(rect, rect.left_top());
    let bottom_right = viewport.screen_to_world(rect, rect.right_bottom());

    let min_x = top_left.x.min(bottom_right.x);
    let max_x = top_left.x.max(bottom_right.x);
    let min_y = top_left.y.min(bottom_right.y);
    let max_y = top_left.y.max(bottom_right.y);

    let start_x = (min_x / step).floor() as i32 - 1;
    let end_x = (max_x / step).ceil() as i32 + 1;
    let start_y = (min_y / step).floor() as i32 - 1;
    let end_y = (max_y / step).ceil() as i32 + 1;

    for ix in start_x..=end_x {
        let world_x = ix as f32 * step;
        let screen_x = viewport.world_to_screen(rect, Vec2::new(world_x, 0.0)).x;
        let is_major = if step >= 1.0 {
            (world_x.round() as i32).rem_euclid(major_every as i32) == 0
        } else {
            false
        };

        let line_color = if is_major {
            Color32::from_gray(112)
        } else {
            color
        };
        let line_width = if is_major { stroke_width * 1.4 } else { stroke_width };

        painter.line_segment(
            [Pos2::new(screen_x, rect.top()), Pos2::new(screen_x, rect.bottom())],
            Stroke::new(line_width, line_color),
        );

        if show_labels && is_major {
            painter.text(
                Pos2::new(screen_x + 3.0, rect.top() + 3.0),
                Align2::LEFT_TOP,
                format!("{:.0} m", world_x),
                FontId::proportional(10.0),
                Color32::from_gray(188),
            );
        }
    }

    for iy in start_y..=end_y {
        let world_y = iy as f32 * step;
        let screen_y = viewport.world_to_screen(rect, Vec2::new(0.0, world_y)).y;
        let is_major = if step >= 1.0 {
            (world_y.round() as i32).rem_euclid(major_every as i32) == 0
        } else {
            false
        };

        let line_color = if is_major {
            Color32::from_gray(112)
        } else {
            color
        };
        let line_width = if is_major { stroke_width * 1.4 } else { stroke_width };

        painter.line_segment(
            [Pos2::new(rect.left(), screen_y), Pos2::new(rect.right(), screen_y)],
            Stroke::new(line_width, line_color),
        );

        if show_labels && is_major {
            painter.text(
                Pos2::new(rect.left() + 4.0, screen_y + 2.0),
                Align2::LEFT_CENTER,
                format!("{:.0} m", world_y),
                FontId::proportional(10.0),
                Color32::from_gray(188),
            );
        }
    }

    let origin = viewport.world_to_screen(rect, Vec2::ZERO);
    painter.circle_filled(origin, 3.5, Color32::from_rgb(220, 120, 80));
}

fn draw_grid_legend(painter: &Painter, rect: Rect, viewport: &GridViewport) {
    let panel_size = Vec2::new(230.0, if viewport.zoom >= SUBGRID_1CM_ZOOM_THRESHOLD { 72.0 } else { 52.0 });
    let panel_rect = Rect::from_min_size(rect.left_top() + Vec2::new(10.0, 10.0), panel_size);

    painter.rect_filled(panel_rect, 6.0, Color32::from_black_alpha(140));
    painter.rect_stroke(panel_rect, 6.0, Stroke::new(1.0, Color32::from_white_alpha(24)));

    painter.text(
        panel_rect.left_top() + Vec2::new(10.0, 10.0),
        Align2::LEFT_TOP,
        "Plano base",
        FontId::proportional(13.0),
        Color32::from_rgb(235, 235, 235),
    );

    painter.text(
        panel_rect.left_top() + Vec2::new(10.0, 28.0),
        Align2::LEFT_TOP,
        "Cada cuadricula base = 1 m x 1 m",
        FontId::proportional(11.0),
        Color32::from_gray(205),
    );

    painter.text(
        panel_rect.left_top() + Vec2::new(10.0, 42.0),
        Align2::LEFT_TOP,
        format!("Zoom actual: {:.0} px/m", viewport.zoom),
        FontId::proportional(11.0),
        Color32::from_gray(180),
    );

    if viewport.zoom >= SUBGRID_1CM_ZOOM_THRESHOLD {
        painter.text(
            panel_rect.left_top() + Vec2::new(10.0, 56.0),
            Align2::LEFT_TOP,
            "Subgrilla visible hasta 1 cm x 1 cm",
            FontId::proportional(11.0),
            Color32::from_rgb(165, 210, 255),
        );
    }
}