use egui::{Color32, Context, Painter, Rect, Response};
use crate::ui::screens::simulator::{SimuladorApp, Tool};
use crate::ui::screens::simulator::geom::building_collides_with_roads;

pub fn handle_building_tool(
    app: &mut SimuladorApp,
    _ctx: &Context,
    rect: Rect,
    response: &Response,
    painter: &Painter,
    pointer_world: Option<egui::Vec2>,
    step: f32,
) {
    if app.selected_tool == Some(Tool::Building) {
        if let Some(p_world) = pointer_world {
            // Snap magnético
            let snapped_x = (p_world.x / step).round() * step;
            let snapped_y = (p_world.y / step).round() * step;
            let snapped_pos = egui::vec2(snapped_x, snapped_y);
            let snapped_screen = app.viewport.world_to_screen(rect, snapped_pos);

            // Acción: Click izquierdo para colocar un vértice
            if response.clicked_by(egui::PointerButton::Primary) {
                if !app.building_draft.is_empty() {
                    let first_screen = app.viewport.world_to_screen(rect, app.building_draft[0]);
                    // Si hacemos clic cerca del vértice inicial, intentamos cerrar el polígono
                    if first_screen.distance(snapped_screen) < 15.0 {
                        if app.building_draft.len() >= 3 {
                            // Comprobar colisión bidireccional: que el nuevo edificio no pise carreteras existentes
                            if !building_collides_with_roads(&app.building_draft, &app.road_segments) {
                                app.obstacles.push(app.building_draft.clone());
                            }
                        }
                        app.building_draft.clear();
                    } else {
                        app.building_draft.push(snapped_pos);
                    }
                } else {
                    app.building_draft.push(snapped_pos);
                }
            }

            // Acción: Click derecho para cerrar el polígono actual
            if response.clicked_by(egui::PointerButton::Secondary) {
                if app.building_draft.len() >= 3 {
                    if !building_collides_with_roads(&app.building_draft, &app.road_segments) {
                        app.obstacles.push(app.building_draft.clone());
                    }
                }
                app.building_draft.clear();
            }

            // Dibujar el borrador actual y líneas guía
            if !app.building_draft.is_empty() {
                // Comprobar si el borrador actual colisiona con pistas para alertar al usuario
                let mut coll_check = app.building_draft.clone();
                coll_check.push(snapped_pos);
                let collides = building_collides_with_roads(&coll_check, &app.road_segments);

                let preview_line_color = if collides { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(59, 130, 246) };
                let points: Vec<egui::Pos2> = app.building_draft.iter().map(|&pt| app.viewport.world_to_screen(rect, pt)).collect();

                // Líneas del borrador
                for i in 0..(points.len() - 1) {
                    painter.line_segment([points[i], points[i+1]], egui::Stroke::new(2.5, preview_line_color));
                }

                // Vértices del borrador
                for &pt in &points {
                    painter.circle_filled(pt, 4.0, preview_line_color);
                }

                // Línea de previsualización (elástica) hasta el cursor
                painter.line_segment([*points.last().unwrap(), snapped_screen], egui::Stroke::new(1.5, preview_line_color));

                // Si el cursor está cerca del inicio, dar feedback visual de cierre
                let first_screen = points[0];
                if first_screen.distance(snapped_screen) < 15.0 {
                    painter.circle_filled(first_screen, 8.0, preview_line_color.linear_multiply(0.5));
                } else {
                    painter.circle_filled(snapped_screen, 4.0, preview_line_color);
                }
            } else {
                // Mostrar punto guía inicial
                painter.circle_filled(snapped_screen, 5.0, Color32::from_rgb(59, 130, 246));
            }
        }
    } else {
        // Limpiar el borrador si el usuario cambia de herramienta activa
        app.building_draft.clear();
    }
}
