use super::*;
use crate::autoplay::action::ActionCandidate;

fn combat_fixture() -> (
    AutoPlayControl,
    CommandState,
    NormalizedState,
    Vec<ActionCandidate>,
) {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [{
                    "id": "Strike_R", "name": "Strike", "cost": 1,
                    "type": "ATTACK", "uuid": "strike-1", "has_target": true,
                    "is_playable": true
                }],
                "monsters": [{
                    "id": "JawWorm", "name": "Jaw Worm", "current_hp": 40,
                    "max_hp": 40, "block": 0, "intent": "ATTACK",
                    "is_gone": false
                }]
            }
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command = command_state(&raw);
    let normalized = state(raw);
    let candidates =
        available_action_candidates(&control, &AutoPlaySession::default(), &command, &normalized);
    (control, command, normalized, candidates)
}

#[test]
fn planner_prompt_includes_the_supplied_experience_context_unchanged() {
    let (_, command, state, candidates) = combat_fixture();
    let memory = json!({
        "schema_version": 1,
        "items": [{"memory_id": "lesson:one", "observation": "Block first."}]
    });

    let prompt = build_planner_prompt_with_memory(
        &AutoPlaySession::default(),
        &command,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        &[],
        Some(&memory),
    )
    .unwrap();
    let payload: Value = serde_json::from_str(&prompt).unwrap();

    assert_eq!(payload["experience_context"], memory);
    let task = payload["task"].as_str().unwrap();
    assert!(task.contains("memory_ids_used"));
    assert!(task.contains("observational"));
    assert!(task.contains("available_actions"));
}

#[test]
fn planner_response_keeps_only_memory_ids_that_were_retrieved() {
    let (control, command, state, candidates) = combat_fixture();
    let parsed = parse_planner_response_with_memory(
        r#"{
            "schema_version": 2,
            "memory_ids_used": ["lesson:one", "invented", "lesson:one"],
            "actions": [{
                "ref": "A0", "target_index": 0,
                "reason": "Known case", "risk": ""
            }]
        }"#,
        &control,
        &command,
        &state,
        &candidates,
        &["lesson:one".to_string(), "case:two".to_string()],
    )
    .unwrap();

    assert_eq!(
        parsed.action,
        AutoPlayAction::Play {
            hand_index: 0,
            target_index: Some(0)
        }
    );
    assert_eq!(parsed.selected_action_id, "combat:play:strike-1");
    assert_eq!(parsed.memory_ids_used, vec!["lesson:one"]);
}

#[test]
fn legacy_prompt_omits_experience_context() {
    let (_, command, state, candidates) = combat_fixture();
    let prompt = build_planner_prompt(
        &AutoPlaySession::default(),
        &command,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        &[],
    )
    .unwrap();
    let payload: Value = serde_json::from_str(&prompt).unwrap();

    assert!(payload.get("experience_context").is_none());
}

#[tokio::test]
async fn combat_reference_resolves_to_the_real_action_and_telemetry_id() {
    let (mut control, command, state, _) = combat_fixture();
    let planned = plan_action_with_memory(
        &LlmProvider::Mock,
        &mut control,
        &mut AutoPlaySession::default(),
        &command,
        &state,
        &Locale::load("en"),
        false,
        None,
        &[],
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(
        planned.action,
        AutoPlayAction::Play {
            hand_index: 0,
            target_index: Some(0)
        }
    );
    assert_eq!(planned.selected_action_id, "combat:play:strike-1");
    assert_eq!(planned.source, DecisionSource::Llm);
}

#[tokio::test]
async fn typed_plan_reports_llm_source_and_stable_candidate_id() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {
                "skip_available": true,
                "cards": [{"id": "Uppercut", "name": "Uppercut"}]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    let command = command_state(&raw);
    let state = state(raw);

    let planned = plan_action_with_memory(
        &LlmProvider::Mock,
        &mut control,
        &mut AutoPlaySession::default(),
        &command,
        &state,
        &Locale::load("en"),
        false,
        None,
        &[],
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(planned.action, AutoPlayAction::Choose(0));
    assert_eq!(
        planned.source,
        crate::learning::telemetry::DecisionSource::Llm
    );
    assert_eq!(planned.selected_action_id, "card_reward:0");
    assert!(planned.memory_ids_used.is_empty());
}
