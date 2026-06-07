/// Calcula la distancia mínima desde un punto P hasta un segmento de recta AB.
pub fn dist_to_segment(p: egui::Vec2, a: egui::Vec2, b: egui::Vec2) -> f32 {
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
pub fn point_in_polygon(p: egui::Vec2, polygon: &[egui::Vec2]) -> bool {
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
pub fn point_in_polygon_strict(p: egui::Vec2, polygon: &[egui::Vec2]) -> bool {
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
