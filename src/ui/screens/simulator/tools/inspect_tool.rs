use egui::{Color32, Context, Painter, Rect, Response};
use crate::ui::screens::simulator::{InspectedObject, SimuladorApp, Tool};
use crate::ui::screens::simulator::geom::{dist_to_segment, point_in_polygon, compute_interior_angles};

pub fn handle_inspect_tool(
    app: &mut SimuladorApp,
    _ctx: &Context,
    rect: Rect,
    response: &Response,
    painter: &Painter,
) {
    if app.selected_tool != Some(Tool::Inspect) {
        app.selected_inspect_object = None;
        return;
    }

    // Lógica interactiva para Inspeccionar elementos
    if let Some(click_pos) = response.interact_pointer_pos() {
        let click_world = app.viewport.screen_to_world(rect, click_pos);
        if response.clicked_by(egui::PointerButton::Primary) {
            let mut found = false;
            
            // 1. Buscar si se clickeó en un edificio
            for (idx, obstacle) in app.obstacles.iter().enumerate() {
                if point_in_polygon(click_world, obstacle) {
                    app.selected_inspect_object = Some(InspectedObject::Building(idx));
                    found = true;
                    break;
                }
            }
            
            // 2. Si no es un edificio, buscar si es una carretera
            if !found {
                for (idx, road) in app.road_segments.iter().enumerate() {
                    let dist = dist_to_segment(click_world, road.from, road.to);
                    let width = road.lanes as f32 * 3.0;
                    if dist < (width / 2.0) {
                        app.selected_inspect_object = Some(InspectedObject::Road(idx));
                        found = true;
                        break;
                    }
                }
            }
            
            if !found {
                app.selected_inspect_object = None;
            }
        }
    }

    // Renderizar la información del elemento inspeccionado en sus lados y vértices
    if let Some(inspect_obj) = &app.selected_inspect_object {
        match inspect_obj {
            &InspectedObject::Building(idx) => {
                if idx < app.obstacles.len() {
                    let obstacle = &app.obstacles[idx];
                    draw_polygon_inspection_info(
                        painter,
                        rect,
                        &app.viewport,
                        obstacle,
                        &format!("Edificio #{}", idx),
                        Color32::from_rgb(59, 130, 246),
                    );
                }
            }
            &InspectedObject::Road(idx) => {
                if idx < app.road_segments.len() {
                    let road = &app.road_segments[idx];
                    let a = road.from;
                    let b = road.to;
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
                    
                    draw_polygon_inspection_info(
                        painter,
                        rect,
                        &app.viewport,
                        &road_poly,
                        &format!("Pista #{}\nCarriles: {}", road.id, road.lanes),
                        Color32::from_rgb(59, 130, 246),
                    );
                }
            }
        }
    }
}

