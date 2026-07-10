use super::*;

#[test]
fn try_deterministic_combat_reward_defers_key_or_relic_choice() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["relic", "sapphire_key"]
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

    let action = try_deterministic_action(
        &mut control,
        &mut AutoPlaySession::default(),
        &command_state,
        &state,
        &candidates,
    );
    assert_eq!(action, None);
    assert_eq!(
        fallback_action(&control, &command_state, &state),
        None,
        "the fallback must not replace the LLM's key-or-relic decision"
    );
}

#[test]
fn try_deterministic_combat_reward_picks_relic_without_key() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["relic", "potion"],
            "potions": [
                {"id": "Potion Slot", "name": "Potion Slot", "can_use": false, "can_discard": false, "description": ""}
            ]
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
fn try_deterministic_combat_reward_picks_gold_before_potion() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["gold", "potion"],
            "potions": [
                {"id": "Potion Slot", "name": "Potion Slot", "can_use": false, "can_discard": false, "description": ""},
                {"id": "Potion Slot", "name": "Potion Slot", "can_use": false, "can_discard": false, "description": ""}
            ]
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
fn try_deterministic_combat_reward_with_full_slots_falls_through() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["potion", "card"],
            "potions": [
                {"id": "Strength Potion", "name": "力量药水", "can_use": false, "can_discard": true, "description": ""}
            ]
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

    // Potion in list but no empty slots — should fall through to LLM (returns None)
    let action = try_deterministic_action(
        &mut control,
        &mut AutoPlaySession::default(),
        &command_state,
        &state,
        &candidates,
    );
    assert_eq!(action, None);
}
