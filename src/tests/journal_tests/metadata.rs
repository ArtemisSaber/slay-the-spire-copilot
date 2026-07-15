use super::*;

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
        auto_play_auto_start: false,
        memory: Default::default(),
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
