use super::*;

#[test]
fn card_reward_request_can_choose_or_skip() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {
                "skip_available": true,
                "cards": [{"id": "Anger", "name": "Anger"}]
            }
        }
    });

    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "card_reward:0")
        ),
        Some(AutoPlayAction::Choose(0))
    );
    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("skip", "card_reward:skip")
        ),
        Some(AutoPlayAction::Skip)
    );
}

#[test]
fn card_reward_boss_uses_boss_prefix_in_candidates() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "floor": 16,
            "screen_state": {
                "skip_available": true,
                "cards": [{"id": "Impervious", "name": "Impervious"}]
            }
        }
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(
        candidates
            .iter()
            .any(|c| c.action_id == "boss_card_reward:0")
    );
    assert!(
        candidates
            .iter()
            .any(|c| c.action_id == "boss_card_reward:skip")
    );
}

#[test]
fn card_reward_no_skip_when_not_available() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {
                "skip_available": false,
                "cards": [{"id": "Anger", "name": "Anger"}]
            }
        }
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "card_reward:0");
}

#[test]
fn resolve_card_reward_rejects_index_out_of_bounds() {
    let raw = json!({
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {"cards": [{"id": "Anger", "name": "Anger"}]}
        }
    });
    let cmd = CommandState {
        ready_for_command: true,
        available_commands: vec!["choose".into()],
        choice_list: vec![],
    };
    let s = state(raw);
    let req = ActionRequest {
        kind: "choose".into(),
        action_id: "card_reward:5".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_card_reward(&cmd, &s, &req), None);
}

#[test]
fn resolve_card_reward_rejects_wrong_kind() {
    let raw = json!({
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {"cards": [{"id": "Anger", "name": "Anger"}]}
        }
    });
    let cmd = CommandState {
        ready_for_command: true,
        available_commands: vec!["choose".into()],
        choice_list: vec![],
    };
    let s = state(raw);
    let req = ActionRequest {
        kind: "skip".into(),
        action_id: "card_reward:0".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_card_reward(&cmd, &s, &req), None);
}

#[test]
fn resolve_boss_card_reward_skip() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "floor": 16,
            "screen_state": {
                "skip_available": true,
                "cards": [{"id": "Impervious", "name": "Impervious"}]
            }
        }
    });
    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("skip", "boss_card_reward:skip")
        ),
        Some(AutoPlayAction::Skip)
    );
}

#[test]
fn card_reward_no_skip_when_skip_command_missing() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {
                "skip_available": true,
                "cards": [{"id": "Anger", "name": "Anger"}]
            }
        }
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "card_reward:0");
}

#[test]
fn boss_card_reward_no_skip_when_skip_command_missing() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "floor": 16,
            "screen_state": {
                "skip_available": true,
                "cards": [{"id": "Impervious", "name": "Impervious"}]
            }
        }
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "boss_card_reward:0");
}

#[test]
fn resolve_card_reward_choose_without_choose_command() {
    let raw = json!({
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {"cards": [{"id": "Anger", "name": "Anger"}]}
        }
    });
    let cmd = CommandState {
        ready_for_command: true,
        available_commands: vec![],
        choice_list: vec![],
    };
    let s = state(raw);
    let req = ActionRequest {
        kind: "choose".into(),
        action_id: "card_reward:0".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_card_reward(&cmd, &s, &req), None);
}
