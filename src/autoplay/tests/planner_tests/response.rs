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
            "schema_version": 2,
            "actions": [{
                "ref": "A1",
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
            "schema_version": 2,
            "actions": [{
                "ref": "A9",
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

    assert!(error.to_string().contains("unknown action ref A9"));
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

    assert!(error.to_string().contains("invalid JSON syntax"));
}

#[test]
fn distinguishes_valid_json_with_an_invalid_planner_schema() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "EVENT", "choice_list": ["Leave"]}
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
        r#"{"ref":"A0"}"#,
        &control,
        &command_state,
        &state,
        &candidates,
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("valid JSON with an invalid schema")
    );
    assert!(error.to_string().contains("missing field `schema_version`"));
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
        rejected_action: Some(ActionSelectionSummary {
            action_ref: "A9".to_string(),
            target_index: None,
        }),
        reason: "autoplay planner returned unknown action ref A9".to_string(),
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
    assert!(prompt.contains("\"ref\": \"A9\""));
    assert!(prompt.contains("unknown action ref"));
}
