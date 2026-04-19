use egui::*;
use crate::presentation::theme::FluentTheme;
use crate::integration::snapshots::VehicleSnapshot;
use crate::presentation::view_model::ViewModel;
use std::collections::HashMap;

/// Tipos de capas visuales
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum RenderLayer {
    Background,
    Geometry,     // Nodos y tramos
    Marks,        // Marcas y carriles
    Control,      // Semáforos
    Dynamic,      // Vehículos
    Heatmap,      // Análisis
    Debug,        // Guías de depuración
    Selection,    // Elementos seleccionados
}

/// Información de transformación de coordenadas
#[derive(Clone, Copy, Debug)]
pub struct ViewTransform {
    pub pan: Vec2,
    pub zoom: f32,
    pub canvas_size: Vec2,
}

impl ViewTransform {
    pub fn new(canvas_size: Vec2) -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            canvas_size,
        }
    }
    
    /// Convierte coordenadas del mundo a pantalla
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        (world_pos + self.pan) * self.zoom
    }
    
    /// Convierte coordenadas de pantalla a mundo
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        (screen_pos / self.zoom) - self.pan
    }
    
    /// Aplica zoom centrando en un punto
    pub fn zoom_at(&mut self, screen_center: Vec2, zoom_factor: f32) {
        let world_before = self.screen_to_world(screen_center);
        self.zoom *= zoom_factor;
        self.pan = (world_before - self.screen_to_world(screen_center)) / self.zoom;
    }
    
    /// Limita el zoom a un rango
    pub fn clamp_zoom(&mut self, min: f32, max: f32) {
        self.zoom = self.zoom.clamp(min, max);
    }
    
    /// Centra la vista en un punto del mundo
    pub fn center_at(&mut self, world_pos: Vec2) {
        self.pan = (self.canvas_size / (2.0 * self.zoom)) - world_pos;
    }
}

/// Estado del canvas (compatible con versión anterior)
pub struct CanvasState {
    pub zoom: f32,
    pub pan: Vec2,
    pub transform: ViewTransform,
    pub selection: SelectionState,
    pub show_grid: bool,
    pub show_debug: bool,
    pub grid_size: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            transform: ViewTransform::new(Vec2::new(800.0, 600.0)),
            selection: SelectionState::default(),
            show_grid: true,
            show_debug: false,
            grid_size: 20.0,
        }
    }
}

/// Comando de dibujo primitivo
#[derive(Clone, Debug)]
pub enum DrawCommand {
    Circle { pos: Vec2, radius: f32, fill: Color32, stroke: Stroke },
    Rect { rect: Rect, fill: Color32, stroke: Stroke, rounding: Rounding },
    Line { points: Vec<Vec2>, stroke: Stroke },
    Text { pos: Vec2, text: String, color: Color32, size: f32 },
    Path { points: Vec<Vec2>, closed: bool, stroke: Stroke, fill: Option<Color32> },
}

/// Canvas mejorado con soporte para capas
pub struct Canvas {
    pub layers: HashMap<RenderLayer, Vec<DrawCommand>>,
    pub theme: FluentTheme,
}

impl Canvas {
    pub fn new(theme: FluentTheme) -> Self {
        Self {
            layers: HashMap::new(),
            theme,
        }
    }
    
    /// Agrega un comando a una capa
    pub fn add_command(&mut self, layer: RenderLayer, command: DrawCommand) {
        self.layers.entry(layer).or_insert_with(Vec::new).push(command);
    }
    
    /// Limpia todos los comandos
    pub fn clear(&mut self) {
        self.layers.clear();
    }
}

/// Sistema de selección
#[derive(Clone, Debug, Default)]
pub struct SelectionState {
    pub selected_ids: Vec<String>,
    pub hover_id: Option<String>,
}

impl SelectionState {
    pub fn is_selected(&self, id: &str) -> bool {
        self.selected_ids.contains(&id.to_string())
    }
    
    pub fn toggle(&mut self, id: String) {
        if let Some(pos) = self.selected_ids.iter().position(|x| x == &id) {
            self.selected_ids.remove(pos);
        } else {
            self.selected_ids.push(id);
        }
    }
    
    pub fn clear(&mut self) {
        self.selected_ids.clear();
        self.hover_id = None;
    }
}

