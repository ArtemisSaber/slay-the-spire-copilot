use super::*;

#[test]
fn try_deterministic_single_candidate_returns_action() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Proceed"],
            "screen_state": {
                "choices": ["Proceed"]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );

    assert_eq!(candidates.len(), 1);
    let action = try_deterministic_action(
        &mut control,
        &mut AutoPlaySession::default(),
        &command_state,
        &state,
        &candidates,
    );
    assert_eq!(action, Some(AutoPlayAction::Choose(0)));
}

#[test]
fn try_deterministic_multiple_candidates_returns_none() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Fight", "Leave"],
            "screen_state": {
                "choices": ["Fight", "Leave"]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );

    assert_eq!(candidates.len(), 2);
    let action = try_deterministic_action(
        &mut control,
        &mut AutoPlaySession::default(),
        &command_state,
        &state,
        &candidates,
    );
    assert_eq!(action, None);
}