fn get_vertex_label(mut index: usize) -> String {
    let mut label = String::new();
    loop {
        let remainder = index % 26;
        label.insert(0, (b'A' + remainder as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    label
}

fn draw_polygon_inspection_info(
    painter: &Painter,
    rect: Rect,
    viewport: &crate::ui::screens::simulator::canvas::viewport::GridViewport,
    points: &[egui::Vec2],
    title: &str,
    border_color: Color32,
) {
    let n = points.len();
    if n < 3 { return; }

    // Dibujar contorno de selección
    let points_screen: Vec<egui::Pos2> = points.iter().map(|&pt| viewport.world_to_screen(rect, pt)).collect();
    painter.add(egui::Shape::closed_line(
        points_screen.clone(),
        egui::Stroke::new(3.0, border_color),
    ));

    // Calcular ángulos internos en grados
    let angles = compute_interior_angles(points);

    // Calcular centroide
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    for pt in points {
        sum_x += pt.x;
        sum_y += pt.y;
    }
    let centroid = egui::vec2(sum_x / n as f32, sum_y / n as f32);

    // Dibujar etiquetas de los lados: a: 10m, b: 12m, etc.
    for i in 0..n {
        let v1 = points[i];
        let v2 = points[(i + 1) % n];
        let mid = (v1 + v2) * 0.5;
        let len = (v2 - v1).length();

        let mid_screen = viewport.world_to_screen(rect, mid);
        let side_name = get_vertex_label(i).to_lowercase();
        let text = format!("{}: {:.1}m", side_name, len);

        let text_size = painter.layout_no_wrap(
            text.clone(),
            egui::FontId::proportional(9.5),
            Color32::from_rgb(147, 197, 253),
        ).rect.size();

        // Fondo con opacidad 1.0 (Negro sólido)
        painter.rect_filled(
            Rect::from_center_size(mid_screen, text_size + egui::vec2(8.0, 6.0)),
            3.0,
            Color32::from_black_alpha(255),
        );
        painter.text(
            mid_screen,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(9.5),
            Color32::from_rgb(147, 197, 253),
        );
    }

    // Dibujar etiquetas de los vértices (Nombre, coordenadas, ángulo) y arcos
    for i in 0..n {
        let pt = points[i];
        let angle = angles[i];
        
        let prev = points[(i + n - 1) % n];
        let next = points[(i + 1) % n];

        let pt_screen = viewport.world_to_screen(rect, pt);
        let prev_screen = viewport.world_to_screen(rect, prev);
        let next_screen = viewport.world_to_screen(rect, next);
        let centroid_screen = viewport.world_to_screen(rect, centroid);

        // Calcular vectores dirección
        let _dir_prev = (prev_screen - pt_screen).normalized();
        let dir_next = (next_screen - pt_screen).normalized();

        // Determinar radio del arco de manera proporcional al tamaño de los lados en pantalla
        let len_prev_screen = (prev_screen - pt_screen).length();
        let len_next_screen = (next_screen - pt_screen).length();
        let arc_radius = 20.0f32.min(len_prev_screen * 0.4).min(len_next_screen * 0.4).max(5.0);

        // Determinar la dirección de barrido correcta (hacia el interior del polígono)
        let rad = angle.to_radians();
        let angle_next = dir_next.y.atan2(dir_next.x);
        
        let mid1 = angle_next + rad * 0.5;
        let mid2 = angle_next - rad * 0.5;
        
        let pos1 = pt_screen + egui::vec2(mid1.cos(), mid1.sin()) * arc_radius;
        let pos2 = pt_screen + egui::vec2(mid2.cos(), mid2.sin()) * arc_radius;
        
        let sweep = if pos1.distance_sq(centroid_screen) < pos2.distance_sq(centroid_screen) {
            rad
        } else {
            -rad
        };

        // Generar puntos del arco
        let num_segments = 16;
        let mut arc_points = Vec::with_capacity(num_segments + 1);
        for j in 0..=num_segments {
            let t = j as f32 / num_segments as f32;
            let a = angle_next + sweep * t;
            let arc_pt = pt_screen + egui::vec2(a.cos(), a.sin()) * arc_radius;
            arc_points.push(arc_pt);
        }

        // Dibujar el arco del ángulo
        painter.add(egui::Shape::line(
            arc_points,
            egui::Stroke::new(1.5, Color32::from_rgb(253, 186, 116)),
        ));

        // Calcular posición desplazada para la etiqueta (fuera del arco)
        let delta = centroid_screen - pt_screen;
        let dist = delta.length();
        let label_offset = arc_radius + 16.0;
        let pt_shifted_screen = if dist > 0.001 {
            pt_screen + (delta / dist) * label_offset
        } else {
            pt_screen
        };

        let vertex_name = get_vertex_label(i);
        let text = format!("{}: ({:.1}, {:.1})\n{:.0}°", vertex_name, pt.x, pt.y, angle);

        let text_size = painter.layout_no_wrap(
            text.clone(),
            egui::FontId::proportional(9.0),
            Color32::from_rgb(253, 186, 116),
        ).rect.size();

        // Fondo con opacidad 1.0 (Negro sólido)
        painter.rect_filled(
            Rect::from_center_size(pt_shifted_screen, text_size + egui::vec2(8.0, 6.0)),
            3.0,
            Color32::from_black_alpha(255),
        );
        painter.text(
            pt_shifted_screen,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(9.0),
            Color32::from_rgb(253, 186, 116),
        );
    }

    // Calcular área y perímetro
    let mut area = 0.0;
    let mut perimeter = 0.0;
    for i in 0..n {
        let v1 = points[i];
        let v2 = points[(i + 1) % n];
        area += v1.x * v2.y - v2.x * v1.y;
        perimeter += (v2 - v1).length();
    }
    let area = area.abs() * 0.5;

    // Mostrar info general en el centroide
    let centroid_screen = viewport.world_to_screen(rect, centroid);
    let info_text = format!("{}\nÁrea: {:.1}m²\nPerímetro: {:.1}m", title, area, perimeter);

    let text_size = painter.layout_no_wrap(
        info_text.clone(),
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    ).rect.size();

    // Fondo con opacidad 1.0 (Negro sólido)
    painter.rect_filled(
        Rect::from_center_size(centroid_screen, text_size + egui::vec2(16.0, 16.0)),
        4.0,
        Color32::from_black_alpha(255),
    );
    painter.text(
        centroid_screen,
        egui::Align2::CENTER_CENTER,
        info_text,
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
}
