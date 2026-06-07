use egui::{Color32, Context, Painter, Rect, Response};
use crate::ui::screens::simulator::{DeleteMode, RoadSegmentGeometry, SimuladorApp, Tool};
use crate::ui::screens::simulator::geom::point_in_polygon;

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

    // Detección de colisiones (hovering) precisas con pistas y edificios según la escala de mundo.
    let mut hovered_road_idx = None;
    let mut hovered_road_subdivision = None; // (road_idx, sub_idx, p_from, p_to, total_steps)
    let mut hovered_building_idx = None;

    if let Some(pw) = pointer_world {
        // 1. Detectar colisión con carreteras. Para elementos diagonales o superpuestos, se proyecta el punto
        // sobre el segmento y se valida que la distancia ortogonal sea menor que la mitad del ancho de la pista,
        // y que la proyección caiga estrictamente dentro del segmento (0.0 <= proj_t <= 1.0).
        for (r_idx, road) in app.road_segments.iter().enumerate() {
            let ab = road.to - road.from;
            let length_sq = ab.length_sq();
            if length_sq == 0.0 { continue; }
            let ap = pw - road.from;
            let proj_t = ap.dot(ab) / length_sq;
            
            // Validamos estrictamente si el punto está dentro del tramo de la pista diagonal/recta
            if proj_t >= 0.0 && proj_t <= 1.0 {
                let projection = road.from + proj_t * ab;
                let dist = (pw - projection).length();
                let half_width = (road.lanes as f32 * 3.0) / 2.0;
                
                if dist < half_width {
                    hovered_road_idx = Some(r_idx);

                    // Si estamos en modo de subdivisión, calcular el segmento de 1m correspondiente
                    if app.delete_mode == DeleteMode::SubPolygon {
                        let length = ab.length();
                        let step_len = 1.0;
                        let num_steps = (length / step_len).round().max(1.0) as usize;
                        let current_step = (proj_t * num_steps as f32).floor() as usize;
                        let current_step = current_step.min(num_steps - 1);

                        let t1 = current_step as f32 / num_steps as f32;
                        let t2 = (current_step + 1) as f32 / num_steps as f32;
                        let sub_from = road.from + t1 * ab;
                        let sub_to = road.from + t2 * ab;

                        hovered_road_subdivision = Some((r_idx, current_step, sub_from, sub_to, num_steps));
                    }
                    break; // Paramos en la primera coincidencia válida
                }
            }
        }

        // 2. Detectar colisión con edificios.
        // Se considera colisión si el cursor está estrictamente dentro del polígono del edificio.
        for (idx, obstacle) in app.obstacles.iter().enumerate() {
            if point_in_polygon(pw, obstacle) {
                hovered_building_idx = Some(idx);
                break;
            }
        }
    }

    // Renderizar previsualizaciones (parpadeo en rojo, ciclo completo cada 1.5 segundos)
    let time = ctx.input(|i| i.time);
    let pulse_freq = 2.0 * std::f64::consts::PI / 1.5;
    let pulse = 0.35 + 0.15 * ((time * pulse_freq).sin() as f32);

    // Función auxiliar para dibujar un borde rotatorio con líneas trazadas (dashed/rayas) de 5.0 px de grosor.
    // Usamos el tiempo actual para desfasar el inicio del patrón de rayas y dar el efecto de movimiento/giro.
    let draw_rotating_dashed_border = |painter: &Painter, pts: &[egui::Pos2], time: f64| {
        let stroke_width = 5.0;
        let stroke_color = Color32::from_rgb(239, 68, 68);
        
        let dash_len = 10.0f32;
        let gap_len = 8.0f32;
        let pattern_len = dash_len + gap_len;
        
        // El desplazamiento (offset) de fase depende del tiempo para animar el movimiento de rotación a lo largo del borde
        let speed = 40.0; // píxeles por segundo
        let offset = (time * speed) % pattern_len as f64;

        for i in 0..pts.len() {
            let p1 = pts[i];
            let p2 = pts[(i + 1) % pts.len()];
            let dir = p2 - p1;
            let len = dir.length();
            if len == 0.0 { continue; }
            let dir_normalized = dir / len;

            let mut current_dist = -offset as f32;
            while current_dist < len {
                let start_t = current_dist.max(0.0);
                let end_t = (current_dist + dash_len).min(len);
                
                if end_t > start_t {
                    let start_point = p1 + dir_normalized * start_t;
                    let end_point = p1 + dir_normalized * end_t;
                    painter.line_segment([start_point, end_point], egui::Stroke::new(stroke_width, stroke_color));
                }
                current_dist += pattern_len;
            }
        }
    };

    match app.delete_mode {
        DeleteMode::SubPolygon => {
            // Previsualización de pista (cara/sub-polígono)
            if app.delete_roads {
                if let Some((_, _, sub_from, sub_to, _)) = hovered_road_subdivision {
                    let road = &app.road_segments[hovered_road_subdivision.unwrap().0];
                    let width = road.lanes as f32 * 3.0;
                    let dir = (sub_to - sub_from).normalized();
                    let normal = egui::vec2(-dir.y, dir.x);
                    let offset_vec = normal * (width / 2.0);

                    let pts = [
                        app.viewport.world_to_screen(rect, sub_from + offset_vec),
                        app.viewport.world_to_screen(rect, sub_to + offset_vec),
                        app.viewport.world_to_screen(rect, sub_to - offset_vec),
                        app.viewport.world_to_screen(rect, sub_from - offset_vec),
                    ];

                    painter.add(egui::Shape::convex_polygon(
                        pts.to_vec(),
                        Color32::from_rgba_unmultiplied(239, 68, 68, (pulse * 255.0) as u8),
                        egui::Stroke::NONE,
                    ));
                    draw_rotating_dashed_border(painter, &pts, time);
                }
            }
            // Previsualización de edificio (un edificio no tiene subdivisión, se parpadea completo)
            if app.delete_buildings {
                if let Some(idx) = hovered_building_idx {
                    let pts: Vec<egui::Pos2> = app.obstacles[idx].iter().map(|&pt| app.viewport.world_to_screen(rect, pt)).collect();
                    painter.add(egui::Shape::convex_polygon(
                        pts.clone(),
                        Color32::from_rgba_unmultiplied(239, 68, 68, (pulse * 100.0) as u8),
                        egui::Stroke::NONE,
                    ));
                    draw_rotating_dashed_border(painter, &pts, time);
                }
            }
        }
        DeleteMode::FullElement => {
            // Parpadea todo el elemento (pista o edificio completo) bajo el cursor
            if app.delete_buildings && hovered_building_idx.is_some() {
                let idx = hovered_building_idx.unwrap();
                let pts: Vec<egui::Pos2> = app.obstacles[idx].iter().map(|&pt| app.viewport.world_to_screen(rect, pt)).collect();
                painter.add(egui::Shape::convex_polygon(
                    pts.clone(),
                    Color32::from_rgba_unmultiplied(239, 68, 68, (pulse * 100.0) as u8),
                    egui::Stroke::NONE,
                ));
                draw_rotating_dashed_border(painter, &pts, time);
            } else if app.delete_roads && hovered_road_idx.is_some() {
                let road = &app.road_segments[hovered_road_idx.unwrap()];
                let width = road.lanes as f32 * 3.0;
                let dir = (road.to - road.from).normalized();
                let normal = egui::vec2(-dir.y, dir.x);
                let offset_vec = normal * (width / 2.0);

                let pts = [
                    app.viewport.world_to_screen(rect, road.from + offset_vec),
                    app.viewport.world_to_screen(rect, road.to + offset_vec),
                    app.viewport.world_to_screen(rect, road.to - offset_vec),
                    app.viewport.world_to_screen(rect, road.from - offset_vec),
                ];

                painter.add(egui::Shape::convex_polygon(
                    pts.to_vec(),
                    Color32::from_rgba_unmultiplied(239, 68, 68, (pulse * 100.0) as u8),
                    egui::Stroke::NONE,
                ));
                draw_rotating_dashed_border(painter, &pts, time);
            }
        }
        _ => {}
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
        if response.clicked_by(egui::PointerButton::Primary) {
            match app.delete_mode {
                DeleteMode::SubPolygon => {
                    // Borrar edificio completo (un edificio no tiene sub-caras)
                    if app.delete_buildings && hovered_building_idx.is_some() {
                        app.obstacles.remove(hovered_building_idx.unwrap());
                    } else if app.delete_roads && hovered_road_subdivision.is_some() {
                        let (r_idx, _, sub_from, sub_to, _) = hovered_road_subdivision.unwrap();
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
                DeleteMode::Lasso => {
                    if let Some(pw) = pointer_world {
                        let snapped_x = (pw.x / step).round() * step;
                        let snapped_y = (pw.y / step).round() * step;
                        let snapped_pos = egui::vec2(snapped_x, snapped_y);
                        app.delete_lasso_points.push(snapped_pos);
                    }
                }
                DeleteMode::FullElement => {
                    if app.delete_buildings && hovered_building_idx.is_some() {
                        app.obstacles.remove(hovered_building_idx.unwrap());
                    } else if app.delete_roads && hovered_road_idx.is_some() {
                        let road_id = app.road_segments[hovered_road_idx.unwrap()].id;
                        app.road_segments.retain(|road| road.id != road_id);
                    }
                }
            }
        } else if response.clicked_by(egui::PointerButton::Secondary) && app.delete_mode == DeleteMode::Lasso && !app.delete_lasso_points.is_empty() {
            lasso_triggered = true;
        }
    }

    // Procesar lazo si ha sido disparado por click o interfaz
    if lasso_triggered && !app.delete_lasso_points.is_empty() {
        if app.delete_buildings {
            use clipper2_rust::{difference_d, make_path_d, core::FillRule, PathsD};

            let mut new_obstacles = Vec::new();
            
            // Construir el lazo en formato de clipper2_rust (PathsD)
            let mut clip_path = Vec::new();
            for pt in &app.delete_lasso_points {
                clip_path.push(pt.x as f64);
                clip_path.push(pt.y as f64);
            }
            let clip_paths: PathsD = vec![make_path_d(&clip_path)];

            for obs in &app.obstacles {
                let mut subj_path = Vec::new();
                for pt in obs {
                    subj_path.push(pt.x as f64);
                    subj_path.push(pt.y as f64);
                }
                let subj_paths: PathsD = vec![make_path_d(&subj_path)];

                // Restar el lazo de selección al polígono del edificio, usando precisión decimal 2 (por ejemplo, 2 decimales)
                let diff_paths = difference_d(&subj_paths, &clip_paths, FillRule::EvenOdd, 2);

                // Convertir los polígonos resultantes de vuelta al formato del simulador (Vec<egui::Vec2>)
                for path in diff_paths {
                    if path.len() >= 3 {
                        let mut new_obs = Vec::with_capacity(path.len());
                        for pt in path {
                            new_obs.push(egui::vec2(pt.x as f32, pt.y as f32));
                        }
                        new_obstacles.push(new_obs);
                    }
                }
            }

            app.obstacles = new_obstacles;
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
                        use clipper2_rust::{difference_d, make_path_d, core::FillRule, PathsD};

                        let mut new_obstacles = Vec::new();
                        
                        let mut clip_path = Vec::new();
                        for pt in &app.delete_lasso_points {
                            clip_path.push(pt.x as f64);
                            clip_path.push(pt.y as f64);
                        }
                        let clip_paths: PathsD = vec![make_path_d(&clip_path)];

                        for obs in &app.obstacles {
                            let mut subj_path = Vec::new();
                            for pt in obs {
                                subj_path.push(pt.x as f64);
                                subj_path.push(pt.y as f64);
                            }
                            let subj_paths: PathsD = vec![make_path_d(&subj_path)];

                            // Restar con precisión decimal 2
                            let diff_paths = difference_d(&subj_paths, &clip_paths, FillRule::EvenOdd, 2);

                            for path in diff_paths {
                                if path.len() >= 3 {
                                    let mut new_obs = Vec::with_capacity(path.len());
                                    for pt in path {
                                        new_obs.push(egui::vec2(pt.x as f32, pt.y as f32));
                                    }
                                    new_obstacles.push(new_obs);
                                }
                            }
                        }

                        app.obstacles = new_obstacles;
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
