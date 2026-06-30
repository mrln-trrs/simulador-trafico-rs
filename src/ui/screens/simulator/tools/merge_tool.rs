use egui::{Color32, Context, Painter, Rect, Response};
use crate::ui::screens::simulator::{SimuladorApp, Tool, RoadSegmentGeometry};
use crate::ui::screens::simulator::geom::{dist_to_segment, point_in_polygon};

/// Tipo de selección para el merge
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MergeSelection {
    Road(usize),    // índice en road_segments
    Building(usize), // índice en obstacles
}

/// Estado persistente del merge tool
#[derive(Debug, Default)]
pub struct MergeToolState {
    pub first_selection: Option<MergeSelection>,
    pub second_selection: Option<MergeSelection>,
    pub confirm_pending: bool,
    pub error_msg: Option<String>,
    pub error_timer: f64,
}

pub fn handle_merge_tool(
    app: &mut SimuladorApp,
    ctx: &Context,
    rect: Rect,
    response: &Response,
    painter: &Painter,
    pointer_world: Option<egui::Vec2>,
) {
    if app.selected_tool != Some(Tool::Merge) {
        app.merge_state.first_selection = None;
        app.merge_state.second_selection = None;
        app.merge_state.confirm_pending = false;
        app.merge_state.error_msg = None;
        return;
    }

    let time = ctx.input(|i| i.time);

    // Limpiar error timer
    if app.merge_state.error_msg.is_some() && time > app.merge_state.error_timer {
        app.merge_state.error_msg = None;
    }

    // ── Clic para seleccionar elementos ──
    if !app.merge_state.confirm_pending {
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(p_world) = pointer_world {
                let hit = find_element_at(p_world, app);

                if let Some(hit_sel) = hit {
                    match app.merge_state.first_selection {
                        None => {
                            // Primera selección: cualquier elemento
                            app.merge_state.first_selection = Some(hit_sel);
                        }
                        Some(first) => {
                            // Segunda selección: debe ser del mismo tipo, y diferente
                            let same_type = match (first, hit_sel) {
                                (MergeSelection::Road(_), MergeSelection::Road(_)) => true,
                                (MergeSelection::Building(_), MergeSelection::Building(_)) => true,
                                _ => false,
                            };
                            let different = first != hit_sel;

                            if same_type && different {
                                app.merge_state.second_selection = Some(hit_sel);
                                app.merge_state.confirm_pending = true;
                            } else if !same_type {
                                set_error(&mut app.merge_state, time, "Solo puedes unir elementos del mismo tipo (pista+pista o edificio+edificio).");
                            } else {
                                // mismo elemento, reiniciar
                                app.merge_state.first_selection = Some(hit_sel);
                            }
                        }
                    }
                } else {
                    // Clic en vacío: reiniciar selección
                    app.merge_state.first_selection = None;
                    app.merge_state.second_selection = None;
                }
            }
        }
    }

    // ── Diálogo de confirmación ──
    if app.merge_state.confirm_pending {
        let (first, second) = match (app.merge_state.first_selection, app.merge_state.second_selection) {
            (Some(f), Some(s)) => (f, s),
            _ => {
                app.merge_state.confirm_pending = false;
                return;
            }
        };

        let type_label = match first {
            MergeSelection::Road(_) => "pistas",
            MergeSelection::Building(_) => "edificios",
        };

        let mut confirmed = false;
        let mut cancelled = false;

        egui::Window::new("Confirmar unión")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(format!("¿Deseas unir estos dos {}?", type_label));
                    ui.label("Esta operación no se puede deshacer.");
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button("✔ Confirmar").clicked() {
                            confirmed = true;
                        }
                        if ui.button("✖ Cancelar").clicked() {
                            cancelled = true;
                        }
                    });
                    ui.add_space(8.0);
                });
            });

        if confirmed {
            let result = perform_merge(app, first, second);
            if let Err(msg) = result {
                set_error(&mut app.merge_state, time, &msg);
            }
            app.merge_state.first_selection = None;
            app.merge_state.second_selection = None;
            app.merge_state.confirm_pending = false;
        } else if cancelled {
            app.merge_state.second_selection = None;
            app.merge_state.confirm_pending = false;
        }
    }

    // ── Resaltar elementos seleccionados ──
    draw_selection_highlight(app, painter, rect);

    // ── Mostrar mensaje de error ──
    if let Some(msg) = &app.merge_state.error_msg.clone() {
        egui::Window::new("⚠ No se puede unir")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 80.0])
            .show(ctx, |ui| {
                ui.label(msg.as_str());
            });
    }

    // ── Instrucción en pantalla ──
    egui::Window::new("Herramienta Unir")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_TOP, [-16.0, 64.0])
        .show(ctx, |ui| {
            match app.merge_state.first_selection {
                None => { ui.label("Clic en el primer elemento a unir."); }
                Some(_) => { ui.label("Clic en el segundo elemento del mismo tipo."); }
            }
        });
}

