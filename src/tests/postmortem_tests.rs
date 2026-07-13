use super::*;
use crate::state::NormalizedState;
use serde_json::{Value, json};

fn event_line(value: Value) -> String {
    serde_json::to_string(&value).unwrap()
}

fn normalized_fixture(name: &str) -> Value {
    let content = std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap();
    let raw: Value = serde_json::from_str(&content).unwrap();
    let state = NormalizedState::from_raw(&raw, crate::test_utils::test_locale());
    serde_json::to_value(state).unwrap()
}

fn state_event(state: Value) -> String {
    event_line(json!({
        "schema_version": 1,
        "event": "state_changed",
        "advice_hash": "hash",
        "observation_hash": "obs",
        "normalized": state
    }))
}

#[path = "postmortem_tests/basics.rs"]
mod basics;
#[path = "postmortem_tests/combat_reports.rs"]
mod combat_reports;
#[path = "postmortem_tests/combat_tracking.rs"]
mod combat_tracking;
#[path = "postmortem_tests/data_edges.rs"]
mod data_edges;
#[path = "postmortem_tests/helpers.rs"]
mod helpers;
#[path = "postmortem_tests/summary.rs"]
mod summary;
