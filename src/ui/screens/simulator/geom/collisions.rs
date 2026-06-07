use crate::ui::screens::simulator::RoadSegmentGeometry;
use super::distance::{dist_to_segment, point_in_polygon_strict};

/// Comprueba la orientación de tres puntos P, Q, R.
/// Retorna: 0 si son colineales, 1 si es sentido horario (CW), 2 si es sentido antihorario (CCW).
pub fn orientation(p: egui::Vec2, q: egui::Vec2, r: egui::Vec2) -> i32 {
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
pub fn line_segments_cross_properly(a: egui::Vec2, b: egui::Vec2, c: egui::Vec2, d: egui::Vec2) -> bool {
    let o1 = orientation(a, b, c);
    let o2 = orientation(a, b, d);
    let o3 = orientation(c, d, a);
    let o4 = orientation(c, d, b);

    o1 != o2 && o1 != 0 && o2 != 0 && o3 != o4 && o3 != 0 && o4 != 0
}

/// Comprueba si las áreas interiores de dos polígonos colisionan (se solapan).
/// Si solo comparten vértices o bordes sin solapamiento interior, no colisionan.
pub fn polygons_collide(poly1: &[egui::Vec2], poly2: &[egui::Vec2]) -> bool {
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
pub fn road_collides_with_obstacles(a: egui::Vec2, b: egui::Vec2, width: f32, obstacles: &[Vec<egui::Vec2>]) -> bool {
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
pub fn building_collides_with_roads(building: &[egui::Vec2], roads: &[RoadSegmentGeometry]) -> bool {
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
pub fn is_point_inside_any_other_road(p: egui::Vec2, current_road_id: usize, roads: &[RoadSegmentGeometry]) -> bool {
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
