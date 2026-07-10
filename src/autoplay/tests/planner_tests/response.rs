use super::*;
#[test]
fn parses_llm_action_json_into_executable_action() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {
                "skip_available": true,
                "cards": [
                    {"id": "Uppercut", "name": "Uppercut"},
                    {"id": "Anger", "name": "Anger"}
                ]
            }
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = command_state(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );

    let prompt = build_planner_prompt(
        &AutoPlaySession::default(),
        &command_state,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        &[],
    )
    .unwrap();

    assert!(prompt.contains("=== Current State ==="));
    assert!(prompt.contains("\"language\": \"English\""));

    let action = parse_planner_response(
        r#"{
            "schema_version": 1,
            "actions": [{
                "kind": "choose",
                "action_id": "card_reward:1",
                "label": "Anger",
                "reason": "Cheap attack.",
                "risk": ""
            }]
        }"#,
        &control,
        &command_state,
        &state,
        &candidates,
    )
    .unwrap();

    assert_eq!(action, Some(AutoPlayAction::Choose(1)));
}

#[test]
fn rejects_unavailable_llm_action() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Leave"]
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = command_state(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );

    let error = parse_planner_response(
        r#"{
            "schema_version": 1,
            "actions": [{
                "kind": "choose",
                "action_id": "event:9",
                "label": "Bad index",
                "reason": "",
                "risk": ""
            }]
        }"#,
        &control,
        &command_state,
        &state,
        &candidates,
    )
    .unwrap_err();

    assert!(error.to_string().contains("unavailable action_id"));
}

#[test]
fn rejects_non_json_llm_output() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Leave"]
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = command_state(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );

    let error = parse_planner_response(
        "推荐：do something",
        &control,
        &command_state,
        &state,
        &candidates,
    )
    .unwrap_err();

    assert!(error.to_string().contains("non-JSON"));
}

#[test]
fn retry_prompt_includes_rejected_action_and_reason() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Leave"]
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = command_state(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );
    let rejections = vec![RejectedAttempt {
        attempt: 1,
        rejected_action: Some(ActionRequestSummary {
            kind: "choose".to_string(),
            action_id: "event:9".to_string(),
            target_index: None,
        }),
        reason: "autoplay planner returned unavailable action_id event:9".to_string(),
    }];

    let prompt = build_planner_prompt(
        &AutoPlaySession::default(),
        &command_state,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        &rejections,
    )
    .unwrap();

    assert!(prompt.contains("\"rejected_attempts\""));
    assert!(prompt.contains("\"action_id\": \"event:9\""));
    assert!(prompt.contains("unavailable action_id"));
}