/// Busca el elemento en el punto de mundo dado
fn find_element_at(p_world: egui::Vec2, app: &SimuladorApp) -> Option<MergeSelection> {
    // Buscar edificio primero
    for (idx, building) in app.obstacles.iter().enumerate() {
        if point_in_polygon(p_world, building) {
            return Some(MergeSelection::Building(idx));
        }
    }
    // Buscar pista
    for (idx, road) in app.road_segments.iter().enumerate() {
        let dist = dist_to_segment(p_world, road.from, road.to);
        let half_w = (road.lanes as f32 * 3.0) / 2.0;
        if dist < half_w {
            return Some(MergeSelection::Road(idx));
        }
    }
    None
}

/// Resaltar con colores los elementos seleccionados
fn draw_selection_highlight(app: &SimuladorApp, painter: &Painter, rect: Rect) {
    let color1 = Color32::from_rgb(52, 211, 153);  // verde para primera sel.
    let color2 = Color32::from_rgb(251, 146, 60);  // naranja para segunda sel.

    for (sel, color) in [
        (app.merge_state.first_selection, color1),
        (app.merge_state.second_selection, color2),
    ] {
        if let Some(sel) = sel {
            match sel {
                MergeSelection::Building(idx) => {
                    if idx < app.obstacles.len() && app.obstacles[idx].len() >= 3 {
                        let pts: Vec<egui::Pos2> = app.obstacles[idx]
                            .iter()
                            .map(|&v| app.viewport.world_to_screen(rect, v))
                            .collect();
                        painter.add(egui::Shape::closed_line(
                            pts,
                            egui::Stroke::new(3.0, color),
                        ));
                    }
                }
                MergeSelection::Road(idx) => {
                    if idx < app.road_segments.len() {
                        let road = &app.road_segments[idx];
                        let width = road.lanes as f32 * 3.0;
                        let dir = (road.to - road.from).normalized();
                        let normal = egui::vec2(-dir.y, dir.x);
                        let off = normal * (width / 2.0);
                        let pts = [
                            app.viewport.world_to_screen(rect, road.from + off),
                            app.viewport.world_to_screen(rect, road.to + off),
                            app.viewport.world_to_screen(rect, road.to - off),
                            app.viewport.world_to_screen(rect, road.from - off),
                        ];
                        painter.add(egui::Shape::closed_line(
                            pts.to_vec(),
                            egui::Stroke::new(3.0, color),
                        ));
                    }
                }
            }
        }
    }
}

/// Realiza la fusión de los dos elementos seleccionados
fn perform_merge(app: &mut SimuladorApp, first: MergeSelection, second: MergeSelection) -> Result<(), String> {
    match (first, second) {
        (MergeSelection::Road(a_idx), MergeSelection::Road(b_idx)) => {
            merge_roads(app, a_idx, b_idx)
        }
        (MergeSelection::Building(a_idx), MergeSelection::Building(b_idx)) => {
            merge_buildings(app, a_idx, b_idx)
        }
        _ => Err("Tipos incompatibles".to_string()),
    }
}

