use crate::ui::screens::simulator::SimuladorApp;

fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([960.0, 640.0]),
        centered: true,
        persist_window: false,
        ..Default::default()
    }
}

pub fn launch_simulator() -> eframe::Result<()> {
    eframe::run_native(
        "Simulador",
        native_options(),
        Box::new(|cc| Box::new(SimuladorApp::new(cc))),
    )
}
