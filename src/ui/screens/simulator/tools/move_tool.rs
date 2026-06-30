use egui::{Color32, Context, Painter, Rect, Response};
use crate::ui::screens::simulator::{SimuladorApp, Tool};
use crate::ui::screens::simulator::geom::{
    dist_to_segment, point_in_polygon,
    road_collides_with_roads, building_collides_with_roads,
    snap_to_elements,
};

/// Elemento siendo arrastrado
#[derive(Debug, Clone, Copy)]
pub enum DragTarget {
    /// Índice en `road_segments` — arrastramos UNO o todos los del mismo `id` de grupo
    Road(usize),
    /// Índice en `obstacles`
    Building(usize),
}

/// Estado persistente del tool de movimiento entre frames
#[derive(Debug, Default)]
pub struct MoveToolState {
    pub drag_target: Option<DragTarget>,
    /// Copia de seguridad del elemento antes de empezar el drag (para revertir si hay colisión)
    pub original_road_from: Option<egui::Vec2>,
    pub original_road_to: Option<egui::Vec2>,
    pub original_building: Option<Vec<egui::Vec2>>,
    /// Posición en mundo donde se hizo el mouse-down
    pub drag_start_world: Option<egui::Vec2>,
    /// Índice del elemento resaltado bajo el cursor
    pub hovered_road: Option<usize>,
    pub hovered_building: Option<usize>,
    /// Error de colisión al soltar
    pub collision_error: bool,
    pub error_timer: f64,
}

