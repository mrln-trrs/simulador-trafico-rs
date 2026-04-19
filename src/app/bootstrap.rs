#[allow(unused_imports)]
use crate::app::composition::build_app;

pub fn run() -> eframe::Result<()> {
    // Por ahora no ejecutamos el simulador.
    // let app = build_app();
    // app.run()
    crate::app::runtime::launch_simulator()
}
