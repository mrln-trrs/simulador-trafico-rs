use crate::integration::codec::{from_json, from_toml, to_json, to_toml};
use crate::model::Scenario;

pub fn serialize_scenario_json(scenario: &Scenario) -> Result<String, String> {
    to_json(scenario).map_err(|error| error.to_string())
}

pub fn serialize_scenario_toml(scenario: &Scenario) -> Result<String, String> {
    to_toml(scenario).map_err(|error| error.to_string())
}

pub fn deserialize_scenario_json(content: &str) -> Result<Scenario, String> {
    from_json(content).map_err(|error| error.to_string())
}

pub fn deserialize_scenario_toml(content: &str) -> Result<Scenario, String> {
    from_toml(content).map_err(|error| error.to_string())
}
