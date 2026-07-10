use super::*;

#[test]
fn combat_reward_request_can_proceed_when_empty() {
    let raw = json!({
        "available_commands": ["proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": []
        }
    });

    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("proceed", "combat_reward:proceed")
        ),
        Some(AutoPlayAction::Proceed)
    );
}

#[test]
fn combat_reward_skips_potion_when_slots_full() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "COMBAT_REWARD", "choice_list": ["potion", "gold"]}
    });
    let session = AutoPlaySession {
        skipped_combat_reward_potion: true,

        ..Default::default()
    };
    let mut s = state(raw.clone());
    s.empty_potion_slots = 0;
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &session,
        &command_state(&raw),
        &s,
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "combat_reward:gold:1");
}

#[test]
fn combat_reward_skips_card_when_session_says_so() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "COMBAT_REWARD", "choice_list": ["card", "gold"]}
    });
    let session = AutoPlaySession {
        skipped_combat_reward_card: true,

        ..Default::default()
    };
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &session,
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "combat_reward:gold:1");
}

#[test]
fn combat_reward_no_proceed_when_candidates_visible() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {"screen_type": "COMBAT_REWARD", "choice_list": ["gold"]}
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].kind, "choose");
}

#[test]
fn combat_reward_proceed_added_when_no_visible_candidates() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {"screen_type": "COMBAT_REWARD", "choice_list": ["unknown"]}
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "combat_reward:proceed");
}

#[test]
fn combat_reward_does_not_skip_potion_when_slots_not_full() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "COMBAT_REWARD", "choice_list": ["potion", "gold"]}
    });
    let session = AutoPlaySession {
        skipped_combat_reward_potion: true,

        ..Default::default()
    };
    let mut s = state(raw.clone());
    s.empty_potion_slots = 2;
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &session,
        &command_state(&raw),
        &s,
    );
    assert_eq!(candidates.len(), 2);
    assert!(
        candidates
            .iter()
            .any(|c| c.action_id == "combat_reward:potion:0")
    );
    assert!(
        candidates
            .iter()
            .any(|c| c.action_id == "combat_reward:gold:1")
    );
}

#[test]
fn resolve_combat_reward_proceed_wrong_kind() {
    let cmd = CommandState {
        ready_for_command: true,
        available_commands: vec!["proceed".into()],
        choice_list: vec![],
    };
    let req = ActionRequest {
        kind: "skip".into(),
        action_id: "combat_reward:proceed".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_combat_reward(&cmd, &req), None);
}