/// Fusión de pistas: conecta los extremos más lejanos de los dos segmentos,
/// solo si son contiguas (comparten un extremo cercano).
fn merge_roads(app: &mut SimuladorApp, a_idx: usize, b_idx: usize) -> Result<(), String> {
    if a_idx >= app.road_segments.len() || b_idx >= app.road_segments.len() {
        return Err("Índice de pista inválido.".to_string());
    }

    let a = app.road_segments[a_idx].clone();
    let b = app.road_segments[b_idx].clone();

    // Deben tener el mismo número de carriles para fusionarse
    if a.lanes != b.lanes {
        return Err(format!(
            "Las pistas deben tener el mismo número de carriles ({} vs {}).",
            a.lanes, b.lanes
        ));
    }

    // Buscar los extremos contiguos (que se toquen dentro de un radio de tolerancia)
    let tolerance = a.lanes as f32 * 3.0 * 0.6; // 60% del ancho de carril
    let pairs = [
        (a.from, a.to, b.from, b.to), // a_from -- a_to -- b_from -- b_to
        (a.from, a.to, b.to, b.from),
        (a.to, a.from, b.from, b.to),
        (a.to, a.from, b.to, b.from),
    ];

    let mut best_start: Option<egui::Vec2> = None;
    let mut best_end: Option<egui::Vec2> = None;
    let mut best_gap = f32::MAX;

    for (start_a, end_a, start_b, end_b) in &pairs {
        let gap = (*end_a - *start_b).length();
        if gap < tolerance && gap < best_gap {
            best_gap = gap;
            best_start = Some(*start_a);
            best_end = Some(*end_b);
        }
    }

    let (new_from, new_to) = match (best_start, best_end) {
        (Some(f), Some(t)) => (f, t),
        _ => return Err("Las pistas no son contiguas. Solo se pueden unir pistas que se toquen.".to_string()),
    };

    let new_lanes = a.lanes;
    let new_id = app.next_road_id;
    app.next_road_id += 1;

    // Validar que el nuevo segmento fusionado no colisione con edificios
    let width = new_lanes as f32 * 3.0;
    if crate::ui::screens::simulator::geom::road_collides_with_obstacles(new_from, new_to, width, &app.obstacles) {
        return Err("La pista fusionada colisionaría con un edificio.".to_string());
    }

    // Eliminar los dos segmentos originales (del mayor índice al menor para no desplazar)
    let (lo, hi) = if a_idx < b_idx { (a_idx, b_idx) } else { (b_idx, a_idx) };
    app.road_segments.remove(hi);
    app.road_segments.remove(lo);

    // Insertar el nuevo segmento fusionado
    app.road_segments.push(RoadSegmentGeometry {
        from: new_from,
        to: new_to,
        lanes: new_lanes,
        id: new_id,
    });

    Ok(())
}

/// Fusión de edificios: calcula la envolvente convexa de todos los vértices combinados.
fn merge_buildings(app: &mut SimuladorApp, a_idx: usize, b_idx: usize) -> Result<(), String> {
    if a_idx >= app.obstacles.len() || b_idx >= app.obstacles.len() {
        return Err("Índice de edificio inválido.".to_string());
    }

    let mut all_points: Vec<egui::Vec2> = Vec::new();
    all_points.extend_from_slice(&app.obstacles[a_idx]);
    all_points.extend_from_slice(&app.obstacles[b_idx]);

    // Calcular envolvente convexa (algoritmo Graham Scan simplificado — Andrew's Monotone Chain)
    let hull = convex_hull(&all_points);
    if hull.len() < 3 {
        return Err("No se pudo calcular la envolvente convexa.".to_string());
    }

    // Validar que el edificio resultante no colisione con carreteras
    if crate::ui::screens::simulator::geom::building_collides_with_roads(&hull, &app.road_segments) {
        return Err("El edificio fusionado colisionaría con una pista.".to_string());
    }

    // Eliminar los dos originales (del mayor al menor)
    let (lo, hi) = if a_idx < b_idx { (a_idx, b_idx) } else { (b_idx, a_idx) };
    app.obstacles.remove(hi);
    app.obstacles.remove(lo);

    // Insertar el nuevo polígono combinado
    app.obstacles.push(hull);

    Ok(())
}

/// Algoritmo Andrew's Monotone Chain para envolvente convexa.
fn convex_hull(points: &[egui::Vec2]) -> Vec<egui::Vec2> {
    let mut pts: Vec<egui::Vec2> = points.to_vec();
    pts.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap().then(a.y.partial_cmp(&b.y).unwrap()));
    pts.dedup_by(|a, b| (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6);

    if pts.len() < 3 { return pts; }

    let cross = |o: egui::Vec2, a: egui::Vec2, b: egui::Vec2| -> f32 {
        (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
    };

    let mut lower: Vec<egui::Vec2> = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len()-2], lower[lower.len()-1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }

    let mut upper: Vec<egui::Vec2> = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len()-2], upper[upper.len()-1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }

    lower.pop();
    upper.pop();
    lower.extend_from_slice(&upper);
    lower
}

fn set_error(state: &mut MergeToolState, time: f64, msg: &str) {
    state.error_msg = Some(msg.to_string());
    state.error_timer = time + 3.5;
}
