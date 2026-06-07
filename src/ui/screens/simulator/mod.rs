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

            // Lógica interactiva de creación de Edificios/Obstáculos
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
                            // Si hacemos clic cerca del vértice inicial, cerramos el polígono
                            if first_screen.distance(snapped_screen) < 15.0 {
                                if self.building_draft.len() >= 3 {
                                    self.obstacles.push(self.building_draft.clone());
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
                            self.obstacles.push(self.building_draft.clone());
                        }
                        self.building_draft.clear();
                    }

                    // Dibujar el borrador actual y líneas guía
                    if !self.building_draft.is_empty() {
                        let points: Vec<egui::Pos2> = self.building_draft.iter().map(|&pt| self.viewport.world_to_screen(rect, pt)).collect();

                        // Líneas del borrador
                        for i in 0..(points.len() - 1) {
                            painter.line_segment([points[i], points[i+1]], egui::Stroke::new(2.5, Color32::from_rgb(59, 130, 246)));
                        }

                        // Vértices del borrador
                        for &pt in &points {
                            painter.circle_filled(pt, 4.0, Color32::from_rgb(59, 130, 246));
                        }

                        // Línea de previsualización (elástica) hasta el cursor
                        painter.line_segment([*points.last().unwrap(), snapped_screen], egui::Stroke::new(1.5, Color32::from_rgb(156, 163, 175)));

                        // Si el cursor está cerca del inicio, dar feedback visual de cierre
                        let first_screen = points[0];
                        if first_screen.distance(snapped_screen) < 15.0 {
                            painter.circle_filled(first_screen, 8.0, Color32::from_rgba_unmultiplied(59, 130, 246, 120));
                        } else {
                            painter.circle_filled(snapped_screen, 4.0, Color32::from_rgb(156, 163, 175));
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

            // Lógica interactiva para borrar obstáculos haciendo click en cualquiera de sus vértices
            if self.selected_tool == Some(Tool::Delete) && response.clicked() {
                if let Some(click_pos) = response.interact_pointer_pos() {
                    let mut to_remove = None;
                    for (idx, obstacle) in self.obstacles.iter().enumerate() {
                        for &vertex in obstacle {
                            let screen_vertex = self.viewport.world_to_screen(rect, vertex);
                            if screen_vertex.distance(click_pos) < 15.0 {
                                to_remove = Some(idx);
                                break;
                            }
                        }
                        if to_remove.is_some() {
                            break;
                        }
                    }
                    if let Some(idx) = to_remove {
                        self.obstacles.remove(idx);
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