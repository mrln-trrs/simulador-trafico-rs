use crate::model::Scenario;
use crate::persistence::serializer::{
    deserialize_scenario_json, deserialize_scenario_toml, serialize_scenario_json,
    serialize_scenario_toml,
};
use std::fs;
use std::path::Path;

pub fn save_scenario(path: impl AsRef<Path>, scenario: &Scenario) -> Result<(), String> {
    let path = path.as_ref();
    let serialized = match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => serialize_scenario_toml(scenario)?,
        _ => serialize_scenario_json(scenario)?,
    };
    fs::write(path, serialized).map_err(|error| error.to_string())
}

pub fn load_scenario(path: impl AsRef<Path>) -> Result<Scenario, String> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => deserialize_scenario_toml(&content),
        _ => deserialize_scenario_json(&content),
    }
}
