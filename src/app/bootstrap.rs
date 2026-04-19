use crate::app::composition::build_app;

pub fn run() -> eframe::Result<()> {
    build_app().run()
}
