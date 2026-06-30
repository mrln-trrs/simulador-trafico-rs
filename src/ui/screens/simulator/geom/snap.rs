use crate::ui::screens::simulator::RoadSegmentGeometry;

/// Radio de snap magnético en unidades de mundo (metros).
pub const SNAP_RADIUS: f32 = 2.5;

/// Calcula el punto magnético más cercano al cursor a partir de todos los puntos
/// candidatos:
///   - Extremos del eje central de pistas (from / to)
///   - Las 4 esquinas de cada segmento de pista (para alineación borde-a-borde)
///   - Vértices de edificios
///
/// - `pos`: posición en espacio mundo (ya snapeada a la rejilla de 1.5m)
/// - `exclude_road_id`: id de la pista que estamos moviendo (para no snapear a sí misma)
/// - `exclude_building_idx`: índice del edificio que estamos moviendo
///
/// Retorna el punto candidato más cercano si está dentro de `SNAP_RADIUS`; de lo
/// contrario, retorna `pos` sin cambios.
pub fn snap_to_elements(
    pos: egui::Vec2,
    roads: &[RoadSegmentGeometry],
    buildings: &[Vec<egui::Vec2>],
    exclude_road_id: Option<usize>,
    exclude_building_idx: Option<usize>,
) -> egui::Vec2 {
    let mut best_pos = pos;
    let mut best_dist = SNAP_RADIUS;

    // ── Candidatos de pistas ──────────────────────────────────────────────────
    for road in roads {
        if let Some(exc) = exclude_road_id {
            if road.id == exc { continue; }
        }
        if road.from == road.to { continue; }

        let half_w = road.lanes as f32 * 1.5; // la mitad del ancho de la pista
        let dir = (road.to - road.from).normalized();
        let normal = egui::vec2(-dir.y, dir.x);
        let off = normal * half_w;

        // 1. Extremos del eje central
        for &center_pt in &[road.from, road.to] {
            try_snap(&mut best_pos, &mut best_dist, center_pt, pos);
        }

        // 2. Las 4 esquinas del rectángulo de la pista
        //    → permiten alinear el borde de una pista nueva con el borde de ésta
        for &corner in &[
            road.from + off,
            road.from - off,
            road.to + off,
            road.to - off,
        ] {
            try_snap(&mut best_pos, &mut best_dist, corner, pos);
        }

        // 3. Puntos medios de los bordes largos
        //    → para prolongar una pista desde el centro de la lateral
        let mid = (road.from + road.to) * 0.5;
        try_snap(&mut best_pos, &mut best_dist, mid + off, pos);
        try_snap(&mut best_pos, &mut best_dist, mid - off, pos);
    }

    // ── Candidatos de edificios ───────────────────────────────────────────────
    for (i, building) in buildings.iter().enumerate() {
        if let Some(exc) = exclude_building_idx {
            if i == exc { continue; }
        }
        for &vertex in building {
            try_snap(&mut best_pos, &mut best_dist, vertex, pos);
        }
    }

    best_pos
}

#[inline(always)]
fn try_snap(best_pos: &mut egui::Vec2, best_dist: &mut f32, candidate: egui::Vec2, pos: egui::Vec2) {
    let d = (candidate - pos).length();
    if d < *best_dist {
        *best_dist = d;
        *best_pos = candidate;
    }
}
