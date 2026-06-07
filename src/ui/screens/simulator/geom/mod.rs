pub mod triangulation;
pub mod collisions;
pub mod distance;
pub mod angles;

pub use triangulation::triangulate_polygon;
pub use collisions::{
    road_collides_with_obstacles, building_collides_with_roads,
    is_point_inside_any_other_road,
};
pub use distance::{dist_to_segment, point_in_polygon};
pub use angles::compute_interior_angles;
