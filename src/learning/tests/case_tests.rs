use super::support::situation;
use crate::learning::action::SemanticAction;
use crate::learning::case::{CaseDraft, CaseError, CaseOutcome, CaseProvenance, seed_hash};
use crate::learning::telemetry::{DecisionSource, RecordedRankedAction};

fn action() -> SemanticAction {
    SemanticAction::PlayCard {
        card_id: "Bash".into(),
        upgraded: false,
        target_monster_id: Some("GremlinNob".into()),
    }
}

fn provenance() -> CaseProvenance {
    CaseProvenance {
        app_version: "0.2.0".into(),
        prompt_schema_version: 1,
        rules_sha256: "rules".into(),
        model_profile_sha256: "model".into(),
        compatibility_sha256: "compatibility".into(),
        knowledge_snapshot_id: None,
    }
}

fn draft() -> CaseDraft {
    CaseDraft {
        run_id: "run-1".into(),
        decision_id: "run-1:8:2:1".into(),
        seed_hash: seed_hash(42, "profile-salt"),
        situation: situation(),
        selected_action: action(),
        decision_source: DecisionSource::Llm,
        available_semantic_actions: vec![action(), SemanticAction::EndTurn],
        ranked_suggestions: vec![RecordedRankedAction {
            semantic_action: action(),
            score: 194,
            tags: vec!["damage".into()],
        }],
        retrieved_memory_ids: vec!["lesson:prior".into()],
        memory_ids_used: vec![],
    }
}

fn outcome() -> CaseOutcome {
    CaseOutcome {
        command_succeeded: true,
        player_hp_before_action: None,
        player_hp_after_action: None,
        action_hp_lost: None,
        player_died_after_action: None,
        alive_monsters_after_action: None,
        turn_hp_lost: Some(8),
        combat_completed: true,
        combat_won: Some(true),
        combat_hp_lost: Some(18),
        combat_turns: Some(3),
        potions_used: vec![],
        run_completed: true,
        run_victory: Some(false),
        final_floor: Some(27),
    }
}

#[test]
fn case_identity_is_deterministic_and_verified() {
    let first = draft().finalize(outcome(), provenance()).unwrap();
    let second = draft().finalize(outcome(), provenance()).unwrap();

    assert_eq!(first.case_id, second.case_id);
    assert!(first.case_id.starts_with("sha256:"));
    assert!(first.verify_id());
    assert_eq!(first.situation_hash, situation().situation_hash().unwrap());
}

#[test]
fn case_identity_detects_changes_to_descriptor_and_memory_provenance() {
    let mut descriptor_changed = draft().finalize(outcome(), provenance()).unwrap();
    descriptor_changed.situation.turn_bucket = crate::learning::descriptor::TurnBucket::Turn3;
    assert!(!descriptor_changed.verify_id());

    let mut memory_changed = draft().finalize(outcome(), provenance()).unwrap();
    memory_changed
        .retrieved_memory_ids
        .push("sha256:later-edit".into());
    assert!(!memory_changed.verify_id());
}

#[test]
fn outcome_change_changes_identity_without_claiming_optimality() {
    let first = draft().finalize(outcome(), provenance()).unwrap();
    let mut changed = outcome();
    changed.turn_hp_lost = Some(0);
    let second = draft().finalize(changed, provenance()).unwrap();

    assert_ne!(first.case_id, second.case_id);
    let json = serde_json::to_string(&first).unwrap();
    assert!(!json.contains("optimal"));
    assert!(!json.contains("profile-salt"));
    assert!(!json.contains("\"seed\":"));
}

#[test]
fn seed_hash_is_stable_but_compatibility_scoped() {
    assert_eq!(seed_hash(42, "a"), seed_hash(42, "a"));
    assert_ne!(seed_hash(42, "a"), seed_hash(42, "b"));
    assert!(!seed_hash(42, "a").contains("42"));
}

#[test]
fn case_caps_fail_closed() {
    let mut oversized = draft();
    oversized.available_semantic_actions = vec![SemanticAction::EndTurn; 33];
    assert_eq!(
        oversized.finalize(outcome(), provenance()),
        Err(CaseError::TooManyAvailableActions)
    );

    let mut invalid = draft();
    invalid.retrieved_memory_ids = vec!["x".repeat(257)];
    assert_eq!(
        invalid.finalize(outcome(), provenance()),
        Err(CaseError::InvalidIdentifier)
    );

    let mut unavailable = draft();
    unavailable.available_semantic_actions = vec![SemanticAction::EndTurn];
    assert_eq!(
        unavailable.finalize(outcome(), provenance()),
        Err(CaseError::SelectedActionUnavailable)
    );

    let mut incomplete = outcome();
    incomplete.run_completed = false;
    assert_eq!(
        draft().finalize(incomplete, provenance()),
        Err(CaseError::IncompleteOutcome)
    );
}
