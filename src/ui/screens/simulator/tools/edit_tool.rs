use egui::{Color32, Context, Painter, Rect, Response};
use crate::ui::screens::simulator::{EditTarget, SimuladorApp, Tool};
use crate::ui::screens::simulator::geom::{dist_to_segment, point_in_polygon};

pub fn handle_edit_tool(
    app: &mut SimuladorApp,
    _ctx: &Context,
    rect: Rect,
    response: &Response,
    painter: &Painter,
    pointer_world: Option<egui::Vec2>,
) {
    if app.selected_tool != Some(Tool::Edit) {
        app.selected_edit_target = None;
        return;
    }

    let scale = app.ui_zoom * app.text_scale;
    let zoom = app.viewport.zoom;

    // Obtener el paso magnético según el zoom actual de la ventana
    let step = if zoom <= 28.0 {
        10.0
    } else if zoom >= 200.0 {
        0.01
    } else if zoom >= 80.0 {
        0.1
    } else {
        1.0
    };

    // 1. Detectar clicks para seleccionar o iniciar el arrastre de vértices.
    if let Some(pw) = pointer_world {
        let screen_pos = app.viewport.world_to_screen(rect, pw);

        if response.drag_started() {
            // Buscar si estamos haciendo click-drag sobre un vértice de un elemento seleccionado
            let mut drag_started = false;

            if let Some(target) = app.selected_edit_target {
                match target {
                    EditTarget::Building(b_idx, _) => {
                        if b_idx < app.obstacles.len() {
                            let obstacle = &app.obstacles[b_idx];
                            for (v_idx, &vertex) in obstacle.iter().enumerate() {
                                let v_screen = app.viewport.world_to_screen(rect, vertex);
                                if screen_pos.distance(v_screen) < 15.0 * scale {
                                    app.selected_edit_target = Some(EditTarget::Building(b_idx, Some(v_idx)));
                                    drag_started = true;
                                    break;
                                }
                            }
                        }
                    }
                    EditTarget::Road(r_idx, _) => {
                        if r_idx < app.road_segments.len() {
                            let road = &app.road_segments[r_idx];
                            let from_screen = app.viewport.world_to_screen(rect, road.from);
                            let to_screen = app.viewport.world_to_screen(rect, road.to);

                            if screen_pos.distance(from_screen) < 15.0 * scale {
                                app.selected_edit_target = Some(EditTarget::Road(r_idx, Some(true)));
                                drag_started = true;
                            } else if screen_pos.distance(to_screen) < 15.0 * scale {
                                app.selected_edit_target = Some(EditTarget::Road(r_idx, Some(false)));
                                drag_started = true;
                            }
                        }
                    }
                }
            }

            // Si no se inició arrastre sobre un vértice seleccionado, probamos a seleccionar un nuevo objeto completo
            if !drag_started {
                let mut found = false;

                // Buscar si se clickeó dentro de un edificio
                for (idx, obstacle) in app.obstacles.iter().enumerate() {
                    if point_in_polygon(pw, obstacle) {
                        app.selected_edit_target = Some(EditTarget::Building(idx, None));
                        found = true;
                        break;
                    }
                }

                // Si no, buscar si es una carretera
                if !found {
                    for (idx, road) in app.road_segments.iter().enumerate() {
                        let dist = dist_to_segment(pw, road.from, road.to);
                        let width = road.lanes as f32 * 3.0;
                        if dist < (width / 2.0) {
                            app.selected_edit_target = Some(EditTarget::Road(idx, None));
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    app.selected_edit_target = None;
                }
            }
        }

        // 2. Si se está arrastrando, actualizar la posición del vértice seleccionado
        if response.dragged() {
            if let Some(target) = app.selected_edit_target {
                // El punto al que se arrastra magnetizado a la cuadrícula (según el paso magnético actual)
                let snapped_x = (pw.x / step).round() * step;
                let snapped_y = (pw.y / step).round() * step;
                let snapped_pos = egui::vec2(snapped_x, snapped_y);

                match target {
                    EditTarget::Building(b_idx, Some(v_idx)) => {
                        if b_idx < app.obstacles.len() {
                            let obstacle = &mut app.obstacles[b_idx];
                            if v_idx < obstacle.len() {
                                obstacle[v_idx] = snapped_pos;
                            }
                        }
                    }
                    EditTarget::Road(r_idx, Some(is_from)) => {
                        if r_idx < app.road_segments.len() {
                            let road = &mut app.road_segments[r_idx];
                            if is_from {
                                road.from = snapped_pos;
                            } else {
                                road.to = snapped_pos;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Si se suelta el arrastre, limpiamos el sub-estado de vértice que se está arrastrando (conservando la selección)
        if response.drag_stopped() {
            if let Some(target) = app.selected_edit_target {
                match target {
                    EditTarget::Building(b_idx, Some(_)) => {
                        app.selected_edit_target = Some(EditTarget::Building(b_idx, None));
                    }
                    EditTarget::Road(r_idx, Some(_)) => {
                        app.selected_edit_target = Some(EditTarget::Road(r_idx, None));
                    }
                    _ => {}
                }
            }
        }
    }

    // 3. Dibujar información visual del objeto seleccionado (Vértices y coordenadas del plano)
    if let Some(target) = app.selected_edit_target {
        let active_color = Color32::from_rgb(59, 130, 246); // Azul brillante para el modo edición
        
        match target {
            EditTarget::Building(b_idx, drag_v) => {
                if b_idx < app.obstacles.len() {
                    let obstacle = &app.obstacles[b_idx];
                    let n = obstacle.len();
                    if n >= 3 {
                        // Encontrar el vértice "más inferior a la izquierda"
                        let mut min_idx = 0;
                        for i in 1..n {
                            if obstacle[i].x < obstacle[min_idx].x {
                                min_idx = i;
                            } else if (obstacle[i].x - obstacle[min_idx].x).abs() < f32::EPSILON {
                                if obstacle[i].y < obstacle[min_idx].y {
                                    min_idx = i;
                                }
                            }
                        }

                        // Dibujar contorno de selección en azul
                        let points_screen: Vec<egui::Pos2> = obstacle.iter().map(|&pt| app.viewport.world_to_screen(rect, pt)).collect();
                        painter.add(egui::Shape::closed_line(
                            points_screen.clone(),
                            egui::Stroke::new(3.0 * scale, active_color),
                        ));

                        // Dibujar cada vértice y su etiqueta de coordenadas
                        for i in 0..n {
                            let pt = obstacle[i];
                            let pt_screen = points_screen[i];
                            
                            let is_dragging = drag_v == Some(i);
                            let is_pivot = i == min_idx;
                            
                            let (v_color, v_radius) = if is_dragging {
                                (Color32::from_rgb(249, 115, 22), 6.0 * scale)
                            } else if is_pivot {
                                (Color32::from_rgb(34, 197, 94), 6.5 * scale) // Verde para el eje principal
                            } else {
                                (active_color, 4.0 * scale)
                            };

                            painter.circle_filled(pt_screen, v_radius, v_color);

                            // Nombre del vértice (A, B, C...)
                            let vertex_name = get_vertex_label(i);
                            let text_vertex = if is_pivot {
                                format!("{}: ({:.1}, {:.1}) [Eje]", vertex_name, pt.x, pt.y)
                            } else {
                                format!("{}: ({:.1}, {:.1})", vertex_name, pt.x, pt.y)
                            };

                            let font_size_vertex = 9.0 * scale;
                            let text_size_vertex = painter.layout_no_wrap(
                                text_vertex.clone(),
                                egui::FontId::proportional(font_size_vertex),
                                Color32::WHITE,
                            ).rect.size();

                            // Calcular dirección exterior aproximada desde el centroide para colocar la etiqueta
                            let mut sum_x = 0.0;
                            let mut sum_y = 0.0;
                            for &v in obstacle {
                                sum_x += v.x;
                                sum_y += v.y;
                            }
                            let centroid = egui::vec2(sum_x / n as f32, sum_y / n as f32);
                            let dir_exterior = (pt - centroid).normalized();

                            let v_box_half_width = (text_size_vertex.x + 8.0 * scale) * 0.5;
                            let v_box_half_height = (text_size_vertex.y + 6.0 * scale) * 0.5;
                            let v_offset_dist = (dir_exterior.x.abs() * v_box_half_width + dir_exterior.y.abs() * v_box_half_height) + 6.0 * scale;
                            let vertex_label_center = pt_screen + dir_exterior * v_offset_dist;

                            painter.rect_filled(
                                Rect::from_center_size(vertex_label_center, text_size_vertex + egui::vec2(8.0, 6.0) * scale),
                                3.0 * scale,
                                Color32::from_black_alpha(240),
                            );
                            painter.text(
                                vertex_label_center,
                                egui::Align2::CENTER_CENTER,
                                text_vertex,
                                egui::FontId::proportional(font_size_vertex),
                                if is_pivot { Color32::from_rgb(74, 222, 128) } else { Color32::from_rgb(147, 197, 253) },
                            );
                        }
                    }
                }
            }
            EditTarget::Road(r_idx, drag_from) => {
                if r_idx < app.road_segments.len() {
                    let road = &app.road_segments[r_idx];
                    let a = road.from;
                    let b = road.to;
                    let width = road.lanes as f32 * 3.0;
                    let dir = (b - a).normalized();
                    let normal = egui::vec2(-dir.y, dir.x);
                    let offset = normal * (width / 2.0);

                    // Eje principal de la carretera: el punto inicial o el más inferior a la izquierda.
                    let is_a_pivot = if a.x < b.x {
                        true
                    } else if (a.x - b.x).abs() < f32::EPSILON {
                        a.y < b.y
                    } else {
                        false
                    };

                    // Dibujar el polígono de selección de la carretera
                    let road_poly = vec![
                        a + offset,
                        b + offset,
                        b - offset,
                        a - offset,
                    ];
                    let points_screen: Vec<egui::Pos2> = road_poly.iter().map(|&pt| app.viewport.world_to_screen(rect, pt)).collect();
                    painter.add(egui::Shape::closed_line(
                        points_screen,
                        egui::Stroke::new(3.0 * scale, active_color),
                    ));

                    // Dibujar extremos (ejes de arrastre de los extremos)
                    let a_screen = app.viewport.world_to_screen(rect, a);
                    let b_screen = app.viewport.world_to_screen(rect, b);

                    let drag_a = drag_from == Some(true);
                    let drag_b = drag_from == Some(false);

                    let color_a = if drag_a { Color32::from_rgb(249, 115, 22) } else if is_a_pivot { Color32::from_rgb(34, 197, 94) } else { active_color };
                    let color_b = if drag_b { Color32::from_rgb(249, 115, 22) } else if !is_a_pivot { Color32::from_rgb(34, 197, 94) } else { active_color };

                    painter.circle_filled(a_screen, 6.0 * scale, color_a);
                    painter.circle_filled(b_screen, 6.0 * scale, color_b);

                    // Coordenadas
                    let text_a = if is_a_pivot {
                        format!("Inicio: ({:.1}, {:.1}) [Eje]", a.x, a.y)
                    } else {
                        format!("Inicio: ({:.1}, {:.1})", a.x, a.y)
                    };
                    let text_b = if !is_a_pivot {
                        format!("Fin: ({:.1}, {:.1}) [Eje]", b.x, b.y)
                    } else {
                        format!("Fin: ({:.1}, {:.1})", b.x, b.y)
                    };

                    let font_size = 9.0 * scale;
                    
                    // Mostrar etiqueta de A
                    let size_a = painter.layout_no_wrap(text_a.clone(), egui::FontId::proportional(font_size), Color32::WHITE).rect.size();
                    let pos_label_a = a_screen - egui::vec2(0.0, 15.0 * scale);
                    painter.rect_filled(Rect::from_center_size(pos_label_a, size_a + egui::vec2(8.0, 6.0) * scale), 3.0 * scale, Color32::from_black_alpha(240));
                    painter.text(pos_label_a, egui::Align2::CENTER_CENTER, text_a, egui::FontId::proportional(font_size), if is_a_pivot { Color32::from_rgb(74, 222, 128) } else { Color32::from_rgb(147, 197, 253) });

                    // Mostrar etiqueta de B
                    let size_b = painter.layout_no_wrap(text_b.clone(), egui::FontId::proportional(font_size), Color32::WHITE).rect.size();
                    let pos_label_b = b_screen - egui::vec2(0.0, 15.0 * scale);
                    painter.rect_filled(Rect::from_center_size(pos_label_b, size_b + egui::vec2(8.0, 6.0) * scale), 3.0 * scale, Color32::from_black_alpha(240));
                    painter.text(pos_label_b, egui::Align2::CENTER_CENTER, text_b, egui::FontId::proportional(font_size), if !is_a_pivot { Color32::from_rgb(74, 222, 128) } else { Color32::from_rgb(147, 197, 253) });
                }
            }
        }
    }
}

fn get_vertex_label(mut index: usize) -> String {
    let mut label = String::new();
    loop {
        let remainder = index % 26;
        label.insert(0, (b'A' + remainder as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    label
}