/// Renderiza snapshot con estilo minimalista Fluent
pub fn draw_snapshot(ui: &mut Ui, view_model: &ViewModel, state: &mut CanvasState, theme: &FluentTheme) {
    let snapshot = &view_model.snapshot;
    let desired_size = ui.available_size_before_wrap();
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::drag());
    
    // Actualiza transformación
    state.transform.canvas_size = desired_size;
    
    if response.dragged() {
        state.pan += response.drag_delta();
        state.transform.pan = state.pan / state.zoom;
    }

    let scroll = ui.input(|input| input.zoom_delta());
    if (scroll - 1.0).abs() > f32::EPSILON {
        state.zoom = (state.zoom * scroll).clamp(0.2, 4.0);
        state.transform.zoom = state.zoom;
    }

    let painter = ui.painter_at(rect);
    
    // Fondo
    painter.rect_filled(rect, 0.0, theme.background);

    // Grid
    if state.show_grid {
        draw_grid(&painter, rect, state, theme);
    }

    // Tramos/segmentos
    for segment in &snapshot.segments {
        let from = snapshot.nodes.iter().find(|node| node.id == segment.from);
        let to = snapshot.nodes.iter().find(|node| node.id == segment.to);
        if let (Some(from), Some(to)) = (from, to) {
            let a = world_to_screen(rect, state, from.position.x as f32, from.position.y as f32);
            let b = world_to_screen(rect, state, to.position.x as f32, to.position.y as f32);
            painter.line_segment(
                [a, b],
                Stroke::new(2.0, theme.outline)
            );
        }
    }

    // Nodos
    for node in &snapshot.nodes {
        let pos = world_to_screen(rect, state, node.position.x as f32, node.position.y as f32);
        let radius = 8.0;
        
        painter.circle_filled(pos, radius, theme.primary);
        painter.circle_stroke(pos, radius, Stroke::new(1.5, theme.text_secondary));
        
        painter.text(
            pos + Vec2::new(15.0, -10.0),
            Align2::LEFT_TOP,
            &node.name,
            FontId::proportional(11.0),
            theme.text_primary,
        );
    }

    // Vehículos
    for vehicle in &snapshot.vehicles {
        if let Some(segment_id) = vehicle.current_segment {
            if let Some(segment) = snapshot
                .segments
                .iter()
                .find(|segment| segment.id == segment_id)
            {
                let from = snapshot.nodes.iter().find(|node| node.id == segment.from);
                let to = snapshot.nodes.iter().find(|node| node.id == segment.to);
                if let (Some(from), Some(to)) = (from, to) {
                    let t = interpolated_progress(
                        vehicle,
                        view_model
                            .previous_snapshot
                            .vehicles
                            .iter()
                            .find(|candidate| candidate.id == vehicle.id),
                        view_model.interpolation_alpha,
                    );
                    let x = from.position.x as f32
                        + (to.position.x as f32 - from.position.x as f32) * t;
                    let y = from.position.y as f32
                        + (to.position.y as f32 - from.position.y as f32) * t;
                    let pos = world_to_screen(rect, state, x, y);
                    painter.circle_filled(pos, 5.0, theme.success);
                    painter.circle_stroke(pos, 5.0, Stroke::new(1.0, theme.text_tertiary));
                }
            }
        }
    }
}

fn interpolated_progress(
    current: &VehicleSnapshot,
    previous: Option<&VehicleSnapshot>,
    alpha: f32,
) -> f32 {
    if let Some(previous) = previous {
        if previous.current_segment == current.current_segment {
            let previous_progress = previous.progress as f32;
            let current_progress = current.progress as f32;
            return previous_progress + (current_progress - previous_progress) * alpha.clamp(0.0, 1.0);
        }
    }

    current.progress as f32
}

fn draw_grid(painter: &Painter, rect: Rect, state: &CanvasState, _theme: &FluentTheme) {
    let grid_size = state.grid_size;
    let grid_color = Color32::from_black_alpha(25);
    
    // Aproximado
    for i in 0..200 {
        let x = (i as f32) * grid_size * state.zoom;
        if x > rect.width() {
            break;
        }
        painter.line_segment(
            [Pos2::new(rect.left() + x, rect.top()), Pos2::new(rect.left() + x, rect.bottom())],
            Stroke::new(0.5, grid_color),
        );
    }
    
    for i in 0..200 {
        let y = (i as f32) * grid_size * state.zoom;
        if y > rect.height() {
            break;
        }
        painter.line_segment(
            [Pos2::new(rect.left(), rect.top() + y), Pos2::new(rect.right(), rect.top() + y)],
            Stroke::new(0.5, grid_color),
        );
    }
}

fn world_to_screen(rect: Rect, state: &CanvasState, x: f32, y: f32) -> Pos2 {
    Pos2::new(
        rect.center().x + (x * state.zoom) + state.pan.x - rect.width() * 0.5 * state.zoom,
        rect.center().y + (y * state.zoom) + state.pan.y - rect.height() * 0.5 * state.zoom,
    )
}
