use super::support::decision_case;
use crate::autoplay::action::AutoPlayAction;
use crate::learning::action::SemanticAction;
use crate::learning::config::{MemoryConfig, MemoryMode};
use crate::learning::eligibility::RunKind;
use crate::learning::session::{ExecutedPlan, LearningSession, SessionProvenance};
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::store::KnowledgeStore;
use crate::learning::telemetry::DecisionSource;
use crate::state::{CardInfo, MonsterInfo, NormalizedState, ScreenType};

fn combat_state(ascension: i64, seed: i64) -> NormalizedState {
    NormalizedState {
        screen_type: Some(ScreenType::None),
        character: Some("IRONCLAD".into()),
        seed: Some(seed),
        ascension_level: Some(ascension),
        floor: Some(8),
        current_hp: Some(40),
        max_hp: Some(80),
        energy: Some(3),
        block: Some(0),
        hand: vec![CardInfo {
            id: "Bash".into(),
            name: "Bash".into(),
            cost: 2,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: Some("local".into()),
            description: String::new(),
            price: None,
            playable: true,
            has_target: true,
        }],
        monsters: vec![MonsterInfo {
            monster_id: Some("GremlinNob".into()),
            index: 0,
            current_hp: Some(60),
            max_hp: Some(100),
            intent: Some("ATTACK".into()),
            damage: Some(10),
            hits: Some(1),
            ..MonsterInfo::default()
        }],
        incoming_damage: 10,
        turn_number: Some(1),
        ..NormalizedState::default()
    }
}

fn config(mode: MemoryMode) -> MemoryConfig {
    MemoryConfig {
        mode,
        ..MemoryConfig::default()
    }
}

fn provenance(synthetic: bool) -> SessionProvenance {
    SessionProvenance {
        locale: "en".into(),
        model_profile_sha256: "model".into(),
        compatibility_sha256: "mods".into(),
        rules_sha256: "rules".into(),
        synthetic_input: synthetic,
    }
}

fn plan() -> ExecutedPlan {
    ExecutedPlan {
        action: AutoPlayAction::End,
        source: DecisionSource::Llm,
        selected_action_id: "combat:end".into(),
        available_semantic_actions: vec![SemanticAction::EndTurn],
        ranked_suggestions: vec![],
        memory_ids_used: vec![],
    }
}

fn finish_state(mut state: NormalizedState, screen: ScreenType) -> NormalizedState {
    state.screen_type = Some(screen);
    state.monsters.clear();
    state.current_hp = Some(35);
    state.floor = Some(51);
    state
}

#[test]
fn legitimate_completed_run_commits_cases_after_finalization_only() {
    let temp = tempfile::tempdir().unwrap();
    let store = KnowledgeStore::new(temp.path());
    let snapshot = KnowledgeSnapshot::build(vec![], &[]).unwrap();
    let mut session = LearningSession::new(
        config(MemoryMode::Collect),
        store,
        snapshot,
        provenance(false),
    );
    let state = combat_state(20, 1);
    session.observe(&state);
    let prepared = session.prepare(&state, &["damage".into()]).unwrap();
    let _ = session.record_executed("run-1", &state, &plan(), &prepared);
    assert!(!temp.path().join("cases.jsonl").exists());

    let reward = finish_state(state.clone(), ScreenType::CombatReward);
    session.observe(&reward);
    let game_over = finish_state(reward, ScreenType::GameOver);
    session.observe(&game_over);
    let summary = session.finalize("game_over").unwrap();

    assert_eq!(summary.eligibility.run_kind, RunKind::Legitimate);
    assert_eq!(summary.appended_cases, 1);
    assert_eq!(session.snapshot().cases.len(), 1);
    assert!(temp.path().join("cases.jsonl").is_file());
    let status: serde_json::Value =
        serde_json::from_slice(&std::fs::read(temp.path().join("status.json")).unwrap()).unwrap();
    assert_eq!(status["mode"], "collect");
    assert_eq!(status["case_count"], 1);
    assert_eq!(status["last_run_eligibility"]["run_kind"], "legitimate");
    let user_status = session.status();
    assert_eq!(user_status.mode, MemoryMode::Collect);
    assert_eq!(user_status.case_count, 1);
    assert_eq!(user_status.lesson_count, 0);
    assert!(user_status.last_run.unwrap().accepted);
}

