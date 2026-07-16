use super::*;

#[tokio::test]
async fn mock_provider_plans_with_llm_json() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {
                "skip_available": true,
                "cards": [
                    {"id": "Uppercut", "name": "Uppercut"}
                ]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    let command_state = command_state(&raw);
    let state = state(raw);

    let action = plan_action(
        &LlmProvider::Mock,
        &mut control,
        &mut AutoPlaySession::default(),
        &command_state,
        &state,
        &Locale::load("en"),
        false,
    )
    .await
    .unwrap();

    assert_eq!(action, Some(AutoPlayAction::Choose(0)));
}

#[test]
fn fallback_event_with_multiple_choices_returns_first() {
    let raw = json!({
        "available_commands": ["choose"],
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Fight", "Leave"]
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);

    let action = fallback_action(&control, &command_state, &state);
    assert_eq!(action, Some(AutoPlayAction::Choose(0)));
}

#[test]
fn fallback_map_with_multiple_children_returns_first() {
    let raw = json!({
        "available_commands": ["choose"],
        "game_state": {
            "screen_type": "MAP",
            "choice_list": ["M", "?"]
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);

    let action = fallback_action(&control, &command_state, &state);
    assert_eq!(action, Some(AutoPlayAction::Choose(0)));
}

#[test]
fn fallback_combat_reward_unknown_choice_returns_first() {
    let raw = json!({
        "available_commands": ["choose"],
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["unknown_reward"]
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);

    let action = fallback_action(&control, &command_state, &state);
    assert_eq!(action, Some(AutoPlayAction::Choose(0)));
}

#[test]
fn fallback_hand_select_returns_first_choice() {
    let raw = json!({
        "available_commands": ["choose"],
        "game_state": {
            "screen_type": "HAND_SELECT",
            "choice_list": ["Strike", "Defend"]
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);

    let action = fallback_action(&control, &command_state, &state);
    assert_eq!(action, Some(AutoPlayAction::Choose(0)));
}

#[test]
fn fallback_combat_uses_has_target_not_card_type() {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [
                    {"id": "Neutralize", "name": "Neutralize", "cost": 0, "type": "SKILL", "uuid": "neut-1", "has_target": true, "is_playable": true},
                    {"id": "Defend", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "def-1", "has_target": false, "is_playable": true}
                ],
                "monsters": [
                    {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                ]
            }
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);

    let action = fallback_action(&control, &command_state, &state);
    match action {
        Some(AutoPlayAction::Play { target_index, .. }) => {
            assert!(
                target_index.is_some(),
                "skill with has_target must include target"
            );
        }
        other => panic!("expected Play with target, got {other:?}"),
    }
}

#[test]
fn try_deterministic_combat_reward_uses_fallback() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["gold", "relic"]
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

    assert!(candidates.len() > 1);
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
fn fallback_rest_proceeds_when_choose_unavailable() {
    let raw = json!({
        "available_commands": ["proceed"],
        "game_state": {
            "screen_type": "REST",
            "screen_state": {
                "rest_options": []
            }
        }
    });
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);

    let action = fallback_rest_action(&command_state, &state);
    assert_eq!(action, Some(AutoPlayAction::Proceed));
}

#[test]
fn fallback_grid_proceeds_when_choose_unavailable() {
    let raw = json!({
        "available_commands": ["confirm"],
        "game_state": {
            "screen_type": "GRID",
            "choice_list": []
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);

    let action = fallback_action(&control, &command_state, &state);
    assert_eq!(action, Some(AutoPlayAction::Proceed));
}

#[test]
fn fallback_hand_select_proceeds_when_choose_unavailable() {
    let raw = json!({
        "available_commands": ["confirm"],
        "game_state": {
            "screen_type": "HAND_SELECT",
            "choice_list": []
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);

    let action = fallback_action(&control, &command_state, &state);
    assert_eq!(action, Some(AutoPlayAction::Proceed));
}
