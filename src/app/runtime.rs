use std::sync::Arc;
use egui_multiwin::egui_glow::EguiGlow;
use egui_multiwin::tracked_window::TrackedWindowOptions;

use crate::ui::screens::simulator::SimuladorApp;

// 1. Invocar macros de egui-multiwin para generar los módulos de ventana con rutas absolutas
egui_multiwin::tracked_window!(crate::app::runtime::AppCommon, egui_multiwin::NoEvent, crate::app::runtime::MyWindows);
egui_multiwin::multi_window!(crate::app::runtime::AppCommon, egui_multiwin::NoEvent, crate::app::runtime::MyWindows);

// Importar tipos generados por las macros en el scope de este archivo
use tracked_window::RedrawResponse;
use tracked_window::TrackedWindow;

// 2. Definir el estado común a todas las ventanas con el método requerido por las macros
pub struct AppCommon {
    pub simulator_app: SimuladorApp,
}

impl AppCommon {
    pub fn process_event(&mut self, _event: egui_multiwin::NoEvent) -> Vec<multi_window::NewWindowRequest> {
        vec![]
    }
}

// 3. Declarar el enum de ventanas usando enum_dispatch
#[egui_multiwin::enum_dispatch::enum_dispatch(TrackedWindow)]
pub enum MyWindows {
    Simulator(SimulatorWindow),
    Settings(SettingsWindow),
}

// 4. Implementación de la Ventana Principal del Simulador
pub struct SimulatorWindow {}

impl SimulatorWindow {
    pub fn new() -> Self {
        Self {}
    }
}

impl TrackedWindow for SimulatorWindow {
    fn is_root(&self) -> bool {
        true
    }

    fn redraw(
        &mut self,
        c: &mut AppCommon,
        egui: &mut EguiGlow,
        window: &egui_multiwin::winit::window::Window,
        _clipboard: &mut egui_multiwin::arboard::Clipboard,
    ) -> RedrawResponse {
        let mut rr = RedrawResponse::default();
        let ctx = &egui.egui_ctx;

        // Inicializar fuentes del simulador (incluyendo Lucide icon font) de manera aislada
        if !c.simulator_app.fonts_initialized {
            let mut fonts = egui_multiwin::egui::FontDefinitions::default();
            fonts.font_data.insert(
                "lucide".to_owned(),
                egui_multiwin::egui::FontData::from_static(include_bytes!(
                    "../../assets/lucide.ttf"
                )),
            );
            fonts
                .families
                .entry(egui_multiwin::egui::FontFamily::Proportional)
                .or_default()
                .push("lucide".to_owned());
            ctx.set_fonts(fonts);
            c.simulator_app.fonts_initialized = true;
        }

        // Llamar a la lógica de pintado del simulador
        c.simulator_app.update_multiwin(ctx, window);

        // Si se solicitó abrir la ventana de configuración nativa:
        if c.simulator_app.show_settings_window {
            c.simulator_app.show_settings_window = false;

            let settings_win = SettingsWindow::new();
            let mut builder = egui_multiwin::winit::window::WindowBuilder::new()
                .with_title("Configuración del Simulador")
                .with_resizable(true);

            if let Some(size) = c.simulator_app.settings_window_size {
                builder = builder.with_inner_size(egui_multiwin::winit::dpi::PhysicalSize::new(size[0], size[1]));
            } else {
                builder = builder.with_inner_size(egui_multiwin::winit::dpi::LogicalSize::new(320, 180));
            }

            if let Some(pos) = c.simulator_app.settings_window_pos {
                builder = builder.with_position(egui_multiwin::winit::dpi::PhysicalPosition::new(pos[0], pos[1]));
            }

            let options = TrackedWindowOptions {
                shader: None,
                vsync: true,
            };

            let req = multi_window::NewWindowRequest::new(
                MyWindows::Settings(settings_win),
                builder,
                options,
                egui_multiwin::multi_window::new_id(),
            );
            rr.new_windows.push(req);
        }

        rr
    }
}

// 5. Implementación de la Ventana de Configuración Aislada
pub struct SettingsWindow {
    menu_app: crate::ui::screens::menu::MenuApp,
}

impl SettingsWindow {
    pub fn new() -> Self {
        Self {
            menu_app: crate::ui::screens::menu::MenuApp::new(),
        }
    }
}

impl TrackedWindow for SettingsWindow {
    fn redraw(
        &mut self,
        c: &mut AppCommon,
        egui: &mut EguiGlow,
        window: &egui_multiwin::winit::window::Window,
        _clipboard: &mut egui_multiwin::arboard::Clipboard,
    ) -> RedrawResponse {
        let mut rr = RedrawResponse::default();
        let ctx = &egui.egui_ctx;

        self.menu_app.show(ctx, &mut c.simulator_app, window, &mut rr);

        rr
    }
}

// 6. Lanzamiento del motor multi-ventana
pub fn launch_simulator() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = egui_multiwin::winit::event_loop::EventLoopBuilder::with_user_event().build()?;

    let mut multi_window = multi_window::MultiWindow::new();

    let mut common = AppCommon {
        simulator_app: SimuladorApp::new_multiwin(),
    };

    let simulator_win = SimulatorWindow::new();
    let builder = egui_multiwin::winit::window::WindowBuilder::new()
        .with_title("Simulador de Tráfico")
        .with_inner_size(egui_multiwin::winit::dpi::LogicalSize::new(1440.0, 900.0))
        .with_min_inner_size(egui_multiwin::winit::dpi::LogicalSize::new(960.0, 640.0));

    let options = TrackedWindowOptions {
        shader: None,
        vsync: true,
    };

    let req = multi_window::NewWindowRequest::new(
        MyWindows::Simulator(simulator_win),
        builder,
        options,
        egui_multiwin::multi_window::new_id(),
    );

    multi_window.add(req, &mut common, &event_loop)?;

    multi_window.run(event_loop, common)?;

    Ok(())
}