#[test]
fn debug_ascension_victory_is_observed_but_never_committed() {
    let temp = tempfile::tempdir().unwrap();
    let store = KnowledgeStore::new(temp.path());
    let snapshot = KnowledgeSnapshot::build(vec![], &[]).unwrap();
    let mut session = LearningSession::new(
        config(MemoryMode::Collect),
        store,
        snapshot,
        provenance(false),
    );
    let state = combat_state(-15, 1);
    session.observe(&state);
    assert!(session.prepare(&state, &[]).is_none());
    let game_over = finish_state(state, ScreenType::GameOver);
    session.observe(&game_over);
    let summary = session.finalize("game_over").unwrap();

    assert_eq!(summary.eligibility.run_kind, RunKind::DebugFlow);
    assert_eq!(summary.appended_cases, 0);
    let last_run = session.status().last_run.unwrap();
    assert!(!last_run.accepted);
    assert_eq!(last_run.run_kind, RunKind::DebugFlow);
    assert!(!temp.path().join("cases.jsonl").exists());
}

#[test]
fn off_mode_is_behaviorally_empty() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = LearningSession::new(
        config(MemoryMode::Off),
        KnowledgeStore::new(temp.path()),
        KnowledgeSnapshot::build(vec![], &[]).unwrap(),
        provenance(false),
    );
    let state = combat_state(20, 1);
    session.observe(&state);
    assert!(session.prepare(&state, &[]).is_none());
}

#[test]
fn synthetic_run_cannot_commit_even_with_valid_gameplay() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = LearningSession::new(
        config(MemoryMode::Collect),
        KnowledgeStore::new(temp.path()),
        KnowledgeSnapshot::build(vec![], &[]).unwrap(),
        provenance(true),
    );
    let state = combat_state(20, 1);
    session.observe(&state);
    let prepared = session.prepare(&state, &[]).unwrap();
    let _ = session.record_executed("run-1", &state, &plan(), &prepared);
    session.observe(&finish_state(state, ScreenType::GameOver));

    let summary = session.finalize("game_over").unwrap();
    assert_eq!(summary.run_kind(), RunKind::Synthetic);
    assert_eq!(summary.appended_cases, 0);
}

#[test]
fn executed_decision_emits_audit_event_and_failed_command_is_discarded() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = LearningSession::new(
        config(MemoryMode::Collect),
        KnowledgeStore::new(temp.path()),
        KnowledgeSnapshot::build(vec![], &[]).unwrap(),
        provenance(false),
    );
    let state = combat_state(20, 7);
    session.observe(&state);
    let prepared = session.prepare(&state, &[]).unwrap();

    let event = session
        .record_executed("run-7", &state, &plan(), &prepared)
        .unwrap();
    assert_eq!(event["event"], "autoplay_decision_proposed");
    assert_eq!(event["selected_semantic_action"]["kind"], "end_turn");
    assert!(session.discard_last_execution());

    session.observe(&finish_state(state, ScreenType::GameOver));
    let summary = session.finalize("game_over").unwrap();
    assert_eq!(summary.eligibility.run_kind, RunKind::Legitimate);
    assert_eq!(summary.appended_cases, 0);
}

#[test]
fn continued_run_restores_its_first_recorded_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let store = KnowledgeStore::new(temp.path().join("knowledge"));
    let original = store.rebuild(&[]).unwrap();
    store
        .append_cases(&[decision_case("older-run", 1)])
        .unwrap();
    let current = store.rebuild(&[]).unwrap();
    assert_ne!(original.snapshot_id, current.snapshot_id);
    let mut session =
        LearningSession::new(config(MemoryMode::On), store, current, provenance(false));
    let journal = temp.path().join("events.jsonl");
    std::fs::write(
        &journal,
        format!(
            "{{\"event\":\"autoplay_decision_proposed\",\"knowledge_snapshot_id\":\"{}\"}}\n",
            original.snapshot_id
        ),
    )
    .unwrap();

    assert!(session.pin_snapshot_from_journal(&journal).unwrap());
    assert_eq!(session.snapshot().snapshot_id, original.snapshot_id);
}

#[test]
fn shadow_retrieval_is_logged_but_does_not_mark_case_as_memory_exposed() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = LearningSession::new(
        config(MemoryMode::Shadow),
        KnowledgeStore::new(temp.path()),
        KnowledgeSnapshot::build(vec![], &[]).unwrap(),
        provenance(false),
    );
    let state = combat_state(20, 9);
    session.observe(&state);
    let mut prepared = session.prepare(&state, &[]).unwrap();
    prepared.retrieved_memory_ids = vec!["lesson:shadow".into()];
    prepared.exposed_memory_ids = vec![];
    let event = session
        .record_executed("run-9", &state, &plan(), &prepared)
        .unwrap();
    assert_eq!(event["retrieved_memory_ids"][0], "lesson:shadow");
    session.observe(&finish_state(state, ScreenType::GameOver));
    session.finalize("game_over").unwrap();

    assert!(session.snapshot().cases[0].retrieved_memory_ids.is_empty());
}
