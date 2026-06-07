use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

use super::render_cache::{grid_label_color, GridRenderCache, VisibleWorldBounds};
use super::viewport::GridViewport;

pub(crate) fn draw_infinite_grid(
    painter: &Painter,
    rect: Rect,
    viewport: &GridViewport,
    cache: &mut GridRenderCache,
    scale: f32,
) {
    let base_step = if viewport.zoom <= super::BASE_GRID_10M_ZOOM_THRESHOLD {
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
        scale,
    );

    if viewport.zoom >= super::SUBGRID_10CM_ZOOM_THRESHOLD {
        draw_step_grid(painter, rect, viewport, visible_world, cache, 0.1, Color32::from_gray(64), 0.5, 10.0, false, scale);
    }

    if viewport.zoom >= super::SUBGRID_1CM_ZOOM_THRESHOLD {
        draw_step_grid(painter, rect, viewport, visible_world, cache, 0.01, Color32::from_gray(54), 0.35, 10.0, false, scale);
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
    scale: f32,
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

    // Margen superior dinámico escalado para evitar la barra de herramientas/menú
    let label_y_offset = 6.0 * scale;
    let label_x_offset = 6.0 * scale;

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
            let label = cache.grid_label_galley(painter, (ix as f32 * step).round() as i64, scale);
            let size = label.size();
            // Evitar que se solape con el menú superior y que se dibuje fuera del canvas
            let y_pos = (rect.top() + label_y_offset).clamp(rect.top(), rect.bottom() - size.y);
            // Evitar que la etiqueta X se corte en los bordes laterales
            let x_pos = (screen_x + 3.0).clamp(rect.left() + label_x_offset, rect.right() - size.x - 3.0);
            labels.push((Pos2::new(x_pos, y_pos), label));
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
            let label = cache.grid_label_galley(painter, (iy as f32 * step).round() as i64, scale);
            let size = label.size();
            // Mantener la etiqueta Y visible en el lateral izquierdo sin recortarse,
            // pero también asegurando que no se solape con el botón de Configuración en el menú superior
            let x_pos = (rect.left() + label_x_offset).clamp(rect.left(), rect.right() - size.x);
            // Evitar el clipping vertical en el borde superior e inferior
            let y_pos = (screen_y + 2.0).clamp(rect.top() + label_y_offset, rect.bottom() - size.y - 2.0);
            labels.push((Pos2::new(x_pos, y_pos), label));
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