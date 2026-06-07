use egui::{Color32, Context, Painter, Rect, Response};
use crate::ui::screens::simulator::{DeleteMode, RoadSegmentGeometry, SimuladorApp, Tool};
use crate::ui::screens::simulator::geom::{dist_to_segment, point_in_polygon};

pub fn handle_delete_tool(
    app: &mut SimuladorApp,
    ctx: &Context,
    rect: Rect,
    response: &Response,
    painter: &Painter,
    pointer_world: Option<egui::Vec2>,
    step: f32,
) {
    if app.selected_tool != Some(Tool::Delete) {
        app.delete_lasso_points.clear();
        return;
    }

    // Previsualización y lógica de selección del sub-polígono (cara) actual si el borrador por sub-polígono está activo
    let mut hovered_road_subdivision = None; // (road_idx, sub_idx, p_from, p_to, total_steps)
    if app.delete_mode == DeleteMode::SubPolygon {
        if let Some(pw) = pointer_world {
            for (r_idx, road) in app.road_segments.iter().enumerate() {
                let dist = dist_to_segment(pw, road.from, road.to);
                let width = road.lanes as f32 * 3.0;
                if dist < (width / 2.0) {
                    // Subdividir en celdas de 1m dinámicamente (on-the-fly) para calcular la celda apuntada
                    let ab = road.to - road.from;
                    let length = ab.length();
                    let step_len = 1.0;
                    let num_steps = (length / step_len).round().max(1.0) as usize;

                    let ap = pw - road.from;
                    let proj_t = (ap.dot(ab) / ab.length_sq()).clamp(0.0, 1.0);
                    let current_step = (proj_t * num_steps as f32).floor() as usize;
                    let current_step = current_step.min(num_steps - 1);

                    let t1 = current_step as f32 / num_steps as f32;
                    let t2 = (current_step + 1) as f32 / num_steps as f32;
                    let sub_from = road.from + t1 * ab;
                    let sub_to = road.from + t2 * ab;

                    hovered_road_subdivision = Some((r_idx, current_step, sub_from, sub_to, num_steps));
                    break;
                }
            }
        }

        // Renderizar el sub-polígono apuntado parpadeando en rojo (solo si se permite eliminar o previsualizar pistas)
        if app.delete_roads {
            if let Some((_, _, sub_from, sub_to, _)) = hovered_road_subdivision {
                let road = &app.road_segments[hovered_road_subdivision.unwrap().0];
                let width = road.lanes as f32 * 3.0;
                let dir = (sub_to - sub_from).normalized();
                let normal = egui::vec2(-dir.y, dir.x);
                let offset = normal * (width / 2.0);

                let pts = [
                    app.viewport.world_to_screen(rect, sub_from + offset),
                    app.viewport.world_to_screen(rect, sub_to + offset),
                    app.viewport.world_to_screen(rect, sub_to - offset),
                    app.viewport.world_to_screen(rect, sub_from - offset),
                ];

                let time = ctx.input(|i| i.time);
                let pulse = 0.35 + 0.15 * ((time * std::f64::consts::PI * 4.0).sin() as f32);
                painter.add(egui::Shape::convex_polygon(
                    pts.to_vec(),
                    Color32::from_rgba_unmultiplied(239, 68, 68, (pulse * 255.0) as u8),
                    egui::Stroke::new(1.5, Color32::from_rgb(239, 68, 68)),
                ));
            }
        }
    }

    // Previsualización de edificios que se eliminarían si el cursor está sobre un vértice (solo si delete_buildings está activo o para mostrar visualmente)
    if app.delete_buildings {
        if let Some(click_pos) = ctx.input(|i| i.pointer.hover_pos()) {
            if rect.contains(click_pos) {
                let mut hovered_building_idx = None;
                for (idx, obstacle) in app.obstacles.iter().enumerate() {
                    for &vertex in obstacle {
                        let screen_vertex = app.viewport.world_to_screen(rect, vertex);
                        if screen_vertex.distance(click_pos) < 15.0 {
                            hovered_building_idx = Some(idx);
                            break;
                        }
                    }
                    if hovered_building_idx.is_some() { break; }
                }

                if let Some(idx) = hovered_building_idx {
                    let pts: Vec<egui::Pos2> = app.obstacles[idx].iter().map(|&pt| app.viewport.world_to_screen(rect, pt)).collect();
                    let time = ctx.input(|i| i.time);
                    let pulse = 0.35 + 0.15 * ((time * std::f64::consts::PI * 4.0).sin() as f32);
                    painter.add(egui::Shape::closed_line(
                        pts,
                        egui::Stroke::new(2.5, Color32::from_rgba_unmultiplied(239, 68, 68, (pulse * 255.0) as u8)),
                    ));
                }
            }
        }
    }

    // Dibujar interfaz y lógica del lazo de borrado
    let mut lasso_triggered = false;
    if app.delete_mode == DeleteMode::Lasso {
        // Dibujar lazo existente
        if !app.delete_lasso_points.is_empty() {
            let pts: Vec<egui::Pos2> = app.delete_lasso_points.iter().map(|&pt| app.viewport.world_to_screen(rect, pt)).collect();
            for i in 0..(pts.len() - 1) {
                painter.line_segment([pts[i], pts[i+1]], egui::Stroke::new(1.5, Color32::from_rgb(239, 68, 68)));
            }
            if let Some(snapped_screen) = pointer_world.map(|pw| {
                let snapped_x = (pw.x / step).round() * step;
                let snapped_y = (pw.y / step).round() * step;
                app.viewport.world_to_screen(rect, egui::vec2(snapped_x, snapped_y))
            }) {
                painter.line_segment([*pts.last().unwrap(), snapped_screen], egui::Stroke::new(1.0, Color32::from_rgb(156, 163, 175)));
                painter.circle_filled(snapped_screen, 4.0, Color32::from_rgb(239, 68, 68));
            }
        }
    }

    // Lógica interactiva para borrar en base a los 3 modos (Solo si delete_roads o delete_buildings están activos)
    if app.delete_roads || app.delete_buildings {
        if let Some(click_pos) = response.interact_pointer_pos() {
            let click_world = app.viewport.screen_to_world(rect, click_pos);

            match app.delete_mode {
                DeleteMode::SubPolygon => {
                    if response.clicked_by(egui::PointerButton::Primary) {
                        // 1. Borrar edificio si hacemos clic cerca de algún vértice (solo si delete_buildings)
                        let mut obstacle_to_remove = None;
                        if app.delete_buildings {
                            for (idx, obstacle) in app.obstacles.iter().enumerate() {
                                for &vertex in obstacle {
                                    let screen_vertex = app.viewport.world_to_screen(rect, vertex);
                                    if screen_vertex.distance(click_pos) < 15.0 {
                                        obstacle_to_remove = Some(idx);
                                        break;
                                    }
                                }
                                if obstacle_to_remove.is_some() { break; }
                            }
                        }

                        if let Some(idx) = obstacle_to_remove {
                            app.obstacles.remove(idx);
                        } else if app.delete_roads {
                            if let Some((r_idx, _, sub_from, sub_to, _)) = hovered_road_subdivision {
                                // 2. Borrar sub-polígono dinámico: Divide la carretera en 2 nuevos macro-segmentos con IDs frescas
                                let old_road = app.road_segments.remove(r_idx);

                                // Fragmento izquierdo (desde inicio hasta el inicio del subsegmento borrado)
                                if (sub_from - old_road.from).length() > 0.1 {
                                    let road_id = app.next_road_id;
                                    app.next_road_id += 1;
                                    app.road_segments.push(RoadSegmentGeometry {
                                        from: old_road.from,
                                        to: sub_from,
                                        lanes: old_road.lanes,
                                        id: road_id,
                                    });
                                }

                                // Fragmento derecho (desde fin del subsegmento borrado hasta el final)
                                if (old_road.to - sub_to).length() > 0.1 {
                                    let road_id = app.next_road_id;
                                    app.next_road_id += 1;
                                    app.road_segments.push(RoadSegmentGeometry {
                                        from: sub_to,
                                        to: old_road.to,
                                        lanes: old_road.lanes,
                                        id: road_id,
                                    });
                                }
                            }
                        }
                    }
                }
                DeleteMode::Lasso => {
                    if let Some(pw) = pointer_world {
                        let snapped_x = (pw.x / step).round() * step;
                        let snapped_y = (pw.y / step).round() * step;
                        let snapped_pos = egui::vec2(snapped_x, snapped_y);

                        // Agregar punto de lazo con click izquierdo
                        if response.clicked_by(egui::PointerButton::Primary) {
                            app.delete_lasso_points.push(snapped_pos);
                        }

                        // Aplicar lazo con click derecho
                        if response.clicked_by(egui::PointerButton::Secondary) && !app.delete_lasso_points.is_empty() {
                            lasso_triggered = true;
                        }
                    }
                }
                DeleteMode::FullElement => {
                    if response.clicked_by(egui::PointerButton::Primary) {
                        // 1. Borrar edificio entero
                        let mut obstacle_to_remove = None;
                        if app.delete_buildings {
                            for (idx, obstacle) in app.obstacles.iter().enumerate() {
                                for &vertex in obstacle {
                                    let screen_vertex = app.viewport.world_to_screen(rect, vertex);
                                    if screen_vertex.distance(click_pos) < 15.0 {
                                        obstacle_to_remove = Some(idx);
                                        break;
                                    }
                                }
                                if obstacle_to_remove.is_some() { break; }
                            }
                        }

                        if let Some(idx) = obstacle_to_remove {
                            app.obstacles.remove(idx);
                        } else if app.delete_roads {
                            // 2. Borrar segmento de carretera completo (toda la ID asociada)
                            let mut road_id_to_remove = None;
                            for road in &app.road_segments {
                                let dist = dist_to_segment(click_world, road.from, road.to);
                                let width = road.lanes as f32 * 3.0;
                                let screen_dist = dist * app.viewport.zoom;
                                if screen_dist < (width * app.viewport.zoom / 2.0 + 10.0) {
                                    road_id_to_remove = Some(road.id);
                                    break;
                                }
                            }
                            if let Some(id) = road_id_to_remove {
                                app.road_segments.retain(|road| road.id != id);
                            }
                        }
                    }
                }
            }
        }
    }

    // Procesar lazo si ha sido disparado por click o interfaz
    if lasso_triggered && !app.delete_lasso_points.is_empty() {
        if app.delete_buildings {
            // Eliminar edificios que tengan algún vértice dentro del lazo
            app.obstacles.retain(|obs| {
                !obs.iter().any(|&vertex| point_in_polygon(vertex, &app.delete_lasso_points))
            });
        }

        if app.delete_roads {
            // Eliminar segmentos de carreteras cuyos centros queden dentro de la selección
            app.road_segments.retain(|road| {
                let center = (road.from + road.to) / 2.0;
                !point_in_polygon(center, &app.delete_lasso_points)
            });
        }

        app.delete_lasso_points.clear();
    }

    // Dibujar interfaz flotante para elegir el modo de borrado
    egui::Window::new("Ajustes de Borrado")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_TOP, [-16.0, 64.0])
        .show(ctx, |ui| {
            crate::ui::screens::simulator::windows::settings_window::apply_local_scale(ui, app.ui_zoom * app.text_scale);
            
            ui.label("Tipo de Eliminación:");
            ui.horizontal(|ui| {
                ui.checkbox(&mut app.delete_roads, "🛣 Pistas");
                ui.checkbox(&mut app.delete_buildings, "🏢 Edificios");
            });
            ui.separator();
            
            ui.label("Modo de Borrado:");
            ui.selectable_value(&mut app.delete_mode, DeleteMode::SubPolygon, "1. Caras / Sub-polígonos");
            ui.selectable_value(&mut app.delete_mode, DeleteMode::Lasso, "2. Lazo Poligonal");
            ui.selectable_value(&mut app.delete_mode, DeleteMode::FullElement, "3. Elemento Completo");
            
            if app.delete_mode == DeleteMode::Lasso && !app.delete_lasso_points.is_empty() {
                ui.separator();
                if ui.button("Confirmar Lazo (Click Derecho)").clicked() {
                    if app.delete_buildings {
                        // Eliminar edificios que tengan algún vértice dentro del lazo
                        app.obstacles.retain(|obs| {
                            !obs.iter().any(|&vertex| point_in_polygon(vertex, &app.delete_lasso_points))
                        });
                    }

                    if app.delete_roads {
                        // Eliminar segmentos de carreteras cuyos centros queden dentro de la selección
                        app.road_segments.retain(|road| {
                            let center = (road.from + road.to) / 2.0;
                            !point_in_polygon(center, &app.delete_lasso_points)
                        });
                    }

                    app.delete_lasso_points.clear();
                }
            }
        });
}
