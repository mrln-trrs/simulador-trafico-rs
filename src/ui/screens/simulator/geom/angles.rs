/// Calcula los ángulos internos de un polígono en grados.
pub fn compute_interior_angles(poly: &[egui::Vec2]) -> Vec<f32> {
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
