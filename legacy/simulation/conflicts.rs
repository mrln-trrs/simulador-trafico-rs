use crate::model::ids::VehicleId;

pub fn resolve_vehicle_order(mut ids: Vec<VehicleId>) -> Vec<VehicleId> {
    ids.sort();
    ids
}
