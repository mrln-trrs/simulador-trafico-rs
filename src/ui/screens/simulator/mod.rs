mod bars;
mod canvas;
mod components;
mod state;

use egui::{CentralPanel, Color32, Context, Rect, TopBottomPanel};

use self::bars::menu_bar::draw_menu_bar;
use self::bars::status_bar::draw_status_bar;
use self::canvas::grid::draw_infinite_grid;
use self::canvas::render_cache::GridRenderCache;
use self::canvas::viewport::GridViewport;
use self::state::window_state::SavedWindowState;

use self::components::sidebar::{Sidebar, SidebarItem, SidebarPosition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Tool {
    Road,
    Building,
    Inspect,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InspectedObject {
    Building(usize),
    Road(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum DeleteMode {
    #[default]
    SubPolygon,
    Lasso,
    FullElement,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RoadSegmentGeometry {
    pub from: egui::Vec2,
    pub to: egui::Vec2,
    pub lanes: usize,
    pub id: usize, // Añadimos un identificador único para poder borrar por "segmento completo"
}

#[derive(Default)]
pub struct SimuladorApp {
    window_state: SavedWindowState,
    viewport: GridViewport,
    cache: GridRenderCache,
    
    // Sidebar state
    sidebar_expanded: bool,
    selected_tool: Option<Tool>,

    // Creación de planos/obstáculos
    building_draft: Vec<egui::Vec2>,
    obstacles: Vec<Vec<egui::Vec2>>,

    // Creación de carreteras/vías
    road_draft: Option<egui::Vec2>,
    road_lanes: usize,
    road_segments: Vec<RoadSegmentGeometry>,
    next_road_id: usize, // Autoincremental para agrupar tramos creados juntos

    // Estado de borrado granular
    delete_mode: DeleteMode,
    delete_lasso_points: Vec<egui::Vec2>,

    // Estado de inspección
    selected_inspect_object: Option<InspectedObject>,
}

impl SimuladorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let window_state = SavedWindowState::load(cc.storage);
        window_state.apply_to_context(&cc.egui_ctx);

        // Configurar la fuente de iconos Lucide de manera súper eficiente
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "lucide".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../../../assets/lucide.ttf"
            )),
        );
        
        // Registrar Lucide al final de la lista como fallback. Así, el texto normal (como tu barra superior y de estado)
        // se renderizará con la fuente estándar del sistema, y solo los códigos unicode de los iconos usarán Lucide.
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("lucide".to_owned());
            
        cc.egui_ctx.set_fonts(fonts);

        Self {
            window_state,
            sidebar_expanded: true,
            selected_tool: None,
            road_lanes: 1,
            next_road_id: 0,
            delete_mode: DeleteMode::SubPolygon,
            ..Default::default()
        }
    }
}

