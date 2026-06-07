/// Triangulación de un polígono simple (soporta cóncavos) mediante el algoritmo de Ear Clipping.
/// Retorna una lista de triángulos.
pub fn triangulate_polygon(vertices: &[egui::Pos2]) -> Vec<[egui::Pos2; 3]> {
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
