use crate::presentation::app_shell::SimulatorApp;
use crate::presentation::simulator_shell::SimuladorApp;

pub fn launch(app: SimulatorApp) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Simulador de Trafico",
        native_options,
        Box::new(move |_cc| Box::new(app)),
    )
}

pub fn launch_simulator() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Simulador",
        native_options,
        Box::new(|_cc| Box::new(SimuladorApp::default())),
    )
}
