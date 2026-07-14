use super::*;

fn find_value(assignments: &[EnvAssignment], key: &str) -> String {
    assignments
        .iter()
        .find(|a| a.key == key)
        .map(|a| a.value.clone())
        .unwrap_or_default()
}

#[path = "setup_wizard_tests/assignments.rs"]
mod assignments;
#[path = "setup_wizard_tests/env_helpers.rs"]
mod env_helpers;
#[path = "setup_wizard_tests/metadata.rs"]
mod metadata;
#[path = "setup_wizard_tests/parse_values.rs"]
mod parse_values;
#[path = "setup_wizard_tests/setup.rs"]
mod setup;
