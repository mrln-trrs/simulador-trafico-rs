mod bars;
mod canvas;
mod components;
mod state;
mod geom;
mod tools;
mod windows;

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
    pub(crate) window_state: SavedWindowState,
    pub(crate) viewport: GridViewport,
    pub(crate) cache: GridRenderCache,
    
    // Sidebar state
    pub(crate) sidebar_expanded: bool,
    pub(crate) selected_tool: Option<Tool>,

    // Creación de planos/obstáculos
    pub(crate) building_draft: Vec<egui::Vec2>,
    pub(crate) obstacles: Vec<Vec<egui::Vec2>>,

    // Creación de carreteras/vías
    pub(crate) road_draft: Option<egui::Vec2>,
    pub(crate) road_lanes: usize,
    pub(crate) road_segments: Vec<RoadSegmentGeometry>,
    pub(crate) next_road_id: usize, // Autoincremental para agrupar tramos creados juntos

    // Estado de borrado granular
    pub(crate) delete_mode: DeleteMode,
    pub(crate) delete_lasso_points: Vec<egui::Vec2>,

    // Estado de inspección
    pub(crate) selected_inspect_object: Option<InspectedObject>,

    // Estado de escala global de la interfaz y texto
    pub(crate) ui_zoom: f32,
    pub(crate) text_scale: f32,
    pub(crate) show_settings_window: bool,
    pub(crate) fonts_initialized: bool,
    pub(crate) settings_window_size: Option<[u32; 2]>,
    pub(crate) settings_window_pos: Option<[i32; 2]>,
    pub(crate) egui_ctx: Option<egui::Context>,
    pub(crate) last_frame_time: Option<std::time::Instant>,
    pub(crate) fps: f32,
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

        let ui_zoom = cc.storage
            .and_then(|s| eframe::get_value(s, "main_ui_zoom"))
            .unwrap_or(1.0f32);
        let text_scale = cc.storage
            .and_then(|s| eframe::get_value(s, "main_text_scale"))
            .unwrap_or(1.0f32);

        Self {
            window_state,
            sidebar_expanded: true,
            selected_tool: None,
            road_lanes: 1,
            next_road_id: 0,
            delete_mode: DeleteMode::SubPolygon,
            ui_zoom,
            text_scale,
            show_settings_window: false,
            fonts_initialized: false,
            ..Default::default()
        }
    }

    pub fn new_multiwin() -> Self {
        Self {
            sidebar_expanded: true,
            selected_tool: None,
            road_lanes: 1,
            next_road_id: 0,
            delete_mode: DeleteMode::SubPolygon,
            ui_zoom: 1.0,
            text_scale: 1.0,
            show_settings_window: false,
            fonts_initialized: false,
            ..Default::default()
        }
    }
}

impl SimuladorApp {
    pub fn update_multiwin(&mut self, ctx: &Context, _window: &egui_multiwin::winit::window::Window) {
        let now = std::time::Instant::now();
        if let Some(last) = self.last_frame_time {
            let delta = now.duration_since(last).as_secs_f32();
            if delta > 0.0 {
                self.fps = self.fps * 0.95 + (1.0 / delta) * 0.05;
            }
        }
        self.last_frame_time = Some(now);

        self.egui_ctx = Some(ctx.clone());
        self.window_state.sync_from_context(ctx);

        draw_menu_bar(ctx, self);

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
            .show(ctx, &mut self.sidebar_expanded, &mut self.selected_tool, self.ui_zoom * self.text_scale);

        let mut pointer_world = None;
        let mut viewport_rect = Rect::NOTHING;

        CentralPanel::default().show(ctx, |ui| {
            crate::ui::screens::simulator::windows::settings_window::apply_local_scale(ui, self.ui_zoom * self.text_scale);
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
                    let triangles = geom::triangulate_polygon(&points);
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
                            if !geom::is_point_inside_any_other_road(mid, road.id, &self.road_segments) {
                                painter.line_segment(
                                    [self.viewport.world_to_screen(rect, p1), self.viewport.world_to_screen(rect, p2)],
                                    egui::Stroke::new(1.0, Color32::from_rgb(108, 117, 125)),
                                );
                            }
                        }
                    }
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
                    let triangles = geom::triangulate_polygon(&points);
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

            // Delegar el control y dibujo interactivo a los módulos de herramientas correspondientes
            tools::handle_building_tool(self, ctx, rect, &response, &painter, pointer_world, step);
            tools::handle_road_tool(self, ctx, rect, &response, &painter, pointer_world, step);
            tools::handle_delete_tool(self, ctx, rect, &response, &painter, pointer_world, step);
            tools::handle_inspect_tool(self, ctx, rect, &response, &painter);
        });

        TopBottomPanel::bottom("status_bar")
            .show_separator_line(false)
            .show(ctx, |ui| {
                crate::ui::screens::simulator::windows::settings_window::apply_local_scale(ui, self.ui_zoom * self.text_scale);
                draw_status_bar(ui, &self.viewport, viewport_rect, pointer_world, &mut self.cache, self.fps);
            });

        ctx.request_repaint();
    }
}