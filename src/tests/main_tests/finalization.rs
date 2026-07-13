use super::*;

#[tokio::test]
async fn finalize_run_writes_postmortem_report() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = crate::journal::Journal::new_at(dir.path(), "run-1");
    let config = crate::config::Config::from_env();
    let raw = serde_json::from_str::<serde_json::Value>(include_str!(
        "../../../tests/fixtures/combat-state.json"
    ))
    .unwrap();
    let state = crate::state::NormalizedState::from_raw(&raw, crate::test_utils::test_locale());
    let provider = crate::llm::LlmProvider::Mock;
    let mut finalized = false;

    journal.log_run_started_with_config(&config);
    journal.log_state_change(&state.stable_hash(), &state);
    finalize_run_once(
        &journal,
        &provider,
        "game_over",
        &mut finalized,
        crate::test_utils::test_locale(),
    )
    .await;

    assert!(finalized);
    let report = std::fs::read_to_string(dir.path().join("run-1").join("postmortem.md")).unwrap();
    assert!(report.contains("# 本局复盘"));
    let events = std::fs::read_to_string(journal.path().unwrap()).unwrap();
    assert!(events.contains("\"event\":\"run_ended\""));
    assert!(events.contains("\"reason\":\"game_over\""));
}

#[tokio::test]
async fn finalize_run_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let journal = crate::journal::Journal::new_at(dir.path(), "run-1");
    let config = crate::config::Config::from_env();
    let provider = crate::llm::LlmProvider::Mock;
    let mut finalized = false;

    journal.log_run_started_with_config(&config);
    finalize_run_once(
        &journal,
        &provider,
        "game_over",
        &mut finalized,
        crate::test_utils::test_locale(),
    )
    .await;
    finalize_run_once(
        &journal,
        &provider,
        "stdin_closed",
        &mut finalized,
        crate::test_utils::test_locale(),
    )
    .await;

    let events = std::fs::read_to_string(journal.path().unwrap()).unwrap();
    assert_eq!(events.matches("\"event\":\"run_ended\"").count(), 1);
    assert!(events.contains("\"reason\":\"game_over\""));
}