impl eframe::App for SimuladorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.window_state.sync_from_context(ctx);

        draw_menu_bar(ctx, &mut self.viewport);

        // Render Sidebar (must be drawn before CentralPanel)
        let sidebar_items = vec![
            SidebarItem {
                value: Tool::Road,
                icon: "\u{e53e}".to_string(), // icon-route
                label: "Carreteras".to_string(),
                tooltip: "Trazado de vías (Carreteras)".to_string(),
            },
            SidebarItem {
                value: Tool::Building,
                icon: "\u{e1cc}".to_string(), // icon-building
                label: "Edificios".to_string(),
                tooltip: "Construir zonas y edificios".to_string(),
            },
            SidebarItem {
                value: Tool::Inspect,
                icon: "\u{e151}".to_string(), // icon-search
                label: "Inspeccionar".to_string(),
                tooltip: "Inspeccionar elementos del mapa".to_string(),
            },
            SidebarItem {
                value: Tool::Delete,
                icon: "\u{e18e}".to_string(), // icon-trash-2
                label: "Borrar".to_string(),
                tooltip: "Eliminar construcciones".to_string(),
            },
        ];


        Sidebar::new("left_sidebar", SidebarPosition::Left, &sidebar_items)
            .show(ctx, &mut self.sidebar_expanded, &mut self.selected_tool);

        let mut pointer_world = None;
        let mut viewport_rect = Rect::NOTHING;

        CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size_before_wrap();
            // Usamos Sense::click_and_drag() para registrar clicks en el lienzo de forma precisa
            let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());
            let painter = ui.painter_at(rect);
            viewport_rect = rect;
            let mut viewport_changed = false;

            let (zoom_delta, hover_pos) = ui.input(|input| (input.zoom_delta(), input.pointer.hover_pos()));

            if response.dragged() {
                self.viewport.pan += response.drag_delta() / self.viewport.zoom;
                viewport_changed = true;
            }

            if (zoom_delta - 1.0).abs() > f32::EPSILON {
                if let Some(pointer_pos) = hover_pos {
                    self.viewport.zoom_at(rect, pointer_pos, zoom_delta);
                } else {
                    self.viewport.zoom = (self.viewport.zoom * zoom_delta).clamp(self.viewport.min_zoom(), self.viewport.max_zoom());
                }
                viewport_changed = true;
            }

            if viewport_changed {
                ctx.request_repaint();
            }

            pointer_world = hover_pos.and_then(|pointer_pos| {
                rect.contains(pointer_pos)
                    .then(|| self.viewport.screen_to_world(rect, pointer_pos))
            });

            // Pintar fondo y rejilla infinita
            painter.rect_filled(rect, 0.0, Color32::from_rgb(16, 18, 22));
            draw_infinite_grid(&painter, rect, &self.viewport, &mut self.cache);

            // Obtener el paso magnético según el zoom actual de la ventana
            let zoom = self.viewport.zoom;
            let step = if zoom <= 28.0 {
                10.0
            } else if zoom >= 200.0 {
                0.01
            } else if zoom >= 80.0 {
                0.1
            } else {
                1.0
            };

            // Dibujar los obstáculos/edificios completados en pantalla
            for obstacle in &self.obstacles {
                if obstacle.len() >= 3 {
                    let points: Vec<egui::Pos2> = obstacle.iter().map(|&pt| self.viewport.world_to_screen(rect, pt)).collect();
                    
                    // Rellenar de forma segura soportando polígonos cóncavos/complejos mediante triangulación por orejas (ear clipping)
                    let fill_color = Color32::from_rgba_unmultiplied(239, 125, 50, 45);
                    let triangles = triangulate_polygon(&points);
                    for tri in triangles {
                        painter.add(egui::Shape::convex_polygon(
                            tri.to_vec(),
                            fill_color,
                            egui::Stroke::NONE,
                        ));
                    }

                    // Dibujar el contorno del polígono cerrado
                    painter.add(egui::Shape::closed_line(
                        points.clone(),
                        egui::Stroke::new(2.0, Color32::from_rgb(239, 125, 50)),
                    ));

                    for pt in points {
                        painter.circle_filled(pt, 3.0, Color32::from_rgb(239, 125, 50));
                    }
                }
            }
            // ================= LAYER 10: ROADS (PISTAS) =================
            // 1. Dibujar el asfalto base (relleno) de todas las pistas para que se fusionen visualmente
            for road in &self.road_segments {
                let a = road.from;
                let b = road.to;
                if a == b { continue; }
                let width = road.lanes as f32 * 3.0;

                let dir = (b - a).normalized();
                let normal = egui::vec2(-dir.y, dir.x);
                let offset = normal * (width / 2.0);

                let pts = [
                    self.viewport.world_to_screen(rect, a + offset),
                    self.viewport.world_to_screen(rect, b + offset),
                    self.viewport.world_to_screen(rect, b - offset),
                    self.viewport.world_to_screen(rect, a - offset),
                ];

                // Relleno de asfalto gris oscuro sin bordes individuales
                painter.add(egui::Shape::convex_polygon(
                    pts.to_vec(),
                    Color32::from_rgb(40, 44, 52),
                    egui::Stroke::NONE,
                ));
            }

            // 2. Dibujar líneas de carriles (recortadas en las intersecciones para una visual limpia)
            for road in &self.road_segments {
                let a = road.from;
                let b = road.to;
                if a == b { continue; }
                let width = road.lanes as f32 * 3.0;
                let dir = (b - a).normalized();
                let normal = egui::vec2(-dir.y, dir.x);

                if road.lanes > 1 {
                    for lane in 1..road.lanes {
                        let fraction = (lane as f32 / road.lanes as f32) - 0.5;
                        let lane_offset = normal * (fraction * width);
                        let la = a + lane_offset;
                        let lb = b + lane_offset;

                        // Dibujar las líneas divisoras segmentadas, saltando tramos dentro de otras pistas (cruces)
                        let lane_len = (lb - la).length();
                        let num_segments = (lane_len * 2.0).round().max(1.0) as usize; // paso de 0.5 metros
                        for i in 0..num_segments {
                            let t1 = i as f32 / num_segments as f32;
                            let t2 = (i + 1) as f32 / num_segments as f32;
                            let p1 = la + t1 * (lb - la);
                            let p2 = la + t2 * (lb - la);

                            let mid = (p1 + p2) * 0.5;
                            if !is_point_inside_any_other_road(mid, road.id, &self.road_segments) {
                                painter.line_segment(
                                    [self.viewport.world_to_screen(rect, p1), self.viewport.world_to_screen(rect, p2)],
                                    egui::Stroke::new(1.0, Color32::from_rgb(108, 117, 125)),
                                );
                            }
                        }
                    }
                }
            }

            // Previsualización y lógica de selección del sub-polígono (cara) actual si el borrador por sub-polígono está activo
            let mut hovered_road_subdivision = None; // (road_idx, sub_idx, p_from, p_to, total_steps)
            if self.selected_tool == Some(Tool::Delete) && self.delete_mode == DeleteMode::SubPolygon {
                if let Some(pw) = pointer_world {
                    for (r_idx, road) in self.road_segments.iter().enumerate() {
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

                // Renderizar el sub-polígono apuntado parpadeando en rojo
                if let Some((_, _, sub_from, sub_to, _)) = hovered_road_subdivision {
                    let road = &self.road_segments[hovered_road_subdivision.unwrap().0];
                    let width = road.lanes as f32 * 3.0;
                    let dir = (sub_to - sub_from).normalized();
                    let normal = egui::vec2(-dir.y, dir.x);
                    let offset = normal * (width / 2.0);

                    let pts = [
                        self.viewport.world_to_screen(rect, sub_from + offset),
                        self.viewport.world_to_screen(rect, sub_to + offset),
                        self.viewport.world_to_screen(rect, sub_to - offset),
                        self.viewport.world_to_screen(rect, sub_from - offset),
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

            // ================= LAYER 100: BUILDINGS (EDIFICIOS) =================
            // Se dibujan siempre por encima de las pistas
            for obstacle in &self.obstacles {
                if obstacle.len() >= 3 {
                    let points: Vec<egui::Pos2> = obstacle.iter().map(|&pt| self.viewport.world_to_screen(rect, pt)).collect();
                    
                    let fill_color = Color32::from_rgb(180, 83, 9); // Marrón/Naranja terracota sólido
                    let border_color = Color32::from_rgb(245, 158, 11); // Naranja brillante
                    
                    // Rellenar de forma segura soportando polígonos cóncavos/complejos mediante triangulación por orejas (ear clipping)
                    let triangles = triangulate_polygon(&points);
                    for tri in triangles {
                        painter.add(egui::Shape::convex_polygon(
                            tri.to_vec(),
                            fill_color,
                            egui::Stroke::NONE,
                        ));
                    }

                    // Dibujar el contorno del polígono cerrado del edificio
                    painter.add(egui::Shape::closed_line(
                        points.clone(),
                        egui::Stroke::new(2.0, border_color),
                    ));

                    for pt in points {
                        painter.circle_filled(pt, 3.5, border_color);
                    }
                }
            }

            // Lógica interactiva de creación de Edificios/Obstáculos (con validación bidireccional contra carreteras)
            if self.selected_tool == Some(Tool::Building) {
                if let Some(p_world) = pointer_world {
                    // Snap magnético
                    let snapped_x = (p_world.x / step).round() * step;
                    let snapped_y = (p_world.y / step).round() * step;
                    let snapped_pos = egui::vec2(snapped_x, snapped_y);
                    let snapped_screen = self.viewport.world_to_screen(rect, snapped_pos);

                    // Acción: Click izquierdo para colocar un vértice
                    if response.clicked_by(egui::PointerButton::Primary) {
                        if !self.building_draft.is_empty() {
                            let first_screen = self.viewport.world_to_screen(rect, self.building_draft[0]);
                            // Si hacemos clic cerca del vértice inicial, intentamos cerrar el polígono
                            if first_screen.distance(snapped_screen) < 15.0 {
                                if self.building_draft.len() >= 3 {
                                    // Comprobar colisión bidireccional: que el nuevo edificio no pise carreteras existentes
                                    if !building_collides_with_roads(&self.building_draft, &self.road_segments) {
                                        self.obstacles.push(self.building_draft.clone());
                                    }
                                }
                                self.building_draft.clear();
                            } else {
                                self.building_draft.push(snapped_pos);
                            }
                        } else {
                            self.building_draft.push(snapped_pos);
                        }
                    }

                    // Acción: Click derecho para cerrar el polígono actual
                    if response.clicked_by(egui::PointerButton::Secondary) {
                        if self.building_draft.len() >= 3 {
                            if !building_collides_with_roads(&self.building_draft, &self.road_segments) {
                                self.obstacles.push(self.building_draft.clone());
                            }
                        }
                        self.building_draft.clear();
                    }

                    // Dibujar el borrador actual y líneas guía
                    if !self.building_draft.is_empty() {
                        // Comprobar si el borrador actual colisiona con pistas para alertar al usuario
                        let mut coll_check = self.building_draft.clone();
                        coll_check.push(snapped_pos);
                        let collides = building_collides_with_roads(&coll_check, &self.road_segments);

                        let preview_line_color = if collides { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(59, 130, 246) };
                        let points: Vec<egui::Pos2> = self.building_draft.iter().map(|&pt| self.viewport.world_to_screen(rect, pt)).collect();

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
                self.building_draft.clear();
            }

            // Lógica interactiva de creación de Carreteras/Vías (Macro-segmentos)
            if self.selected_tool == Some(Tool::Road) {
                if let Some(p_world) = pointer_world {
                    // Snap magnético
                    let snapped_x = (p_world.x / step).round() * step;
                    let snapped_y = (p_world.y / step).round() * step;
                    let snapped_pos = egui::vec2(snapped_x, snapped_y);
                    let snapped_screen = self.viewport.world_to_screen(rect, snapped_pos);

                    let road_width = self.road_lanes as f32 * 3.0;

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
                        let pt = self.viewport.world_to_screen(rect, snapped_pos + offset);
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
                        if let Some(start_pos) = self.road_draft {
                            if start_pos != snapped_pos {
                                // Validar colisiones antes de colocar la pista
                                if !road_collides_with_obstacles(start_pos, snapped_pos, road_width, &self.obstacles) {
                                    // Crear como UN SOLO macrosegmento continuo y rápido
                                    let road_id = self.next_road_id;
                                    self.next_road_id += 1;
                                    
                                    self.road_segments.push(RoadSegmentGeometry {
                                        from: start_pos,
                                        to: snapped_pos,
                                        lanes: self.road_lanes,
                                        id: road_id,
                                    });

                                    // Encadenar trazado
                                    self.road_draft = Some(snapped_pos);
                                }
                            }
                        } else {
                            self.road_draft = Some(snapped_pos);
                        }
                    }

                    // Acción: Click derecho para cancelar/terminar trazado elástico
                    if response.clicked_by(egui::PointerButton::Secondary) {
                        self.road_draft = None;
                    }

                    // Renderizar borrador/elástico actual de la carretera
                    if let Some(start_pos) = self.road_draft {
                        if start_pos != snapped_pos {
                            // Comprobar colisión para decidir el color del preview (rojo si choca)
                            let collides = road_collides_with_obstacles(start_pos, snapped_pos, road_width, &self.obstacles);
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
                                self.viewport.world_to_screen(rect, start_pos + offset),
                                self.viewport.world_to_screen(rect, snapped_pos + offset),
                                self.viewport.world_to_screen(rect, snapped_pos - offset),
                                self.viewport.world_to_screen(rect, start_pos - offset),
                            ];

                            // Dibujar rectángulo del asfalto temporal
                            painter.add(egui::Shape::convex_polygon(
                                pts.to_vec(),
                                preview_color,
                                egui::Stroke::new(1.5, border_color),
                            ));

                            // Línea del eje central
                            painter.line_segment(
                                [self.viewport.world_to_screen(rect, start_pos), snapped_screen],
                                egui::Stroke::new(1.0, border_color),
                            );
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
                            ui.add(egui::Slider::new(&mut self.road_lanes, 1..=4));
                        });
                        ui.label(format!("Ancho total: {} metros", self.road_lanes * 3));
                    });
            } else {
                self.road_draft = None;
            }

            // Dibujar interfaz y lógica del lazo de borrado
            let mut lasso_triggered = false;
            if self.selected_tool == Some(Tool::Delete) && self.delete_mode == DeleteMode::Lasso {
                // Dibujar lazo existente
                if !self.delete_lasso_points.is_empty() {
                    let pts: Vec<egui::Pos2> = self.delete_lasso_points.iter().map(|&pt| self.viewport.world_to_screen(rect, pt)).collect();
                    for i in 0..(pts.len() - 1) {
                        painter.line_segment([pts[i], pts[i+1]], egui::Stroke::new(1.5, Color32::from_rgb(239, 68, 68)));
                    }
                    if let Some(snapped_screen) = pointer_world.map(|pw| {
                        let snapped_x = (pw.x / step).round() * step;
                        let snapped_y = (pw.y / step).round() * step;
                        self.viewport.world_to_screen(rect, egui::vec2(snapped_x, snapped_y))
                    }) {
                        painter.line_segment([*pts.last().unwrap(), snapped_screen], egui::Stroke::new(1.0, Color32::from_rgb(156, 163, 175)));
                        painter.circle_filled(snapped_screen, 4.0, Color32::from_rgb(239, 68, 68));
                    }
                }
            }

            // Lógica interactiva para borrar en base a los 3 modos
            if self.selected_tool == Some(Tool::Delete) {
                if let Some(click_pos) = response.interact_pointer_pos() {
                    let click_world = self.viewport.screen_to_world(rect, click_pos);

                    match self.delete_mode {
                        DeleteMode::SubPolygon => {
                            if response.clicked_by(egui::PointerButton::Primary) {
                                // 1. Borrar edificio si hacemos clic cerca de algún vértice
                                let mut obstacle_to_remove = None;
                                for (idx, obstacle) in self.obstacles.iter().enumerate() {
                                    for &vertex in obstacle {
                                        let screen_vertex = self.viewport.world_to_screen(rect, vertex);
                                        if screen_vertex.distance(click_pos) < 15.0 {
                                            obstacle_to_remove = Some(idx);
                                            break;
                                        }
                                    }
                                    if obstacle_to_remove.is_some() { break; }
                                }

                                if let Some(idx) = obstacle_to_remove {
                                    self.obstacles.remove(idx);
                                } else if let Some((r_idx, _, sub_from, sub_to, _)) = hovered_road_subdivision {
                                    // 2. Borrar sub-polígono dinámico: Divide la carretera en 2 nuevos macro-segmentos con IDs frescas
                                    let old_road = self.road_segments.remove(r_idx);

                                    // Fragmento izquierdo (desde inicio hasta el inicio del subsegmento borrado)
                                    if (sub_from - old_road.from).length() > 0.1 {
                                        let road_id = self.next_road_id;
                                        self.next_road_id += 1;
                                        self.road_segments.push(RoadSegmentGeometry {
                                            from: old_road.from,
                                            to: sub_from,
                                            lanes: old_road.lanes,
                                            id: road_id,
                                        });
                                    }

                                    // Fragmento derecho (desde fin del subsegmento borrado hasta el final)
                                    if (old_road.to - sub_to).length() > 0.1 {
                                        let road_id = self.next_road_id;
                                        self.next_road_id += 1;
                                        self.road_segments.push(RoadSegmentGeometry {
                                            from: sub_to,
                                            to: old_road.to,
                                            lanes: old_road.lanes,
                                            id: road_id,
                                        });
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
                                    self.delete_lasso_points.push(snapped_pos);
                                }

                                // Aplicar lazo con click derecho
                                if response.clicked_by(egui::PointerButton::Secondary) && !self.delete_lasso_points.is_empty() {
                                    lasso_triggered = true;
                                }
                            }
                        }
                        DeleteMode::FullElement => {
                            if response.clicked_by(egui::PointerButton::Primary) {
                                // 1. Borrar edificio entero
                                let mut obstacle_to_remove = None;
                                for (idx, obstacle) in self.obstacles.iter().enumerate() {
                                    for &vertex in obstacle {
                                        let screen_vertex = self.viewport.world_to_screen(rect, vertex);
                                        if screen_vertex.distance(click_pos) < 15.0 {
                                            obstacle_to_remove = Some(idx);
                                            break;
                                        }
                                    }
                                    if obstacle_to_remove.is_some() { break; }
                                }

                                if let Some(idx) = obstacle_to_remove {
                                    self.obstacles.remove(idx);
                                } else {
                                    // 2. Borrar segmento de carretera completo (toda la ID asociada)
                                    let mut road_id_to_remove = None;
                                    for road in &self.road_segments {
                                        let dist = dist_to_segment(click_world, road.from, road.to);
                                        let width = road.lanes as f32 * 3.0;
                                        let screen_dist = dist * self.viewport.zoom;
                                        if screen_dist < (width * self.viewport.zoom / 2.0 + 10.0) {
                                            road_id_to_remove = Some(road.id);
                                            break;
                                        }
                                    }
                                    if let Some(id) = road_id_to_remove {
                                        self.road_segments.retain(|road| road.id != id);
                                    }
                                }
                            }
                        }
                    }
                }

                // Procesar lazo si ha sido disparado por click o interfaz
                if lasso_triggered && !self.delete_lasso_points.is_empty() {
                    // Eliminar edificios que tengan algún vértice dentro del lazo
                    self.obstacles.retain(|obs| {
                        !obs.iter().any(|&vertex| point_in_polygon(vertex, &self.delete_lasso_points))
                    });

                    // Eliminar segmentos de carreteras cuyos centros queden dentro de la selección
                    self.road_segments.retain(|road| {
                        let center = (road.from + road.to) / 2.0;
                        !point_in_polygon(center, &self.delete_lasso_points)
                    });

                    self.delete_lasso_points.clear();
                }

                // Dibujar interfaz flotante para elegir el modo de borrado
                egui::Window::new("Ajustes de Borrado")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::RIGHT_TOP, [-16.0, 64.0])
                    .show(ctx, |ui| {
                        ui.selectable_value(&mut self.delete_mode, DeleteMode::SubPolygon, "1. Caras / Sub-polígonos");
                        ui.selectable_value(&mut self.delete_mode, DeleteMode::Lasso, "2. Lazo Poligonal");
                        ui.selectable_value(&mut self.delete_mode, DeleteMode::FullElement, "3. Elemento Completo");
                        
                        if self.delete_mode == DeleteMode::Lasso && !self.delete_lasso_points.is_empty() {
                            ui.separator();
                            if ui.button("Confirmar Lazo (Click Derecho)").clicked() {
                                lasso_triggered = true;
                            }
                        }
                    });
            } else {
                self.delete_lasso_points.clear();
            }

            // Reset inspect state if tool is switched
            if self.selected_tool != Some(Tool::Inspect) {
                self.selected_inspect_object = None;
            }

            // Lógica interactiva para Inspeccionar elementos
            if self.selected_tool == Some(Tool::Inspect) {
                if let Some(click_pos) = response.interact_pointer_pos() {
                    let click_world = self.viewport.screen_to_world(rect, click_pos);
                    if response.clicked_by(egui::PointerButton::Primary) {
                        let mut found = false;
                        
                        // 1. Buscar si se clickeó en un edificio
                        for (idx, obstacle) in self.obstacles.iter().enumerate() {
                            if point_in_polygon(click_world, obstacle) {
                                self.selected_inspect_object = Some(InspectedObject::Building(idx));
                                found = true;
                                break;
                            }
                        }
                        
                        // 2. Si no es un edificio, buscar si es una carretera
                        if !found {
                            for (idx, road) in self.road_segments.iter().enumerate() {
                                let dist = dist_to_segment(click_world, road.from, road.to);
                                let width = road.lanes as f32 * 3.0;
                                if dist < (width / 2.0) {
                                    self.selected_inspect_object = Some(InspectedObject::Road(idx));
                                    found = true;
                                    break;
                                }
                            }
                        }
                        
                        if !found {
                            self.selected_inspect_object = None;
                        }
                    }
                }
            }

            // Renderizar la información del elemento inspeccionado en sus lados y vértices
            if let Some(inspect_obj) = &self.selected_inspect_object {
                match inspect_obj {
                    &InspectedObject::Building(idx) => {
                        if idx < self.obstacles.len() {
                            let obstacle = &self.obstacles[idx];
                            let n = obstacle.len();
                            if n >= 3 {
                                // Dibujar contorno de selección en azul
                                let points_screen: Vec<egui::Pos2> = obstacle.iter().map(|&pt| self.viewport.world_to_screen(rect, pt)).collect();
                                painter.add(egui::Shape::closed_line(
                                    points_screen.clone(),
                                    egui::Stroke::new(3.0, Color32::from_rgb(59, 130, 246)),
                                ));
                                
                                // Calcular ángulos internos en grados
                                let angles = compute_interior_angles(obstacle);
                                
                                // Calcular centroide aproximado
                                let mut sum_x = 0.0;
                                let mut sum_y = 0.0;
                                for pt in obstacle {
                                    sum_x += pt.x;
                                    sum_y += pt.y;
                                }
                                let centroid = egui::vec2(sum_x / n as f32, sum_y / n as f32);
                                
                                // Dibujar el largo de cada lado
                                for i in 0..n {
                                    let v1 = obstacle[i];
                                    let v2 = obstacle[(i + 1) % n];
                                    let mid = (v1 + v2) * 0.5;
                                    let len = (v2 - v1).length();
                                    
                                    let mid_screen = self.viewport.world_to_screen(rect, mid);
                                    let text = format!("{:.1}m", len);
                                    
                                    // Fondo semitransparente para mejor legibilidad
                                    painter.circle_filled(mid_screen, 12.0, Color32::from_black_alpha(180));
                                    painter.text(
                                        mid_screen,
                                        egui::Align2::CENTER_CENTER,
                                        text,
                                        egui::FontId::proportional(11.0),
                                        Color32::from_rgb(147, 197, 253), // Azul claro
                                    );
                                }
                                
                                // Dibujar los ángulos en los vértices (desplazados hacia el interior del polígono)
                                for i in 0..n {
                                    let pt = obstacle[i];
                                    let angle = angles[i];
                                    
                                    let to_centroid = (centroid - pt).normalized();
                                    let pt_shifted = pt + to_centroid * 0.5; // Desplazar 0.5 metros en coordenadas de simulación
                                    let pt_screen = self.viewport.world_to_screen(rect, pt_shifted);
                                    
                                    let text = format!("{:.0}°", angle);
                                    painter.circle_filled(pt_screen, 10.0, Color32::from_black_alpha(180));
                                    painter.text(
                                        pt_screen,
                                        egui::Align2::CENTER_CENTER,
                                        text,
                                        egui::FontId::proportional(10.0),
                                        Color32::from_rgb(253, 186, 116), // Naranja claro
                                    );
                                }
                                
                                // Calcular área y perímetro
                                let mut area = 0.0;
                                let mut perimeter = 0.0;
                                for i in 0..n {
                                    let v1 = obstacle[i];
                                    let v2 = obstacle[(i + 1) % n];
                                    area += v1.x * v2.y - v2.x * v1.y;
                                    perimeter += (v2 - v1).length();
                                }
                                let area = area.abs() * 0.5;
                                
                                // Mostrar info general del edificio en el centroide
                                let centroid_screen = self.viewport.world_to_screen(rect, centroid);
                                let info_text = format!("Edificio #{}\nÁrea: {:.1}m²\nPerímetro: {:.1}m", idx, area, perimeter);
                                
                                let text_size = painter.layout_no_wrap(
                                    info_text.clone(),
                                    egui::FontId::proportional(12.0),
                                    Color32::WHITE
                                ).rect.size();
                                
                                painter.rect_filled(
                                    Rect::from_center_size(centroid_screen, text_size + egui::vec2(16.0, 16.0)),
                                    4.0,
                                    Color32::from_black_alpha(210)
                                );
                                
                                painter.text(
                                    centroid_screen,
                                    egui::Align2::CENTER_CENTER,
                                    info_text,
                                    egui::FontId::proportional(12.0),
                                    Color32::WHITE,
                                );
                            }
                        }
                    }
                    &InspectedObject::Road(idx) => {
                        if idx < self.road_segments.len() {
                            let road = &self.road_segments[idx];
                            let a = road.from;
                            let b = road.to;
                            let width = road.lanes as f32 * 3.0;
                            let dir = (b - a).normalized();
                            let normal = egui::vec2(-dir.y, dir.x);
                            let offset = normal * (width / 2.0);
                            
                            // Dibujar contorno de selección en azul
                            let pts = [
                                self.viewport.world_to_screen(rect, a + offset),
                                self.viewport.world_to_screen(rect, b + offset),
                                self.viewport.world_to_screen(rect, b - offset),
                                self.viewport.world_to_screen(rect, a - offset),
                            ];
                            painter.add(egui::Shape::closed_line(
                                pts.to_vec(),
                                egui::Stroke::new(3.0, Color32::from_rgb(59, 130, 246)),
                            ));
                            
                            let len = (b - a).length();
                            let mid = (a + b) * 0.5;
                            
                            // Mostrar info en el centro de la carretera
                            let mid_screen = self.viewport.world_to_screen(rect, mid);
                            let info_text = format!("Pista #{}\nCarriles: {}\nLargo: {:.1}m\nAncho: {:.1}m", road.id, road.lanes, len, width);
                            
                            let text_size = painter.layout_no_wrap(
                                info_text.clone(),
                                egui::FontId::proportional(12.0),
                                Color32::WHITE
                                ).rect.size();
                            
                            painter.rect_filled(
                                Rect::from_center_size(mid_screen, text_size + egui::vec2(16.0, 16.0)),
                                4.0,
                                Color32::from_black_alpha(210)
                            );
                            
                            painter.text(
                                mid_screen,
                                egui::Align2::CENTER_CENTER,
                                info_text,
                                egui::FontId::proportional(12.0),
                                Color32::WHITE,
                            );
                        }
                    }
                }
            }
        });

        TopBottomPanel::bottom("status_bar")
            .show_separator_line(false)
            .show(ctx, |ui| {
                draw_status_bar(ui, &self.viewport, viewport_rect, pointer_world, &mut self.cache);
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.window_state.save(storage);
    }
}

/// Triangulación de un polígono simple (soporta cóncavos) mediante el algoritmo de Ear Clipping.
/// Retorna una lista de triángulos.
fn triangulate_polygon(vertices: &[egui::Pos2]) -> Vec<[egui::Pos2; 3]> {
    let mut triangles = Vec::new();
    if vertices.len() < 3 {
        return triangles;
    }
    
    let mut indices: Vec<usize> = (0..vertices.len()).collect();
    
    // Comprobar si un punto está dentro de un triángulo
    fn point_in_triangle(p: egui::Pos2, a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> bool {
        let det_ab = (p.x - a.x) * (b.y - a.y) - (p.y - a.y) * (b.x - a.x);
        let det_bc = (p.x - b.x) * (c.y - b.y) - (p.y - b.y) * (c.x - b.x);
        let det_ca = (p.x - c.x) * (a.y - c.y) - (p.y - c.y) * (a.x - c.x);
        
        let has_neg = (det_ab < 0.0) || (det_bc < 0.0) || (det_ca < 0.0);
        let has_pos = (det_ab > 0.0) || (det_bc > 0.0) || (det_ca > 0.0);
        
        !(has_neg && has_pos)
    }

    // Comprobar si un vértice es una "oreja" (ear)
    fn is_ear(vertices: &[egui::Pos2], u: usize, v: usize, w: usize, indices: &[usize]) -> bool {
        let a = vertices[u];
        let b = vertices[v];
        let c = vertices[w];
        
        // El triángulo debe estar orientado en sentido antihorario
        let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        if area <= 0.0 {
            return false;
        }
        
        for &idx in indices {
            if idx == u || idx == v || idx == w {
                continue;
            }
            if point_in_triangle(vertices[idx], a, b, c) {
                return false;
            }
        }
        true
    }

    // Asegurar sentido antihorario (CCW). Si es horario (CW), invertimos el orden de trabajo.
    let mut area = 0.0;
    for i in 0..vertices.len() {
        let j = (i + 1) % vertices.len();
        area += (vertices[i].x * vertices[j].y) - (vertices[j].x * vertices[i].y);
    }
    let mut working_vertices = vertices.to_vec();
    if area < 0.0 {
        working_vertices.reverse();
    }

    let mut count = 2 * indices.len();
    while indices.len() > 2 && count > 0 {
        count -= 1;
        let n = indices.len();
        let mut ear_found = false;
        for i in 0..n {
            let u = indices[(i + n - 1) % n];
            let v = indices[i];
            let w = indices[(i + 1) % n];
            
            if is_ear(&working_vertices, u, v, w, &indices) {
                triangles.push([working_vertices[u], working_vertices[v], working_vertices[w]]);
                indices.remove(i);
                ear_found = true;
                break;
            }
        }
        if !ear_found {
            // Fallback si no es un polígono simple: cortar el triángulo de la esquina de todos modos
            let u = indices[0];
            let v = indices[1];
            let w = indices[2];
            triangles.push([working_vertices[u], working_vertices[v], working_vertices[w]]);
            indices.remove(1);
        }
    }
    
    triangles
}

/// Comprueba la orientación de tres puntos P, Q, R.
/// Retorna: 0 si son colineales, 1 si es sentido horario (CW), 2 si es sentido antihorario (CCW).
fn orientation(p: egui::Vec2, q: egui::Vec2, r: egui::Vec2) -> i32 {
    let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);
    let epsilon = 1e-4; // tolerancia para evitar problemas de precisión numérica
    if val.abs() < epsilon {
        0
    } else if val > 0.0 {
        1
    } else {
        2
    }
}

/// Comprueba si dos segmentos de recta AB y CD se cruzan propiamente (se intersectan en sus interiores).
/// Compartir extremos o tocarse en los extremos no se considera cruce propio.
fn line_segments_cross_properly(a: egui::Vec2, b: egui::Vec2, c: egui::Vec2, d: egui::Vec2) -> bool {
    let o1 = orientation(a, b, c);
    let o2 = orientation(a, b, d);
    let o3 = orientation(c, d, a);
    let o4 = orientation(c, d, b);

    o1 != o2 && o1 != 0 && o2 != 0 && o3 != o4 && o3 != 0 && o4 != 0
}

/// Calcula la distancia mínima desde un punto P hasta un segmento de recta AB.
fn dist_to_segment(p: egui::Vec2, a: egui::Vec2, b: egui::Vec2) -> f32 {
    let l2 = (a - b).length_sq();
    if l2 == 0.0 {
        return (p - a).length();
    }
    let t = ((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2;
    let t = t.clamp(0.0, 1.0);
    let projection = a + t * (b - a);
    (p - projection).length()
}

/// Comprueba si un punto P está dentro de un polígono en 2D (algoritmo ray casting).
fn point_in_polygon(p: egui::Vec2, polygon: &[egui::Vec2]) -> bool {
    if polygon.len() < 3 { return false; }
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        if ((polygon[i].y > p.y) != (polygon[j].y > p.y))
            && (p.x < (polygon[j].x - polygon[i].x) * (p.y - polygon[i].y) / (polygon[j].y - polygon[i].y) + polygon[i].x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Comprueba si un punto P está estrictamente dentro de un polígono (excluyendo el borde y vértices compartidos).
fn point_in_polygon_strict(p: egui::Vec2, polygon: &[egui::Vec2]) -> bool {
    if polygon.len() < 3 { return false; }
    let boundary_epsilon = 0.05; // 5 cm de tolerancia
    for i in 0..polygon.len() {
        let v1 = polygon[i];
        let v2 = polygon[(i + 1) % polygon.len()];
        if dist_to_segment(p, v1, v2) < boundary_epsilon {
            return false; // está en el borde o muy cerca del vértice, no es estrictamente interior
        }
    }
    point_in_polygon(p, polygon)
}

/// Comprueba si las áreas interiores de dos polígonos colisionan (se solapan).
/// Si solo comparten vértices o bordes sin solapamiento interior, no colisionan.
fn polygons_collide(poly1: &[egui::Vec2], poly2: &[egui::Vec2]) -> bool {
    if poly1.len() < 3 || poly2.len() < 3 { return false; }

    // 1. Validar si algún borde se cruza propiamente
    for i in 0..poly1.len() {
        let a = poly1[i];
        let b = poly1[(i + 1) % poly1.len()];
        for j in 0..poly2.len() {
            let c = poly2[j];
            let d = poly2[(j + 1) % poly2.len()];
            if line_segments_cross_properly(a, b, c, d) {
                return true;
            }
        }
    }

    // 2. Validar si algún vértice de poly1 está estrictamente dentro de poly2
    for &p in poly1 {
        if point_in_polygon_strict(p, poly2) {
            return true;
        }
    }

    // 3. Validar si algún vértice de poly2 está estrictamente dentro de poly1
    for &p in poly2 {
        if point_in_polygon_strict(p, poly1) {
            return true;
        }
    }

    false
}

/// Comprueba si un segmento de carretera (de grosor `width`) colisiona con algún obstáculo/edificio.
fn road_collides_with_obstacles(a: egui::Vec2, b: egui::Vec2, width: f32, obstacles: &[Vec<egui::Vec2>]) -> bool {
    if a == b {
        return false;
    }
    let dir = (b - a).normalized();
    let normal = egui::vec2(-dir.y, dir.x);
    let offset = normal * (width / 2.0);

    let road_poly = vec![
        a + offset,
        b + offset,
        b - offset,
        a - offset,
    ];

    for obs in obstacles {
        if polygons_collide(&road_poly, obs) {
            return true;
        }
    }
    false
}

/// Comprueba si un nuevo edificio colisiona con alguna de las carreteras existentes.
fn building_collides_with_roads(building: &[egui::Vec2], roads: &[RoadSegmentGeometry]) -> bool {
    if building.len() < 3 { return false; }
    for road in roads {
        let a = road.from;
        let b = road.to;
        if a == b { continue; }
        let width = road.lanes as f32 * 3.0;
        let dir = (b - a).normalized();
        let normal = egui::vec2(-dir.y, dir.x);
        let offset = normal * (width / 2.0);

        let road_poly = vec![
            a + offset,
            b + offset,
            b - offset,
            a - offset,
        ];

        if polygons_collide(building, &road_poly) {
            return true;
        }
    }
    false
}

/// Comprueba si un punto específico está dentro del espacio (rectángulo) de alguna otra carretera.
fn is_point_inside_any_other_road(p: egui::Vec2, current_road_id: usize, roads: &[RoadSegmentGeometry]) -> bool {
    for road in roads {
        if road.id == current_road_id { continue; }
        let dist = dist_to_segment(p, road.from, road.to);
        let half_width = (road.lanes as f32 * 3.0) / 2.0;
        if dist < half_width {
            return true;
        }
    }
    false
}

/// Calcula los ángulos internos de un polígono en grados.
fn compute_interior_angles(poly: &[egui::Vec2]) -> Vec<f32> {
    let n = poly.len();
    if n < 3 {
        return vec![0.0; n];
    }
    
    // Determinar la orientación del devanado (winding order) del polígono
    let mut area = 0.0;
    for i in 0..n {
        let v1 = poly[i];
        let v2 = poly[(i + 1) % n];
        area += (v2.x - v1.x) * (v2.y + v1.y);
    }
    let is_ccw = area < 0.0; // En coordenadas de pantalla (Y hacia abajo), el área negativa indica sentido antihorario

    let mut angles = Vec::with_capacity(n);
    for i in 0..n {
        let prev = poly[(i + n - 1) % n];
        let curr = poly[i];
        let next = poly[(i + 1) % n];

        let incoming = (curr - prev).normalized();
        let outgoing = (next - curr).normalized();

        let inc_angle = incoming.y.atan2(incoming.x);
        let out_angle = outgoing.y.atan2(outgoing.x);

        let mut turning_angle = out_angle - inc_angle;
        while turning_angle > std::f32::consts::PI {
            turning_angle -= 2.0 * std::f32::consts::PI;
        }
        while turning_angle < -std::f32::consts::PI {
            turning_angle += 2.0 * std::f32::consts::PI;
        }

        let interior_angle = if is_ccw {
            std::f32::consts::PI - turning_angle
        } else {
            std::f32::consts::PI + turning_angle
        };
        
        let mut deg = interior_angle.to_degrees();
        if deg < 0.0 { deg += 360.0; }
        if deg >= 360.0 { deg -= 360.0; }
        angles.push(deg);
    }
    angles
}
