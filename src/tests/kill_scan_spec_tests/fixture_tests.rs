use crate::combat::{can_end_fight, find_kill_sequence};
use crate::locales::Locale;
use crate::state::NormalizedState;

fn load_fixture(name: &str) -> serde_json::Value {
    let path = format!("tests/fixtures/{name}");
    let content = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn fixture_list() -> Vec<String> {
    let content = std::fs::read_to_string("tests/fixtures/kill-scan-fixtures.json").unwrap();
    serde_json::from_str(&content).unwrap()
}

fn fixture_by_line(line: &str) -> String {
    fixture_list()
        .into_iter()
        .find(|n| n.contains(line))
        .expect("fixture not found")
}

fn run_fixture_list() -> Vec<serde_json::Value> {
    let content = std::fs::read_to_string("tests/fixtures/kill-scan-run-fixtures.json").unwrap();
    serde_json::from_str(&content).unwrap()
}

fn edge_fixture_list() -> Vec<serde_json::Value> {
    let content = std::fs::read_to_string("tests/fixtures/kill-scan-edge-fixtures.json").unwrap();
    serde_json::from_str(&content).unwrap()
}

#[path = "fixture_tests/edge_regressions.rs"]
mod edge_regressions;
#[path = "fixture_tests/run_fixtures.rs"]
mod run_fixtures;
#[path = "fixture_tests/standard.rs"]
mod standard;
#[path = "fixture_tests/targeting.rs"]
mod targeting;
#[path = "fixture_tests/time_warp.rs"]
mod time_warp;
