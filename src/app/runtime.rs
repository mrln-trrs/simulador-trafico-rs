use crate::presentation::app_shell::SimulatorApp;

pub fn launch(app: SimulatorApp) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Simulador de Trafico",
        native_options,
        Box::new(move |_cc| Box::new(app)),
    )
}
