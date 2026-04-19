use crate::model::VehicleSpawn;

pub fn build_demo_schedule() -> Vec<VehicleSpawn> {
    vec![
        VehicleSpawn::new(0, "Vehiculo 1", 1, 4),
        VehicleSpawn::new(1, "Vehiculo 2", 1, 4),
        VehicleSpawn::new(2, "Vehiculo 3", 1, 4),
        VehicleSpawn::new(3, "Vehiculo 4", 1, 3),
        VehicleSpawn::new(4, "Vehiculo 5", 1, 4),
        VehicleSpawn::new(5, "Vehiculo 6", 1, 4),
        VehicleSpawn::new(6, "Vehiculo 7", 1, 4),
    ]
}