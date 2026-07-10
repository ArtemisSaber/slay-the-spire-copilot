use super::*;

#[test]
fn disallowed_screen_flag_returns_empty() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "REST", "screen_state": {"rest_options": ["rest"]}}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_rest = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn combat_reward_filters_disallowed_choices() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "COMBAT_REWARD", "choice_list": ["unknown_type", "gold"]}
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "combat_reward:gold:1");
}

#[test]
fn boss_reward_disallowed_flag_returns_empty() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "BOSS_REWARD",
            "screen_state": {"relics": [{"id": "Runic Dome", "name": "Runic Dome"}]}
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_boss_rewards = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn card_reward_disallowed_flag_returns_empty() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {"cards": [{"id": "Anger", "name": "Anger"}]}
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_card_rewards = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn combat_reward_disallowed_flag_returns_empty() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "COMBAT_REWARD", "choice_list": ["gold"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_combat_rewards = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn event_disallowed_flag_returns_empty() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "EVENT", "choice_list": ["Fight"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_events = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn shop_disallowed_flag_returns_empty() {
    let raw = json!({
        "available_commands": ["choose", "leave"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_SCREEN", "choice_list": ["purge"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_shop = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn map_disallowed_flag_returns_empty() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "MAP", "choice_list": ["M"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_map = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn combat_none_disallowed_flag_returns_empty() {
    let raw = json!({
        "available_commands": ["end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [],
                "monsters": [{"name": "Worm", "current_hp": 10, "max_hp": 10, "block": 0, "intent": "ATTACK", "is_gone": false}]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_combat = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn grid_disallowed_flag_returns_empty() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "GRID", "choice_list": ["Strike"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn hand_select_disallowed_flag_returns_empty() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "HAND_SELECT", "choice_list": ["Strike"]}
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_selection_screens = false;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}
