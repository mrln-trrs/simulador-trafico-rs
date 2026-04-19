use crate::generation::fixtures;
use crate::presentation::app_shell::SimulatorApp;

pub struct AppRuntime {
    app: SimulatorApp,
}

impl AppRuntime {
    pub fn run(self) -> eframe::Result<()> {
        crate::app::runtime::launch(self.app)
    }
}

pub fn build_app() -> AppRuntime {
    let scenario = fixtures::demo_scenario();
    AppRuntime {
        app: SimulatorApp::new(scenario),
    }
}
