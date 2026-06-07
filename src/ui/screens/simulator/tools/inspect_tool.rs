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
                    let n = obstacle.len();
                    if n >= 3 {
                        // Dibujar contorno de selección en azul
                        let points_screen: Vec<egui::Pos2> = obstacle.iter().map(|&pt| app.viewport.world_to_screen(rect, pt)).collect();
                        painter.add(egui::Shape::closed_line(
                            points_screen.clone(),
                            egui::Stroke::new(3.0, Color32::from_rgb(59, 130, 246)),
                        ));
                        
                        // Calcular ángulos internos en grados
                        let angles = compute_interior_angles(obstacle);
                        
                        // Calcular centroide aproximado
                        let mut sum_x = 0.0;
                        let mut sum_y = 0.0;
                        for pt in obstacle {
                            sum_x += pt.x;
                            sum_y += pt.y;
                        }
                        let centroid = egui::vec2(sum_x / n as f32, sum_y / n as f32);
                        
                        // Dibujar el largo de cada lado
                        for i in 0..n {
                            let v1 = obstacle[i];
                            let v2 = obstacle[(i + 1) % n];
                            let mid = (v1 + v2) * 0.5;
                            let len = (v2 - v1).length();
                            
                            let mid_screen = app.viewport.world_to_screen(rect, mid);
                            let text = format!("{:.1}m", len);
                            
                            // Fondo semitransparente para mejor legibilidad
                            painter.circle_filled(mid_screen, 12.0, Color32::from_black_alpha(180));
                            painter.text(
                                mid_screen,
                                egui::Align2::CENTER_CENTER,
                                text,
                                egui::FontId::proportional(11.0),
                                Color32::from_rgb(147, 197, 253), // Azul claro
                            );
                        }
                        
                        // Dibujar los ángulos en los vértices (desplazados hacia el interior del polígono)
                        for i in 0..n {
                            let pt = obstacle[i];
                            let angle = angles[i];
                            
                            let to_centroid = (centroid - pt).normalized();
                            let pt_shifted = pt + to_centroid * 0.5; // Desplazar 0.5 metros en coordenadas de simulación
                            let pt_screen = app.viewport.world_to_screen(rect, pt_shifted);
                            
                            let text = format!("{:.0}°", angle);
                            painter.circle_filled(pt_screen, 10.0, Color32::from_black_alpha(180));
                            painter.text(
                                pt_screen,
                                egui::Align2::CENTER_CENTER,
                                text,
                                egui::FontId::proportional(10.0),
                                Color32::from_rgb(253, 186, 116), // Naranja claro
                            );
                        }
                        
                        // Calcular área y perímetro
                        let mut area = 0.0;
                        let mut perimeter = 0.0;
                        for i in 0..n {
                            let v1 = obstacle[i];
                            let v2 = obstacle[(i + 1) % n];
                            area += v1.x * v2.y - v2.x * v1.y;
                            perimeter += (v2 - v1).length();
                        }
                        let area = area.abs() * 0.5;
                        
                        // Mostrar info general del edificio en el centroide
                        let centroid_screen = app.viewport.world_to_screen(rect, centroid);
                        let info_text = format!("Edificio #{}\nÁrea: {:.1}m²\nPerímetro: {:.1}m", idx, area, perimeter);
                        
                        let text_size = painter.layout_no_wrap(
                            info_text.clone(),
                            egui::FontId::proportional(12.0),
                            Color32::WHITE
                        ).rect.size();
                        
                        painter.rect_filled(
                            Rect::from_center_size(centroid_screen, text_size + egui::vec2(16.0, 16.0)),
                            4.0,
                            Color32::from_black_alpha(210)
                        );
                        
                        painter.text(
                            centroid_screen,
                            egui::Align2::CENTER_CENTER,
                            info_text,
                            egui::FontId::proportional(12.0),
                            Color32::WHITE,
                        );
                    }
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
                    
                    // Dibujar contorno de selección en azul
                    let pts = [
                        app.viewport.world_to_screen(rect, a + offset),
                        app.viewport.world_to_screen(rect, b + offset),
                        app.viewport.world_to_screen(rect, b - offset),
                        app.viewport.world_to_screen(rect, a - offset),
                    ];
                    painter.add(egui::Shape::closed_line(
                        pts.to_vec(),
                        egui::Stroke::new(3.0, Color32::from_rgb(59, 130, 246)),
                    ));
                    
                    let len = (b - a).length();
                    let mid = (a + b) * 0.5;
                    
                    // Mostrar info en el centro de la carretera
                    let mid_screen = app.viewport.world_to_screen(rect, mid);
                    let info_text = format!("Pista #{}\nCarriles: {}\nLargo: {:.1}m\nAncho: {:.1}m", road.id, road.lanes, len, width);
                    
                    let text_size = painter.layout_no_wrap(
                        info_text.clone(),
                        egui::FontId::proportional(12.0),
                        Color32::WHITE
                        ).rect.size();
                    
                    painter.rect_filled(
                        Rect::from_center_size(mid_screen, text_size + egui::vec2(16.0, 16.0)),
                        4.0,
                        Color32::from_black_alpha(210)
                    );
                    
                    painter.text(
                        mid_screen,
                        egui::Align2::CENTER_CENTER,
                        info_text,
                        egui::FontId::proportional(12.0),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
}
