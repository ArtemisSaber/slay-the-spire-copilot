use super::*;

#[test]
fn missing_default_control_is_active_for_testing() {
    let load = ControlLoad::MissingDefault(AutoPlayControl::default_enabled());

    assert_eq!(
        active_control(&load),
        Some(&AutoPlayControl::default_enabled())
    );
}

#[test]
fn malformed_control_is_not_active() {
    let load = ControlLoad::Malformed("bad json".to_string());

    assert_eq!(active_control(&load), None);
}

#[test]
fn paused_control_has_no_action_candidates() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["gold"]
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.mode = crate::autoplay::control::AutoPlayMode::Paused;

    assert!(
        available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw)
        )
        .is_empty()
    );
}

#[test]
fn stale_control_is_not_active() {
    assert_eq!(active_control(&ControlLoad::Stale), None);
}

#[test]
fn updated_control_is_active() {
    let load = ControlLoad::Updated(AutoPlayControl::default_enabled());
    assert!(active_control(&load).is_some());
}

#[test]
fn require_confirmation_blocks_candidates() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "EVENT", "choice_list": ["Fight"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.require_confirmation = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn off_mode_produces_no_candidates() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["gold"]
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.mode = crate::autoplay::control::AutoPlayMode::Off;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn advise_mode_produces_no_candidates() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["gold"]
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.mode = crate::autoplay::control::AutoPlayMode::Advise;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}
