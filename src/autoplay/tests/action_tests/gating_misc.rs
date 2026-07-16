use super::*;

#[test]
fn not_ready_has_no_action_candidates() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": false,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["gold"]
        }
    });

    assert!(
        available_action_candidates(
            &AutoPlayControl::default_enabled(),
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw)
        )
        .is_empty()
    );
}

#[test]
fn unknown_screen_type_returns_empty_candidates() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "UNKNOWN_WEIRD_SCREEN", "choice_list": ["A"]}
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn shop_screen_keeps_choices_until_returning_to_shop_room() {
    let raw = json!({
        "available_commands": ["choose", "proceed", "leave"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_SCREEN", "floor": 5, "choice_list": ["purge"]}
    });
    let session = AutoPlaySession {
        completed_shop_floor: Some(5),

        ..Default::default()
    };
    let mut s = state(raw.clone());
    s.floor = Some(5);
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &session,
        &command_state(&raw),
        &s,
    );
    assert!(candidates.iter().any(|c| c.action_id == "shop:choice:0"));
    assert!(candidates.iter().any(|c| c.action_id == "shop:leave"));
}

#[test]
fn completed_shop_room_suppresses_reentry() {
    let raw = json!({
        "available_commands": ["choose", "proceed", "leave"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_ROOM", "floor": 5, "choice_list": ["shop"]}
    });
    let session = AutoPlaySession {
        completed_shop_floor: Some(5),

        ..Default::default()
    };
    let mut s = state(raw.clone());
    s.floor = Some(5);
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &session,
        &command_state(&raw),
        &s,
    );
    assert!(candidates.iter().any(|c| c.action_id == "shop:proceed"));
    assert!(candidates.iter().any(|c| c.action_id == "shop:leave"));
    assert!(!candidates.iter().any(|c| c.kind == "choose"));
}

#[test]
fn completed_shop_on_another_floor_does_not_suppress_entry() {
    let raw = json!({
        "available_commands": ["choose", "leave"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_ROOM", "floor": 6, "choice_list": ["shop"]}
    });
    let session = AutoPlaySession {
        completed_shop_floor: Some(5),

        ..Default::default()
    };
    let mut s = state(raw.clone());
    s.floor = Some(6);
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &session,
        &command_state(&raw),
        &s,
    );
    assert!(candidates.iter().any(|c| c.action_id == "shop:choice:0"));
}
