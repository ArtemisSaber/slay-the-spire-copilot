use super::*;

#[test]
fn complete_candidates_returns_proceed() {
    let raw = json!({
        "game_state": {
            "screen_type": "COMPLETE",
            "screen_state": {},
            "seed": -582230291998696632_i64,
            "relics": [],
            "deck": [],
            "map": [],
            "floor": 50,
        },
        "available_commands": ["proceed", "wait", "state"],
        "ready_for_command": true,
        "in_game": true,
    });
    let command_state = command_state(&raw);
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].kind, "proceed");
    assert_eq!(candidates[0].action_id, "complete:proceed");
}

#[test]
fn resolve_complete_proceed_returns_proceed() {
    let raw = json!({
        "game_state": {
            "screen_type": "COMPLETE",
            "screen_state": {},
            "seed": -582230291998696632_i64,
            "relics": [],
            "deck": [],
            "map": [],
            "floor": 50,
        },
        "available_commands": ["proceed", "wait", "state"],
        "ready_for_command": true,
        "in_game": true,
    });
    let command_state = command_state(&raw);
    let state = state(raw);
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    let request = ActionRequest {
        kind: "proceed".to_string(),
        action_id: "complete:proceed".to_string(),
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
fn chest_candidates_choose_options() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "CHEST", "choice_list": ["Open", "Leave"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].action_id, "chest:0");
    assert_eq!(candidates[1].action_id, "chest:1");
}

#[test]
fn chest_candidates_proceed_only() {
    let raw = json!({
        "available_commands": ["proceed"],
        "ready_for_command": true,
        "game_state": {"screen_type": "CHEST", "choice_list": []}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "chest:proceed");
}

#[test]
fn chest_candidates_empty_no_commands() {
    let raw = json!({
        "available_commands": ["wait"],
        "ready_for_command": true,
        "game_state": {"screen_type": "CHEST", "choice_list": []}
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
fn resolve_chest_proceed_returns_proceed() {
    let raw = json!({
        "available_commands": ["proceed"],
        "ready_for_command": true,
        "game_state": {"screen_type": "CHEST", "choice_list": []}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    assert_eq!(
        resolve_requested_action(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
            &request("proceed", "chest:proceed"),
        ),
        Some(AutoPlayAction::Proceed)
    );
}

#[test]
fn resolve_complete_rejects_non_proceed() {
    let raw = json!({
        "game_state": {
            "screen_type": "COMPLETE", "screen_state": {},
            "seed": -582230291998696632_i64, "relics": [], "deck": [], "map": [], "floor": 50
        },
        "available_commands": ["proceed", "wait", "state"],
        "ready_for_command": true,
        "in_game": true
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    assert_eq!(
        resolve_requested_action(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
            &request("skip", "complete:proceed"),
        ),
        None
    );
}

#[test]
fn resolve_chest_no_candidates_for_non_proceed_non_choose() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "CHEST", "choice_list": ["Open"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    assert_eq!(
        resolve_requested_action(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
            &request("skip", "chest:0"),
        ),
        None
    );
}

#[test]
fn resolve_chest_choose_returns_choose() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "CHEST", "choice_list": ["Open", "Leave"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    assert_eq!(
        resolve_requested_action(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
            &request("choose", "chest:1"),
        ),
        Some(AutoPlayAction::Choose(1))
    );
}

#[test]
fn chest_candidates_both_choose_and_proceed_choose_priority() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {"screen_type": "CHEST", "choice_list": ["Open"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "chest:0");
    assert_eq!(candidates[0].kind, "choose");
}
