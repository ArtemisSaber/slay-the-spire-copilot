use super::support::situation;
use crate::autoplay::action::AutoPlayAction;
use crate::learning::action::SemanticAction;
use crate::learning::telemetry::{
    DecisionIdGenerator, DecisionSource, PlannedDecision, RecordedAction, proposal_event,
};

fn planned() -> PlannedDecision {
    PlannedDecision {
        decision_id: "run:8:2:1".into(),
        selected_action_id: "combat:play:run-local-uuid".into(),
        selected_semantic_action: SemanticAction::PlayCard {
            card_id: "Bash".into(),
            upgraded: false,
            target_monster_id: Some("GremlinNob".into()),
        },
        action: AutoPlayAction::Play {
            hand_index: 0,
            target_index: Some(0),
        },
        source: DecisionSource::Llm,
        available_actions: vec![RecordedAction {
            action_id: "combat:play:run-local-uuid".into(),
            semantic_action: SemanticAction::EndTurn,
        }],
        ranked_suggestions: vec![],
        retrieved_memory_ids: vec!["lesson:1".into()],
        memory_ids_used: vec!["lesson:1".into()],
        knowledge_snapshot_id: Some("sha256:snapshot".into()),
    }
}

#[test]
fn decision_ids_are_unique_and_stable_within_a_run() {
    let mut generator = DecisionIdGenerator::new("run-id");
    assert_eq!(generator.next(Some(8), Some(2)), "run-id:8:2:1");
    assert_eq!(generator.next(Some(8), Some(2)), "run-id:8:2:2");
    assert_eq!(generator.next(None, None), "run-id:unknown:unknown:3");
}

#[test]
fn proposal_event_keeps_audit_metadata_but_not_execution_indices() {
    let situation = situation();
    let event = proposal_event(&planned(), "observation", &situation).unwrap();

    assert_eq!(event["event"], "autoplay_decision_proposed");
    assert_eq!(event["source"], "llm");
    assert_eq!(event["selected_semantic_action"]["card_id"], "Bash");
    assert_eq!(event["retrieved_memory_ids"][0], "lesson:1");
    assert_eq!(event["memory_ids_used"][0], "lesson:1");
    assert!(event.get("hand_index").is_none());
    assert!(event.get("target_index").is_none());
}

#[test]
fn model_memory_use_is_a_self_report_not_a_causal_label() {
    let event = proposal_event(&planned(), "observation", &situation()).unwrap();
    assert_eq!(event["memory_ids_used"], event["retrieved_memory_ids"]);
    assert!(event.get("memory_caused_action").is_none());
}
