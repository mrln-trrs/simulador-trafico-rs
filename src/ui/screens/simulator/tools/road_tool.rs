use egui::{Color32, Context, Painter, Rect, Response};
use crate::ui::screens::simulator::{RoadSegmentGeometry, SimuladorApp, Tool};
use crate::ui::screens::simulator::geom::{
    road_collides_with_obstacles, road_collides_with_any_road,
    snap_to_elements,
};

pub fn handle_road_tool(
    app: &mut SimuladorApp,
    ctx: &Context,
    rect: Rect,
    response: &Response,
    painter: &Painter,
    pointer_world: Option<egui::Vec2>,
    step: f32,
) {
    if app.selected_tool == Some(Tool::Road) {
        if let Some(p_world) = pointer_world {
            // Snap a la rejilla de 1.5m (= medio carril).
            // Esto garantiza que los BORDES de la pista siempre caigan
            // en líneas exactas: bordes = centro ± (lanes × 1.5m).
            // Con step=1.5m: borde de 1 carril en 0/3/6/9..., 3 carriles en 0/9/18... ✓
            let road_snap_step: f32 = if step >= 5.0 {
                3.0  // Zoom muy alejado: snap cada 3m (alineación gruesa)
            } else {
                1.5  // Snap preciso: medio carril
            };
            let snapped_x = (p_world.x / road_snap_step).round() * road_snap_step;
            let snapped_y = (p_world.y / road_snap_step).round() * road_snap_step;
            let grid_snapped = egui::vec2(snapped_x, snapped_y);

            // Snap magnético a elementos cercanos (centros de pistas y vértices de edificios)
            let snapped_pos = snap_to_elements(grid_snapped, &app.road_segments, &app.obstacles, None, None);
            let snapped_screen = app.viewport.world_to_screen(rect, snapped_pos);

            let road_width = app.road_lanes as f32 * 3.0;

            // Dibujar el círculo guía de centrado de 1.5m de radio con animación
            let time = ctx.input(|i| i.time);
            let opacity = 0.25 + 0.15 * ((time * std::f64::consts::PI).sin() as f32);
            let guide_color = Color32::from_rgba_unmultiplied(59, 130, 246, (opacity * 255.0) as u8);
            let rotation_angle = (time * 0.5 * std::f64::consts::PI) % (2.0 * std::f64::consts::PI);

            let segments = 24;
            let mut guide_points = Vec::new();
            for i in 0..segments {
                let angle = rotation_angle + (i as f64 * 2.0 * std::f64::consts::PI / segments as f64);
                let offset = egui::vec2(angle.cos() as f32 * 1.5, angle.sin() as f32 * 1.5);
                let pt = app.viewport.world_to_screen(rect, snapped_pos + offset);
                guide_points.push(pt);
            }
            for i in (0..segments).step_by(2) {
                painter.line_segment(
                    [guide_points[i], guide_points[(i + 1) % segments]],
                    egui::Stroke::new(1.5, guide_color),
                );
            }
            painter.circle_filled(snapped_screen, 3.0, guide_color);

            // Acción: Click izquierdo para colocar/anclar calle
            if response.clicked_by(egui::PointerButton::Primary) {
                if let Some(start_pos) = app.road_draft {
                    if start_pos != snapped_pos {
                        // Validar colisiones antes de colocar la pista:
                        // 1) No puede solaparse con edificios
                        let collides_obs = road_collides_with_obstacles(start_pos, snapped_pos, road_width, &app.obstacles);
                        // 2) No puede solaparse con otras pistas
                        let collides_roads = road_collides_with_any_road(start_pos, snapped_pos, road_width, &app.road_segments);

                        if !collides_obs && !collides_roads {
                            // Crear como UN SOLO macrosegmento continuo y rápido
                            let road_id = app.next_road_id;
                            app.next_road_id += 1;
                            
                            app.road_segments.push(RoadSegmentGeometry {
                                from: start_pos,
                                to: snapped_pos,
                                lanes: app.road_lanes,
                                id: road_id,
                            });

                            // Encadenar trazado
                            app.road_draft = Some(snapped_pos);
                        }
                        // Si hay colisión: no se hace nada (el preview en rojo ya advierte al usuario)
                    }
                } else {
                    app.road_draft = Some(snapped_pos);
                }
            }

            // Acción: Click derecho para cancelar/terminar trazado elástico
            if response.clicked_by(egui::PointerButton::Secondary) {
                app.road_draft = None;
            }

            // Renderizar borrador/elástico actual de la carretera
            if let Some(start_pos) = app.road_draft {
                if start_pos != snapped_pos {
                    // Comprobar colisión para decidir el color del preview
                    let collides_obs = road_collides_with_obstacles(start_pos, snapped_pos, road_width, &app.obstacles);
                    let collides_roads = road_collides_with_any_road(start_pos, snapped_pos, road_width, &app.road_segments);
                    let collides = collides_obs || collides_roads;

                    let preview_color = if collides {
                        Color32::from_rgba_unmultiplied(239, 68, 68, 60) // Rojo transparente
                    } else {
                        Color32::from_rgba_unmultiplied(59, 130, 246, 60) // Azul transparente
                    };
                    let border_color = if collides { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(59, 130, 246) };

                    let dir = (snapped_pos - start_pos).normalized();
                    let normal = egui::vec2(-dir.y, dir.x);
                    let offset = normal * (road_width / 2.0);

                    let pts = [
                        app.viewport.world_to_screen(rect, start_pos + offset),
                        app.viewport.world_to_screen(rect, snapped_pos + offset),
                        app.viewport.world_to_screen(rect, snapped_pos - offset),
                        app.viewport.world_to_screen(rect, start_pos - offset),
                    ];

                    // Dibujar rectángulo del asfalto temporal
                    painter.add(egui::Shape::convex_polygon(
                        pts.to_vec(),
                        preview_color,
                        egui::Stroke::new(1.5, border_color),
                    ));

                    // Línea del eje central
                    painter.line_segment(
                        [app.viewport.world_to_screen(rect, start_pos), snapped_screen],
                        egui::Stroke::new(1.0, border_color),
                    );

                    // Etiqueta de colisión
                    if collides_roads {
                        painter.text(
                            snapped_screen + egui::vec2(12.0, -16.0),
                            egui::Align2::LEFT_CENTER,
                            "⚠ Pista superpuesta",
                            egui::FontId::proportional(11.0),
                            Color32::from_rgb(239, 68, 68),
                        );
                    }
                }
            }
        }

        // Dibujar pequeña interfaz flotante de configuración de pista
        egui::Window::new("Ajustes de Vía")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::RIGHT_TOP, [-16.0, 64.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Carriles:");
                    ui.add(egui::Slider::new(&mut app.road_lanes, 1..=4));
                });
                ui.label(format!("Ancho total: {} metros", app.road_lanes * 3));
            });
    } else {
        app.road_draft = None;
    }
}
