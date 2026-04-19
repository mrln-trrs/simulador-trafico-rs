use crate::integration::codec::{from_json, from_toml};
use crate::model::Scenario;
use std::fs;
use std::path::Path;

pub fn load_scenario(path: impl AsRef<Path>) -> Result<Scenario, String> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => from_toml(&content).map_err(|error| error.to_string()),
        _ => from_json(&content).map_err(|error| error.to_string()),
    }
}
