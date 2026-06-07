use egui::{Painter, Rect, Ui, Vec2};

use super::super::canvas::render_cache::{status_font, status_text_color, GridRenderCache};
use super::super::canvas::viewport::GridViewport;
use super::super::canvas::{BASE_GRID_10M_ZOOM_THRESHOLD, SUBGRID_10CM_ZOOM_THRESHOLD, SUBGRID_1CM_ZOOM_THRESHOLD};

pub(crate) fn draw_status_bar(
    ui: &mut Ui,
    viewport: &GridViewport,
    rect: Rect,
    pointer_world: Option<Vec2>,
    cache: &mut GridRenderCache,
    fps: f32,
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
            format!("| Reloj: {} |", chrono::Local::now().format("%H:%M:%S")),
            60.0,
        );
        status_entry(
            ui,
            &painter,
            format!("| Frecuencia: {:.0} Hz (FPS) |", fps),
            60.0,
        );
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

    let size = Vec2::new(min_width.max(text_galley.size().x), text_galley.size().y);
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