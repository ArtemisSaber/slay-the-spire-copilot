use super::*;
use serde_json::Value;

fn load_i18n() -> crate::i18n::I18n {
    crate::i18n::I18n::load()
}

fn load_fixture(name: &str) -> Value {
    let path = format!("tests/fixtures/{name}");
    let content = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn read_events(path: &Path) -> Vec<Value> {
    let content = std::fs::read_to_string(path).unwrap();
    content
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn state_changes_are_logged_as_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new_at(dir.path(), "test-run");
    let i18n = load_i18n();
    let raw = load_fixture("card-reward-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, &i18n);
    let hash = state.stable_hash();

    journal.log_state_change(&hash, &state);

    let events = read_events(journal.path());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "state_changed");
    assert_eq!(events[0]["run_id"], "test-run");
    assert_eq!(events[0]["hash"], hash);
    assert_eq!(events[0]["screen_type"], "CARD_REWARD");
    assert_eq!(events[0]["normalized"]["floor"], 14);
}

#[test]
fn repeated_state_hashes_are_deduplicated() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new_at(dir.path(), "test-run");
    let i18n = load_i18n();
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, &i18n);
    let hash = state.stable_hash();

    journal.log_state_change(&hash, &state);
    journal.log_state_change(&hash, &state);

    let events = read_events(journal.path());
    assert_eq!(events.len(), 1);
}

#[test]
fn advice_events_are_logged() {
    let dir = tempfile::tempdir().unwrap();
    let journal = Journal::new_at(dir.path(), "test-run");

    journal.log_advice("abc123", Effort::Heavy, "prompt text", "advice text");

    let events = read_events(journal.path());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "advice");
    assert_eq!(events[0]["state_hash"], "abc123");
    assert_eq!(events[0]["effort"], "heavy");
    assert_eq!(events[0]["prompt"], "prompt text");
    assert_eq!(events[0]["advice"], "advice text");
}

#[test]
fn run_started_event_is_logged() {
    let dir = tempfile::tempdir().unwrap();
    let journal = Journal::new_at(dir.path(), "test-run");

    journal.log_run_started();

    let events = read_events(journal.path());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "run_started");
    assert_eq!(events[0]["run_id"], "test-run");
    assert!(events[0]["ts_ms"].is_number());
}

#[test]
fn run_ended_event_is_logged_with_reason() {
    let dir = tempfile::tempdir().unwrap();
    let journal = Journal::new_at(dir.path(), "test-run");

    journal.log_run_ended("stdin_closed");

    let events = read_events(journal.path());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "run_ended");
    assert_eq!(events[0]["run_id"], "test-run");
    assert_eq!(events[0]["reason"], "stdin_closed");
    assert!(events[0]["ts_ms"].is_number());
}
