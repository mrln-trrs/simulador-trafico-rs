use crate::model::{validate_scenario, Scenario, ValidationIssue};

pub fn validate_engine_scenario(scenario: &Scenario) -> Result<(), Vec<ValidationIssue>> {
    validate_scenario(scenario)
}
