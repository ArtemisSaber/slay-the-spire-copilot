use super::support::situation;
use crate::learning::action::SemanticAction;
use crate::learning::case::{CaseDraft, CaseOutcome, CaseProvenance, DecisionCase, seed_hash};
use crate::learning::config::{MemoryConfig, MemoryMode};
use crate::learning::session::{LearningSession, SessionProvenance};
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::store::KnowledgeStore;
use crate::learning::telemetry::DecisionSource;

#[test]
fn critic_trims_oldest_cases_before_terminal_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let cases: Vec<_> = (1..=14).map(large_case).collect();
    let first_decision = cases.first().unwrap().decision_id.clone();
    let terminal_decisions: Vec<_> = cases
        .iter()
        .rev()
        .take(4)
        .map(|case| case.decision_id.clone())
        .collect();
    let store = KnowledgeStore::new(temp.path());
    store.append_cases(&cases).unwrap();
    let session = LearningSession::new(
        MemoryConfig {
            mode: MemoryMode::Collect,
            ..MemoryConfig::default()
        },
        store,
        KnowledgeSnapshot::build(cases, &[]).unwrap(),
        SessionProvenance {
            locale: "en".into(),
            model_profile_sha256: "model".into(),
            compatibility_sha256: "mods".into(),
            rules_sha256: "rules".into(),
            synthetic_input: false,
        },
    );

    let prompt = session
        .build_critic_prompt("base report", "run-large")
        .expect("later run evidence should survive prompt fitting");

    assert!(prompt.len() <= 40_000);
    assert!(!prompt.contains(&first_decision));
    for decision_id in terminal_decisions {
        assert!(prompt.contains(&decision_id));
    }
}

fn large_case(sequence: i64) -> DecisionCase {
    let mut state = situation();
    state.player_power_ids = (0..20)
        .map(|index| format!("power-{sequence}-{index}-{}", "x".repeat(80)))
        .collect();
    CaseDraft {
        run_id: "run-large".into(),
        decision_id: format!("run-large:50:6:{sequence:04}"),
        seed_hash: seed_hash(sequence, "mods"),
        situation: state,
        ascension_level: Some(20),
        selected_action: SemanticAction::EndTurn,
        decision_source: DecisionSource::Llm,
        available_semantic_actions: vec![SemanticAction::EndTurn],
        ranked_suggestions: vec![],
        retrieved_memory_ids: vec![],
        memory_ids_used: vec![],
    }
    .finalize(
        CaseOutcome {
            command_succeeded: true,
            player_hp_before_action: Some(10),
            player_hp_after_action: Some(0),
            action_hp_lost: Some(10),
            player_died_after_action: Some(sequence == 14),
            alive_monsters_after_action: Some(1),
            turn_hp_lost: Some(10),
            combat_completed: true,
            combat_won: Some(false),
            combat_hp_lost: Some(10),
            combat_turns: Some(6),
            potions_used: vec![],
            run_completed: true,
            run_victory: Some(false),
            final_floor: Some(50),
        },
        CaseProvenance {
            app_version: "0.2.0".into(),
            prompt_schema_version: 1,
            rules_sha256: "rules".into(),
            model_profile_sha256: "model".into(),
            compatibility_sha256: "mods".into(),
            knowledge_snapshot_id: None,
        },
    )
    .unwrap()
}
