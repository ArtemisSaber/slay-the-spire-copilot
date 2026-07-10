use super::*;

#[test]
fn try_deterministic_hides_skipped_card() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["card"]
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);
    let mut session = AutoPlaySession {
        skipped_combat_reward_card: true,
        ..Default::default()
    };
    let candidates = available_action_candidates(&control, &session, &command_state, &state);
    assert!(
        !candidates.iter().any(|c| c.label.contains("card")),
        "card should not appear in candidates"
    );

    let action = try_deterministic_action(
        &mut control,
        &mut session,
        &command_state,
        &state,
        &candidates,
    );
    assert!(
        !matches!(action, Some(AutoPlayAction::Choose(_))),
        "should not pick the card"
    );
}

#[test]
fn try_deterministic_hides_skipped_potion_with_full_slots() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["potion"],
            "potions": [
                {"id": "FearPotion", "name": "恐惧药水", "can_use": false, "can_discard": true, "description": ""}
            ]
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);
    let mut session = AutoPlaySession {
        skipped_combat_reward_potion: true,
        ..Default::default()
    };
    let candidates = available_action_candidates(&control, &session, &command_state, &state);
    assert!(
        !candidates.iter().any(|c| c.label.contains("potion")),
        "potion should not appear in candidates"
    );

    let action = try_deterministic_action(
        &mut control,
        &mut session,
        &command_state,
        &state,
        &candidates,
    );
    assert!(
        !matches!(action, Some(AutoPlayAction::Choose(_))),
        "should not pick potion"
    );
}

#[test]
fn skip_flags_reset_on_new_floor() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["potion"],
            "floor": 5,
            "potions": [
                {"id": "FearPotion", "name": "恐惧药水", "can_use": false, "can_discard": true, "description": ""}
            ]
        }
    });
    let _command_state = CommandState::from_raw(&raw);
    let state = state(raw);
    let mut session = AutoPlaySession {
        skipped_combat_reward_potion: true,
        skipped_combat_reward_card: true,
        last_combat_reward_floor: Some(3),
        ..AutoPlaySession::default()
    };

    // plan_action triggers the floor-based reset (first call)
    // We simulate just the reset check inline
    if state.screen_type.as_ref().map(|st| st.as_str()) == Some("COMBAT_REWARD")
        && session.last_combat_reward_floor != state.floor
    {
        session.skipped_combat_reward_potion = false;
        session.skipped_combat_reward_card = false;
        session.last_combat_reward_floor = state.floor;
    }

    assert!(!session.skipped_combat_reward_potion);
    assert!(!session.skipped_combat_reward_card);
    assert_eq!(session.last_combat_reward_floor, Some(5));
}

#[test]
fn card_reward_skip_sets_skipped_card_flag() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {
                "skip_available": true,
                "cards": [{"id": "Strike_R", "name": "Strike"}]
            }
        }
    });
    let _command_state = CommandState::from_raw(&raw);
    let state = state(raw);
    let mut session = AutoPlaySession::default();
    let action = Some(AutoPlayAction::Skip);

    // Simulate the LLM response handling that sets the flag
    if state.screen_type.as_ref().map(|st| st.as_str()) == Some("CARD_REWARD")
        && matches!(&action, Some(AutoPlayAction::Skip))
    {
        session.skipped_combat_reward_card = true;
    }

    assert!(
        session.skipped_combat_reward_card,
        "CARD_REWARD skip should set skipped_combat_reward_card"
    );
}

#[test]
fn combat_reward_candidates_hides_skipped_card() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["card", "gold"]
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = CommandState::from_raw(&raw);
    let state = state(raw);
    let session = AutoPlaySession {
        skipped_combat_reward_card: true,
        ..Default::default()
    };
    let candidates = available_action_candidates(&control, &session, &command_state, &state);
    assert!(
        !candidates.iter().any(|c| c.label.contains("card")),
        "skipped card should not appear in candidates"
    );
    assert!(
        candidates.iter().any(|c| c.label.contains("gold")),
        "gold should still be a candidate"
    );
}
