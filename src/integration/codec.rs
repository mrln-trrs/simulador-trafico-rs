use serde::{de::DeserializeOwned, Serialize};

pub fn to_json<T: Serialize>(value: &T) -> serde_json::Result<String> {
    serde_json::to_string_pretty(value)
}

pub fn from_json<T: DeserializeOwned>(json: &str) -> serde_json::Result<T> {
    serde_json::from_str(json)
}

pub fn to_toml<T: Serialize>(value: &T) -> Result<String, toml::ser::Error> {
    toml::to_string_pretty(value)
}

pub fn from_toml<T: DeserializeOwned>(text: &str) -> Result<T, toml::de::Error> {
    toml::from_str(text)
}
