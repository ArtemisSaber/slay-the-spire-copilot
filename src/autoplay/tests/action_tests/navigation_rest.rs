use super::*;

#[test]
fn rest_request_maps_option_to_index() {
    let raw = json!({
        "available_commands": ["choose", "return"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "REST",
            "screen_state": {
                "rest_options": ["rest", "smith", "toke"]
            }
        }
    });

    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "rest:smith")
        ),
        Some(AutoPlayAction::Choose(1))
    );
}

#[test]
fn rest_candidates_includes_proceed_when_available() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "REST",
            "screen_state": {
                "rest_options": ["rest", "smith"]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_rest = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 3);
    assert!(candidates.iter().any(|c| c.action_id == "rest:rest"));
    assert!(candidates.iter().any(|c| c.action_id == "rest:smith"));
    assert!(
        candidates
            .iter()
            .any(|c| c.action_id == "rest:proceed" && c.kind == "proceed")
    );
}

#[test]
fn resolve_rest_proceed_returns_proceed() {
    let raw = json!({
        "game_state": {
            "screen_type": "REST",
            "screen_state": {
                "rest_options": []
            }
        }
    });
    let state = state(raw);
    let request = ActionRequest {
        kind: "proceed".to_string(),
        action_id: "rest:proceed".to_string(),
        target_index: None,
    };
    assert_eq!(
        resolve_requested_rest(&state, &request),
        Some(AutoPlayAction::Proceed)
    );
}

#[test]
fn rest_candidates_empty_without_choose_or_proceed() {
    let raw = json!({
        "available_commands": ["wait"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "REST",
            "screen_state": {"rest_options": ["rest"]}
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_rest = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn resolve_rest_option_not_found() {
    let s = NormalizedState {
        rest_options: vec!["rest".to_string()],
        ..NormalizedState::default()
    };
    let req = ActionRequest {
        kind: "choose".into(),
        action_id: "rest:nonexistent".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_rest(&s, &req), None);
}

#[test]
fn resolve_rest_rejects_wrong_kind() {
    let s = NormalizedState {
        rest_options: vec!["rest".to_string()],
        ..NormalizedState::default()
    };
    let req = ActionRequest {
        kind: "skip".into(),
        action_id: "rest:rest".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_rest(&s, &req), None);
}

#[test]
fn rest_candidates_empty_options_with_proceed() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "REST",
            "screen_state": {"rest_options": []}
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_rest = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "rest:proceed");
}
