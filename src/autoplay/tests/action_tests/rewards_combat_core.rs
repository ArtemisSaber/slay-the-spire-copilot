use super::*;

#[test]
fn combat_reward_request_collects_requested_reward() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["gold", "card"]
        }
    });

    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "combat_reward:card:1")
        ),
        Some(AutoPlayAction::Choose(1))
    );
}

#[test]
fn combat_reward_card_not_allowed_when_flag_false() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "COMBAT_REWARD", "choice_list": ["card", "gold"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_card_rewards = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "combat_reward:gold:1");
}

#[test]
fn resolve_combat_reward_rejects_wrong_kind() {
    let cmd = CommandState {
        ready_for_command: true,
        available_commands: vec!["choose".into()],
        choice_list: vec!["gold".into()],
    };
    let req = ActionRequest {
        kind: "skip".into(),
        action_id: "combat_reward:gold:0".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_combat_reward(&cmd, &req), None);
}

#[test]
fn resolve_combat_reward_rejects_no_choose_command() {
    let cmd = CommandState {
        ready_for_command: true,
        available_commands: vec!["proceed".into()],
        choice_list: vec!["gold".into()],
    };
    let req = ActionRequest {
        kind: "choose".into(),
        action_id: "combat_reward:gold:0".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_combat_reward(&cmd, &req), None);
}

#[test]
fn resolve_combat_reward_rejects_bad_index_format() {
    let cmd = CommandState {
        ready_for_command: true,
        available_commands: vec!["choose".into()],
        choice_list: vec!["gold".into()],
    };
    let req = ActionRequest {
        kind: "choose".into(),
        action_id: "combat_reward:gold:abc".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_combat_reward(&cmd, &req), None);
}
