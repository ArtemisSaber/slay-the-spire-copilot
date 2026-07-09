use super::*;
use crate::llm::AdviceScenario;
use crate::test_utils::{load_fixture, test_locale};
use chrono::{TimeZone, Utc};
use serde_json::Value;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

const JOURNAL_LOCK_CHILD_ROOT: &str = "STS_COPILOT_JOURNAL_LOCK_CHILD_ROOT";
const JOURNAL_LOCK_CHILD_READY: &str = "STS_COPILOT_JOURNAL_LOCK_CHILD_READY";

fn wait_for_ready(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("child did not signal readiness: {}", path.display());
}

fn assert_child_waits_while_locked(child: &mut Child) {
    thread::sleep(Duration::from_millis(150));
    assert!(
        child.try_wait().unwrap().is_none(),
        "child append completed while parent lock was held"
    );
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
    let raw = load_fixture("card-reward-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, test_locale());
    let hash = state.stable_hash();
    let observation_hash = state.observation_hash();

    journal.log_state_change(&hash, &state);

    let events = read_events(journal.path().unwrap());
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
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, test_locale());
    let hash = state.stable_hash();

    journal.log_state_change(&hash, &state);
    journal.log_state_change(&hash, &state);

    let events = read_events(journal.path().unwrap());
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

    let events = read_events(journal.path().unwrap());
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

    let events = read_events(journal.path().unwrap());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "run_started");
    assert_eq!(events[0]["schema_version"], 1);
    assert_eq!(events[0]["run_id"], "test-run");
    assert!(events[0]["ts_ms"].is_number());
}

#[test]
fn journal_append_waits_for_cross_process_lock() {
    if let Ok(root) = std::env::var(JOURNAL_LOCK_CHILD_ROOT) {
        let ready = std::env::var(JOURNAL_LOCK_CHILD_READY).unwrap();
        let journal = Journal::new_at(root, "lock-test");
        std::fs::write(ready, "ready").unwrap();
        journal.log_run_started();
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let run_dir = dir.path().join("lock-test");
    std::fs::create_dir_all(&run_dir).unwrap();
    let journal_path = run_dir.join("events.jsonl");
    let ready_path = dir.path().join("child-ready");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal_path)
        .unwrap();
    fs4::FileExt::lock(&file).unwrap();

    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("journal::tests::journal_append_waits_for_cross_process_lock")
        .arg("--exact")
        .env(JOURNAL_LOCK_CHILD_ROOT, dir.path())
        .env(JOURNAL_LOCK_CHILD_READY, &ready_path)
        .spawn()
        .unwrap();

    wait_for_ready(&ready_path);
    assert_child_waits_while_locked(&mut child);
    fs4::FileExt::unlock(&file).unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "child test failed: {status}");

    let events = read_events(&journal_path);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "run_started");
}

#[test]
fn run_ended_event_is_logged_with_reason() {
    let dir = tempfile::tempdir().unwrap();
    let journal = Journal::new_at(dir.path(), "test-run");

    journal.log_run_ended("stdin_closed");

    let events = read_events(journal.path().unwrap());
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
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, test_locale());

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

    let events = read_events(journal.path().unwrap());
    assert!(events.iter().all(|event| event["schema_version"] == 1));
}

#[test]
fn journal_dedupes_by_observation_hash_not_advice_hash() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new_at(dir.path(), "test-run");
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["draw_pile"][0]["uuid"] =
        Value::String("changed-draw".into());

    let state1 = crate::state::NormalizedState::from_raw(&raw1, test_locale());
    let state2 = crate::state::NormalizedState::from_raw(&raw2, test_locale());
    let advice_hash = state1.stable_hash();
    assert_eq!(advice_hash, state2.stable_hash());
    assert_ne!(state1.observation_hash(), state2.observation_hash());

    journal.log_state_change(&advice_hash, &state1);
    journal.log_state_change(&advice_hash, &state2);

    let events = read_events(journal.path().unwrap());
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
        disable_fast_thinking: false,
        auto_play: false,
    };

    journal.log_run_started_with_config(&config);

    let events = read_events(journal.path().unwrap());
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
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, test_locale());

    journal.log_state_change(&state.stable_hash(), &state);

    let events = read_events(journal.path().unwrap());
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
    let raw = serde_json::json!({"in_game": true, "game_state": {"screen_type": "NONE"}});
    let state = crate::state::NormalizedState::from_raw(&raw, test_locale());

    journal.log_state_change(&state.stable_hash(), &state);

    let events = read_events(journal.path().unwrap());
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

    let content = std::fs::read_to_string(journal.path().unwrap()).unwrap();
    assert!(content.contains("first"));
    assert!(content.contains("second"));
}

