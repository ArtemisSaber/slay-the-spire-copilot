use super::*;
use crate::llm::AdviceScenario;
use crate::test_utils::{load_fixture, load_i18n};
use serde_json::Value;

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
    let observation_hash = state.observation_hash();

    journal.log_state_change(&hash, &state);

    let events = read_events(journal.path());
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["event"], "run_metadata");
    assert_eq!(events[1]["event"], "state_changed");
    assert_eq!(events[1]["run_id"], "test-run");
    assert_eq!(events[1]["hash"], hash);
    assert_eq!(events[1]["advice_hash"], hash);
    assert_eq!(events[1]["observation_hash"], observation_hash);
    assert_eq!(events[1]["screen_type"], "CARD_REWARD");
    assert_eq!(events[1]["normalized"]["floor"], 14);
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
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["event"], "run_metadata");
    assert_eq!(events[1]["event"], "state_changed");
}

#[test]
fn advice_events_are_logged() {
    let dir = tempfile::tempdir().unwrap();
    let journal = Journal::new_at(dir.path(), "test-run");

    journal.log_advice(
        "abc123",
        Effort::Heavy,
        AdviceScenario::CardReward,
        "prompt text",
        "advice text",
    );

    let events = read_events(journal.path());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "advice");
    assert_eq!(events[0]["schema_version"], 1);
    assert_eq!(events[0]["state_hash"], "abc123");
    assert_eq!(events[0]["advice_hash"], "abc123");
    assert_eq!(events[0]["effort"], "heavy");
    assert_eq!(events[0]["scenario"], "card_reward");
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
    assert_eq!(events[0]["schema_version"], 1);
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
    assert_eq!(events[0]["schema_version"], 1);
    assert_eq!(events[0]["run_id"], "test-run");
    assert_eq!(events[0]["reason"], "stdin_closed");
    assert!(events[0]["ts_ms"].is_number());
}

#[test]
fn journal_events_include_schema_version() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new_at(dir.path(), "test-run");
    let i18n = load_i18n();
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, &i18n);

    journal.log_run_started();
    journal.log_state_change(&state.stable_hash(), &state);
    journal.log_advice(
        &state.stable_hash(),
        Effort::Fast,
        AdviceScenario::CombatEntry,
        "prompt",
        "advice",
    );
    journal.log_run_ended("stdin_closed");

    let events = read_events(journal.path());
    assert!(events.iter().all(|event| event["schema_version"] == 1));
}

#[test]
fn journal_dedupes_by_observation_hash_not_advice_hash() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new_at(dir.path(), "test-run");
    let i18n = load_i18n();
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["draw_pile"][0]["uuid"] =
        Value::String("changed-draw".into());

    let state1 = crate::state::NormalizedState::from_raw(&raw1, &i18n);
    let state2 = crate::state::NormalizedState::from_raw(&raw2, &i18n);
    let advice_hash = state1.stable_hash();
    assert_eq!(advice_hash, state2.stable_hash());
    assert_ne!(state1.observation_hash(), state2.observation_hash());

    journal.log_state_change(&advice_hash, &state1);
    journal.log_state_change(&advice_hash, &state2);

    let events = read_events(journal.path());
    let state_events: Vec<&Value> = events
        .iter()
        .filter(|event| event["event"] == "state_changed")
        .collect();
    assert_eq!(state_events.len(), 2);
}

#[test]
fn run_started_includes_provider_and_models() {
    let dir = tempfile::tempdir().unwrap();
    let journal = Journal::new_at(dir.path(), "test-run");
    let config = crate::config::Config {
        provider: "openai-compatible".into(),
        base_url: Some("https://api.example.com".into()),
        api_key: Some("sk-test".into()),
        model_fast: "fast-model".into(),
        model_medium: "medium-model".into(),
        model_heavy: "heavy-model".into(),
        max_tokens_fast: 100,
        max_tokens_medium: 200,
        max_tokens_heavy: 300,
        temperature: 0.2,
    };

    journal.log_run_started_with_config(&config);

    let events = read_events(journal.path());
    assert_eq!(events[0]["event"], "run_started");
    assert_eq!(events[0]["provider"], "openai-compatible");
    assert_eq!(events[0]["model_fast"], "fast-model");
    assert_eq!(events[0]["model_medium"], "medium-model");
    assert_eq!(events[0]["model_heavy"], "heavy-model");
    assert_eq!(events[0]["app_version"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn first_observed_state_can_update_run_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new_at(dir.path(), "test-run");
    let i18n = load_i18n();
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, &i18n);

    journal.log_state_change(&state.stable_hash(), &state);

    let events = read_events(journal.path());
    assert_eq!(events[0]["event"], "run_metadata");
    assert_eq!(events[0]["character"], "IRONCLAD");
    assert_eq!(events[0]["ascension_level"], 20);
    assert_eq!(events[0]["seed"], -3047511808784702860_i64);
    assert_eq!(events[0]["floor"], 1);
}

#[test]
fn metadata_fields_are_null_when_missing() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new_at(dir.path(), "test-run");
    let i18n = load_i18n();
    let raw = serde_json::json!({"in_game": true, "game_state": {"screen_type": "NONE"}});
    let state = crate::state::NormalizedState::from_raw(&raw, &i18n);

    journal.log_state_change(&state.stable_hash(), &state);

    let events = read_events(journal.path());
    assert_eq!(events[0]["event"], "run_metadata");
    assert!(events[0]["character"].is_null());
    assert!(events[0]["ascension_level"].is_null());
    assert!(events[0]["seed"].is_null());
}

#[test]
fn advice_events_are_written_to_journal() {
    let dir = tempfile::tempdir().unwrap();
    let journal = Journal::new_at(dir.path(), "test-run");
    journal.log_advice(
        "hash1",
        Effort::Fast,
        AdviceScenario::Generic,
        "prompt 1",
        "first",
    );
    journal.log_advice(
        "hash2",
        Effort::Heavy,
        AdviceScenario::CardReward,
        "prompt 2",
        "second",
    );

    let content = std::fs::read_to_string(journal.path()).unwrap();
    assert!(content.contains("first"));
    assert!(content.contains("second"));
}
