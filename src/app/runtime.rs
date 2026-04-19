use crate::presentation::app_shell::SimulatorApp;
use crate::presentation::simulator_shell::SimuladorApp;

fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([960.0, 640.0]),
        centered: true,
        persist_window: true,
        ..Default::default()
    }
}

pub fn launch(app: SimulatorApp) -> eframe::Result<()> {
    eframe::run_native(
        "Simulador de Trafico",
        native_options(),
        Box::new(move |_cc| Box::new(app)),
    )
}

pub fn launch_simulator() -> eframe::Result<()> {
    eframe::run_native(
        "Simulador",
        native_options(),
        Box::new(|_cc| Box::new(SimuladorApp::default())),
    )
}
