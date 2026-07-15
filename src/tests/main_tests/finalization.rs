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

#[tokio::test]
async fn finalization_publishes_plain_learning_status_to_overlay() {
    let dir = tempfile::tempdir().unwrap();
    let journal_root = dir.path().join("runs");
    let mut journal = crate::journal::Journal::new_at(&journal_root, "run-debug");
    let config = crate::config::Config::from_map(&std::collections::HashMap::from([(
        "MEMORY_MODE",
        "collect",
    )]));
    let state = crate::state::NormalizedState {
        screen_type: Some(crate::state::ScreenType::GameOver),
        character: Some("IRONCLAD".into()),
        seed: Some(7),
        ascension_level: Some(-15),
        floor: Some(51),
        current_hp: Some(40),
        max_hp: Some(80),
        ..crate::state::NormalizedState::default()
    };
    let mut learning = crate::learning::session::LearningSession::new(
        config.memory.clone(),
        crate::learning::store::KnowledgeStore::new(dir.path().join("knowledge")),
        crate::learning::snapshot::KnowledgeSnapshot::build(vec![], &[]).unwrap(),
        crate::learning::session::SessionProvenance {
            locale: "en".into(),
            model_profile_sha256: "automatic-model-profile".into(),
            compatibility_sha256: "automatic-compatibility".into(),
            rules_sha256: "rules".into(),
            synthetic_input: false,
        },
    );
    let overlay_path = dir.path().join("output/overlay.json");
    let provider = crate::llm::LlmProvider::Mock;
    let mut finalized = false;

    journal.log_run_started_with_config(&config);
    journal.log_state_change(&state.stable_hash(), &state);
    learning.observe(&state);
    crate::app::finalization::finalize_run_once_with_learning(
        &journal,
        &provider,
        "game_over",
        &mut finalized,
        crate::test_utils::test_locale(),
        &mut learning,
        &overlay_path,
    )
    .await;

    let overlay: serde_json::Value =
        serde_json::from_slice(&std::fs::read(overlay_path).unwrap()).unwrap();
    assert_eq!(overlay["learning"]["mode"], "collect");
    assert_eq!(overlay["learning"]["last_run"]["accepted"], false);
    assert_eq!(overlay["learning"]["last_run"]["run_kind"], "debug_flow");
    assert_eq!(
        overlay["learning"]["last_run"]["reasons"][0],
        "debug_ascension"
    );
    assert!(overlay["learning"].get("compatibility_sha256").is_none());
}

#[tokio::test]
async fn eligible_finalization_creates_a_lesson_from_a_json_wrapped_report() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = crate::journal::Journal::new_at(dir.path().join("runs"), "run-learn");
    let config = crate::config::Config::from_map(&std::collections::HashMap::from([(
        "MEMORY_MODE",
        "collect",
    )]));
    let snapshot = crate::learning::snapshot::KnowledgeSnapshot::build(vec![], &[]).unwrap();
    let mut learning = crate::learning::session::LearningSession::new(
        config.memory.clone(),
        crate::learning::store::KnowledgeStore::new(dir.path().join("knowledge")),
        snapshot,
        crate::learning::session::SessionProvenance {
            locale: "en".into(),
            model_profile_sha256: "automatic-model-profile".into(),
            compatibility_sha256: "automatic-compatibility".into(),
            rules_sha256: "rules".into(),
            synthetic_input: false,
        },
    );
    let mut combat = crate::state::NormalizedState {
        screen_type: Some(crate::state::ScreenType::None),
        character: Some("IRONCLAD".into()),
        seed: Some(7),
        ascension_level: Some(0),
        floor: Some(8),
        current_hp: Some(40),
        max_hp: Some(80),
        energy: Some(3),
        block: Some(0),
        monsters: vec![crate::state::MonsterInfo {
            monster_id: Some("GremlinNob".into()),
            index: 0,
            current_hp: Some(60),
            max_hp: Some(100),
            intent: Some("ATTACK".into()),
            damage: Some(10),
            hits: Some(1),
            ..crate::state::MonsterInfo::default()
        }],
        incoming_damage: 10,
        turn_number: Some(1),
        ..crate::state::NormalizedState::default()
    };
    learning.observe(&combat);
    let prepared = learning.prepare(&combat, &[]).unwrap();
    learning.record_executed(
        "run-learn",
        &combat,
        &crate::learning::session::ExecutedPlan {
            action: crate::autoplay::action::AutoPlayAction::End,
            source: crate::learning::telemetry::DecisionSource::Llm,
            selected_action_id: "combat:end".into(),
            available_semantic_actions: vec![crate::learning::action::SemanticAction::EndTurn],
            ranked_suggestions: vec![],
            memory_ids_used: vec![],
        },
        &prepared,
    );
    combat.turn_number = Some(2);
    combat.current_hp = Some(38);
    learning.observe(&combat);
    combat.screen_type = Some(crate::state::ScreenType::CombatReward);
    combat.monsters.clear();
    learning.observe(&combat);
    combat.screen_type = Some(crate::state::ScreenType::GameOver);
    combat.floor = Some(51);
    learning.observe(&combat);

    journal.log_run_started_with_config(&config);
    journal.log_state_change(&combat.stable_hash(), &combat);
    let overlay_path = dir.path().join("output/overlay.json");
    let mut finalized = false;
    crate::app::finalization::finalize_run_once_with_learning(
        &journal,
        &crate::llm::LlmProvider::Mock,
        "game_over",
        &mut finalized,
        crate::test_utils::test_locale(),
        &mut learning,
        &overlay_path,
    )
    .await;

    assert_eq!(learning.status().case_count, 1);
    assert_eq!(learning.status().lesson_count, 1);
    let events = std::fs::read_to_string(journal.path().unwrap()).unwrap();
    assert!(events.contains("\"accepted_lessons\":1"));
    let report = std::fs::read_to_string(dir.path().join("runs/run-learn/postmortem.md")).unwrap();
    assert!(report.contains("# Mock Review"));
    assert!(!report.trim_start().starts_with('{'));
}
