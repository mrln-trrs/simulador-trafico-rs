pub mod triangulation;
pub mod collisions;
pub mod distance;
pub mod angles;
pub mod snap;

pub use triangulation::triangulate_polygon;
pub use collisions::{
    road_collides_with_obstacles, building_collides_with_roads,
    is_point_inside_any_other_road, road_collides_with_roads, road_collides_with_any_road,
};
pub use distance::{dist_to_segment, point_in_polygon};
pub use angles::compute_interior_angles;
pub use snap::snap_to_elements;
