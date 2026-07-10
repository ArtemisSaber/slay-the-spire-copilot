use super::*;

#[test]
fn grid_candidates_use_choice_list() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "GRID",
            "choice_list": ["Strike", "Strike", "Defend"]
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0].action_id, "grid:0");
    assert_eq!(candidates[2].action_id, "grid:2");

    let raw_with_confirm = json!({
        "available_commands": ["choose", "confirm"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "GRID",
            "choice_list": ["Strike"]
        }
    });
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw_with_confirm),
        &state(raw_with_confirm),
    );
    assert_eq!(candidates.len(), 2);
    assert!(candidates.iter().any(|c| c.action_id == "grid:0"));
    assert!(
        candidates
            .iter()
            .any(|c| c.action_id == "grid:confirm" && c.kind == "proceed")
    );
}

#[test]
fn resolve_grid_confirm_returns_proceed() {
    let raw = json!({
        "available_commands": ["confirm"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "GRID",
            "choice_list": []
        }
    });
    let command_state = command_state(&raw);
    let state = state(raw);
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    let request = ActionRequest {
        kind: "proceed".to_string(),
        action_id: "grid:confirm".to_string(),
        target_index: None,
    };
    assert_eq!(
        resolve_requested_action(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state,
            &request
        ),
        Some(AutoPlayAction::Proceed)
    );
}

#[test]
fn grid_candidates_empty_no_commands() {
    let raw = json!({
        "available_commands": ["wait"],
        "ready_for_command": true,
        "game_state": {"screen_type": "GRID", "choice_list": []}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn resolve_grid_no_candidates_for_non_confirm_non_choose() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "GRID", "choice_list": ["Strike"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    assert_eq!(
        resolve_requested_action(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
            &request("skip", "grid:0"),
        ),
        None
    );
}

#[test]
fn resolve_grid_choose_returns_choose() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "GRID", "choice_list": ["Strike", "Defend"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    assert_eq!(
        resolve_requested_action(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
            &request("choose", "grid:1"),
        ),
        Some(AutoPlayAction::Choose(1))
    );
}
