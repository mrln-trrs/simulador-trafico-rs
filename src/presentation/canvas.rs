use crate::integration::snapshots::Snapshot;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};

pub struct CanvasState {
    pub zoom: f32,
    pub pan: Vec2,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }
}

pub fn draw_snapshot(ui: &mut egui::Ui, snapshot: &Snapshot, state: &mut CanvasState) {
    let desired_size = ui.available_size_before_wrap();
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::drag());
    if response.dragged() {
        state.pan += response.drag_delta();
    }

    let scroll = ui.input(|input| input.zoom_delta());
    if (scroll - 1.0).abs() > f32::EPSILON {
        state.zoom = (state.zoom * scroll).clamp(0.2, 4.0);
    }

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_gray(20));

    for segment in &snapshot.segments {
        let from = snapshot.nodes.iter().find(|node| node.id == segment.from);
        let to = snapshot.nodes.iter().find(|node| node.id == segment.to);
        if let (Some(from), Some(to)) = (from, to) {
            let a = world_to_screen(rect, state, from.position.x as f32, from.position.y as f32);
            let b = world_to_screen(rect, state, to.position.x as f32, to.position.y as f32);
            painter.line_segment([a, b], Stroke::new(3.0, Color32::from_rgb(110, 110, 140)));
        }
    }

    for node in &snapshot.nodes {
        let pos = world_to_screen(rect, state, node.position.x as f32, node.position.y as f32);
        painter.circle_filled(pos, 8.0, Color32::from_rgb(220, 220, 120));
        painter.text(
            pos + Vec2::new(10.0, -10.0),
            egui::Align2::LEFT_TOP,
            &node.name,
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
    }

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
                    let t = vehicle.progress as f32;
                    let x = from.position.x as f32
                        + (to.position.x as f32 - from.position.x as f32) * t;
                    let y = from.position.y as f32
                        + (to.position.y as f32 - from.position.y as f32) * t;
                    let pos = world_to_screen(rect, state, x, y);
                    painter.circle_filled(pos, 6.0, Color32::from_rgb(120, 220, 160));
                }
            }
        }
    }
}

fn world_to_screen(rect: Rect, state: &CanvasState, x: f32, y: f32) -> Pos2 {
    Pos2::new(
        rect.center().x + (x * state.zoom) + state.pan.x - rect.width() * 0.5 * state.zoom,
        rect.center().y + (y * state.zoom) + state.pan.y - rect.height() * 0.5 * state.zoom,
    )
}
