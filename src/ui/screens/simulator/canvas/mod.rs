pub(crate) mod grid;
pub(crate) mod render_cache;
pub(crate) mod viewport;

pub(crate) const BASE_GRID_10M_ZOOM_THRESHOLD: f32 = 28.0;
pub(crate) const SUBGRID_10CM_ZOOM_THRESHOLD: f32 = 80.0;
pub(crate) const SUBGRID_1CM_ZOOM_THRESHOLD: f32 = 200.0;