#[test]
fn not_confirmed_until_confirm_called() {
    let dir = tempfile::tempdir().unwrap();
    let journal = Journal::new(dir.path().to_path_buf());
    assert!(!journal.is_confirmed());
    assert!(journal.path().is_none());
}

#[test]
fn confirm_creates_readable_folder_name() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new(dir.path().to_path_buf());
    let config = crate::config::Config::from_env();
    let locale = test_locale();

    journal.confirm(42, "IRONCLAD", 20, &config, locale);
    assert!(journal.is_confirmed());

    let path = journal.path().unwrap();
    let folder_name = path
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let expected_class = locale.i18n.character_display_name("IRONCLAD");
    assert!(
        folder_name.contains(expected_class),
        "folder should contain class name '{expected_class}': {folder_name}"
    );
    assert!(
        folder_name.contains("_A20"),
        "folder should contain ascension: {folder_name}"
    );
}

#[test]
fn confirm_finds_existing_unfinished_run() {
    let dir = tempfile::tempdir().unwrap();
    let locale = test_locale();
    let seed = -3047511808784702860_i64;

    let mut existing = Journal::new_at(dir.path(), "existing-run");
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, locale);
    existing.log_state_change(&state.stable_hash(), &state);

    let config = crate::config::Config::from_env();
    let mut journal = Journal::new(dir.path().to_path_buf());
    journal.confirm(seed, "IRONCLAD", 20, &config, locale);

    assert!(journal.is_confirmed());
    assert!(journal.is_continued_run());
    assert_eq!(
        journal
            .path()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap(),
        "existing-run"
    );

    let events = read_events(journal.path().unwrap());
    let continued: Vec<_> = events
        .iter()
        .filter(|e| e["event"] == "run_continued")
        .collect();
    assert_eq!(continued.len(), 1);
    assert_eq!(continued[0]["seed"], seed);
    assert_eq!(continued[0]["character"], "IRONCLAD");
}

#[test]
fn confirm_skips_ended_run() {
    let dir = tempfile::tempdir().unwrap();
    let locale = test_locale();
    let seed = -3047511808784702860_i64;

    let mut existing = Journal::new_at(dir.path(), "ended-run");
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, locale);
    existing.log_state_change(&state.stable_hash(), &state);
    existing.log_run_ended("game_over");

    let config = crate::config::Config::from_env();
    let mut journal = Journal::new(dir.path().to_path_buf());
    journal.confirm(seed, "IRONCLAD", 20, &config, locale);

    assert!(!journal.is_continued_run());
    assert_ne!(
        journal
            .path()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap(),
        "ended-run"
    );
}

#[test]
fn confirm_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new(dir.path().to_path_buf());
    let config = crate::config::Config::from_env();
    let locale = test_locale();

    journal.confirm(99, "DEFECT", 5, &config, locale);
    let path1 = journal.path().unwrap().to_path_buf();
    let count1 = read_events(&path1).len();

    journal.confirm(99, "DEFECT", 5, &config, locale);
    let path2 = journal.path().unwrap().to_path_buf();
    let count2 = read_events(&path2).len();

    assert_eq!(path1, path2);
    assert_eq!(count1, count2);
}

#[test]
fn run_continued_includes_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let locale = test_locale();
    let seed = -3047511808784702860_i64;

    let mut existing = Journal::new_at(dir.path(), "existing-run");
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, locale);
    existing.log_state_change(&state.stable_hash(), &state);

    let config = crate::config::Config::from_env();
    let mut journal = Journal::new(dir.path().to_path_buf());
    journal.confirm(seed, "IRONCLAD", 20, &config, locale);

    let events = read_events(journal.path().unwrap());
    let continued = events
        .iter()
        .find(|e| e["event"] == "run_continued")
        .unwrap();
    assert_eq!(continued["seed"], seed);
    assert_eq!(continued["character"], "IRONCLAD");
    assert_eq!(continued["ascension_level"], 20);
    assert!(!continued["app_version"].as_str().unwrap().is_empty());
}

#[test]
fn format_datetime_produces_valid_format() {
    let ts = timestamp_ms();
    let result = format_datetime(ts);
    assert!(
        result
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == '_'),
        "expected YYYY-MM-DD_HH-MM, got {result}"
    );
    assert_eq!(result.len(), 16);
}

#[test]
fn format_datetime_uses_local_time() {
    let ts_ms = timestamp_ms();
    let utc_str = Utc
        .timestamp_millis_opt(ts_ms as i64)
        .single()
        .map(|dt| dt.format("%Y-%m-%d_%H-%M").to_string());
    let local = format_datetime(ts_ms);
    if utc_str.as_ref() == Some(&local) {
        return;
    }
    assert_ne!(
        utc_str.unwrap(),
        local,
        "should differ from UTC when timezone offset is non-zero"
    );
}