pub fn handle_move_tool(
    app: &mut SimuladorApp,
    ctx: &Context,
    rect: Rect,
    response: &Response,
    painter: &Painter,
    pointer_world: Option<egui::Vec2>,
    step: f32,
) {
    if app.selected_tool != Some(Tool::Move) {
        app.move_state.drag_target = None;
        app.move_state.drag_start_world = None;
        app.move_state.hovered_road = None;
        app.move_state.hovered_building = None;
        app.move_state.collision_error = false;
        return;
    }

    let time = ctx.input(|i| i.time);

    // ── Limpiar error timer ──
    if app.move_state.collision_error && time > app.move_state.error_timer {
        app.move_state.collision_error = false;
    }

    // ── Detectar hover (solo cuando no está arrastrando) ──
    if app.move_state.drag_target.is_none() {
        app.move_state.hovered_road = None;
        app.move_state.hovered_building = None;

        if let Some(p_world) = pointer_world {
            // Buscar edificio bajo el cursor
            for (idx, building) in app.obstacles.iter().enumerate() {
                if point_in_polygon(p_world, building) {
                    app.move_state.hovered_building = Some(idx);
                    break;
                }
            }
            // Si no es edificio, buscar pista
            if app.move_state.hovered_building.is_none() {
                for (idx, road) in app.road_segments.iter().enumerate() {
                    let dist = dist_to_segment(p_world, road.from, road.to);
                    let half_w = (road.lanes as f32 * 3.0) / 2.0;
                    if dist < half_w {
                        app.move_state.hovered_road = Some(idx);
                        break;
                    }
                }
            }
        }
    }

    // ── Dibujar highlight del hover ──
    if let Some(idx) = app.move_state.hovered_building {
        if idx < app.obstacles.len() {
            let points: Vec<egui::Pos2> = app.obstacles[idx]
                .iter()
                .map(|&pt| app.viewport.world_to_screen(rect, pt))
                .collect();
            painter.add(egui::Shape::closed_line(
                points,
                egui::Stroke::new(2.5, Color32::from_rgba_unmultiplied(250, 204, 21, 200)),
            ));
        }
    }
    if let Some(idx) = app.move_state.hovered_road {
        if idx < app.road_segments.len() {
            let road = &app.road_segments[idx];
            let a = road.from;
            let b = road.to;
            let width = road.lanes as f32 * 3.0;
            let dir = (b - a).normalized();
            let normal = egui::vec2(-dir.y, dir.x);
            let off = normal * (width / 2.0);
            let pts = [
                app.viewport.world_to_screen(rect, a + off),
                app.viewport.world_to_screen(rect, b + off),
                app.viewport.world_to_screen(rect, b - off),
                app.viewport.world_to_screen(rect, a - off),
            ];
            painter.add(egui::Shape::closed_line(
                pts.to_vec(),
                egui::Stroke::new(2.5, Color32::from_rgba_unmultiplied(250, 204, 21, 200)),
            ));
        }
    }

    // ── Mouse Down: iniciar drag ──
    if response.drag_started() {
        if let Some(p_world) = pointer_world {
            if let Some(idx) = app.move_state.hovered_building {
                if idx < app.obstacles.len() {
                    app.move_state.drag_target = Some(DragTarget::Building(idx));
                    app.move_state.original_building = Some(app.obstacles[idx].clone());
                    app.move_state.drag_start_world = Some(p_world);
                }
            } else if let Some(idx) = app.move_state.hovered_road {
                if idx < app.road_segments.len() {
                    let road = &app.road_segments[idx];
                    app.move_state.drag_target = Some(DragTarget::Road(road.id));
                    app.move_state.original_road_from = Some(road.from);
                    app.move_state.original_road_to = Some(road.to);
                    app.move_state.drag_start_world = Some(p_world);
                }
            }
        }
    }

    // ── Dragging: mover el elemento ──
    if response.dragged() {
        if let (Some(p_world), Some(drag_start), Some(target)) = (
            pointer_world,
            app.move_state.drag_start_world,
            app.move_state.drag_target,
        ) {
            let delta = p_world - drag_start;
            // Snap de la posición delta a la grilla
            let snapped_dx = (delta.x / step).round() * step;
            let snapped_dy = (delta.y / step).round() * step;
            let snapped_delta = egui::vec2(snapped_dx, snapped_dy);

            match target {
                DragTarget::Building(idx) => {
                    if idx < app.obstacles.len() {
                        if let Some(original) = &app.move_state.original_building {
                            let new_verts: Vec<egui::Vec2> = original
                                .iter()
                                .map(|&v| v + snapped_delta)
                                .collect();
                            app.obstacles[idx] = new_verts;
                        }
                    }
                }
                DragTarget::Road(road_id) => {
                    if let (Some(orig_from), Some(orig_to)) = (
                        app.move_state.original_road_from,
                        app.move_state.original_road_to,
                    ) {
                        for seg in &mut app.road_segments {
                            if seg.id == road_id {
                                seg.from = orig_from + snapped_delta;
                                seg.to = orig_to + snapped_delta;
                            }
                        }
                    }
                }
            }
        }
    }

    // ── Mouse Up: validar colisión, aplicar snap magnético final ──
    if response.drag_stopped() {
        if let Some(target) = app.move_state.drag_target {
            let mut collision = false;

            match target {
                DragTarget::Building(idx) => {
                    if idx < app.obstacles.len() {
                        // Snap magnético vértice a vértice al soltar
                        let building_copy = app.obstacles[idx].clone();
                        let snapped_building: Vec<egui::Vec2> = building_copy
                            .iter()
                            .map(|&v| snap_to_elements(v, &app.road_segments, &app.obstacles, None, Some(idx)))
                            .collect();
                        // Si algún vértice se snap-eó, usar el desplazamiento del primer snap
                        if snapped_building[0] != building_copy[0] {
                            let snap_delta = snapped_building[0] - building_copy[0];
                            app.obstacles[idx] = building_copy.iter().map(|&v| v + snap_delta).collect();
                        }

                        // Validar colisión con carreteras
                        if building_collides_with_roads(&app.obstacles[idx], &app.road_segments) {
                            // Revertir
                            if let Some(original) = &app.move_state.original_building {
                                app.obstacles[idx] = original.clone();
                            }
                            collision = true;
                        }
                    }
                }
                DragTarget::Road(road_id) => {
                    // Encontrar índice de pista con ese road_id
                    let road_idx = app.road_segments.iter().position(|r| r.id == road_id);
                    if let Some(idx) = road_idx {
                        let road = &app.road_segments[idx];
                        let current_from = road.from;
                        let current_to = road.to;
                        let width = road.lanes as f32 * 3.0;

                        // Snap magnético: intentar snap en from y to
                        let snapped_from = snap_to_elements(current_from, &app.road_segments, &app.obstacles, Some(road_id), None);
                        let snapped_to = snap_to_elements(current_to, &app.road_segments, &app.obstacles, Some(road_id), None);
                        // Solo aplicar snap si algún extremo se atrajo
                        let (final_from, final_to) = if snapped_from != current_from {
                            let d = snapped_from - current_from;
                            (current_from + d, current_to + d)
                        } else if snapped_to != current_to {
                            let d = snapped_to - current_to;
                            (current_from + d, current_to + d)
                        } else {
                            (current_from, current_to)
                        };

                        // Validar colisión con obstáculos y otras pistas
                        let collides_obs = crate::ui::screens::simulator::geom::road_collides_with_obstacles(final_from, final_to, width, &app.obstacles);
                        let collides_roads = road_collides_with_roads(final_from, final_to, width, &app.road_segments, Some(road_id));

                        if collides_obs || collides_roads {
                            // Revertir
                            if let (Some(orig_from), Some(orig_to)) = (
                                app.move_state.original_road_from,
                                app.move_state.original_road_to,
                            ) {
                                for seg in &mut app.road_segments {
                                    if seg.id == road_id {
                                        seg.from = orig_from;
                                        seg.to = orig_to;
                                    }
                                }
                            }
                            collision = true;
                        } else {
                            // Aplicar posición final con snap
                            for seg in &mut app.road_segments {
                                if seg.id == road_id {
                                    seg.from = final_from;
                                    seg.to = final_to;
                                }
                            }
                        }
                    }
                }
            }

            if collision {
                app.move_state.collision_error = true;
                app.move_state.error_timer = time + 2.0; // mostrar error 2 segundos
            }

            // Limpiar estado de drag
            app.move_state.drag_target = None;
            app.move_state.drag_start_world = None;
            app.move_state.original_building = None;
            app.move_state.original_road_from = None;
            app.move_state.original_road_to = None;
        }
    }

    // ── Dibujar preview mientras arrastra ──
    if let Some(target) = app.move_state.drag_target {
        let dragging_color = Color32::from_rgba_unmultiplied(250, 204, 21, 100);
        match target {
            DragTarget::Building(idx) => {
                if idx < app.obstacles.len() && app.obstacles[idx].len() >= 3 {
                    let points: Vec<egui::Pos2> = app.obstacles[idx]
                        .iter()
                        .map(|&pt| app.viewport.world_to_screen(rect, pt))
                        .collect();
                    painter.add(egui::Shape::closed_line(
                        points.clone(),
                        egui::Stroke::new(2.0, Color32::from_rgb(250, 204, 21)),
                    ));
                    for pt in &points {
                        painter.circle_filled(*pt, 4.0, Color32::from_rgb(250, 204, 21));
                    }
                }
            }
            DragTarget::Road(road_id) => {
                for road in app.road_segments.iter().filter(|r| r.id == road_id) {
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
                    painter.add(egui::Shape::convex_polygon(
                        pts.to_vec(),
                        dragging_color,
                        egui::Stroke::new(2.0, Color32::from_rgb(250, 204, 21)),
                    ));
                }
            }
        }
    }

    // ── Mostrar error de colisión ──
    if app.move_state.collision_error {
        egui::Window::new("⚠ No se puede mover")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 80.0])
            .show(ctx, |ui| {
                ui.label("El elemento colisiona en la nueva posición.\nSe ha revertido al lugar original.");
            });
    }
}
