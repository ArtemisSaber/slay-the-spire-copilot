use super::*;

#[test]
fn shop_request_can_leave_or_choose_index() {
    let raw = json!({
        "available_commands": ["choose", "leave"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "SHOP_SCREEN",
            "choice_list": ["purge", "Strike"]
        }
    });
    let control = AutoPlayControl::default_enabled();

    assert_eq!(
        resolve(&raw, &control, &request("leave", "shop:leave")),
        Some(AutoPlayAction::Leave)
    );
    assert_eq!(
        resolve(&raw, &control, &request("choose", "shop:choice:1")),
        Some(AutoPlayAction::Choose(1))
    );
}

#[test]
fn shop_candidates_first_visit_shows_choices_and_leave() {
    let raw = json!({
        "available_commands": ["choose", "leave"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_SCREEN", "floor": 5, "choice_list": ["purge", "Strike"]}
    });
    let session = AutoPlaySession {
        last_shop_room_floor: Some(3),

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
    assert!(candidates.iter().any(|c| c.action_id == "shop:choice:1"));
    assert!(candidates.iter().any(|c| c.action_id == "shop:leave"));
}

#[test]
fn resolve_shop_proceed_returns_proceed() {
    let raw = json!({
        "available_commands": ["proceed"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_SCREEN", "floor": 5, "choice_list": []}
    });
    let session = AutoPlaySession {
        last_shop_room_floor: Some(5),

        ..Default::default()
    };
    let mut s = state(raw.clone());
    s.floor = Some(5);
    assert_eq!(
        resolve_requested_action(
            &AutoPlayControl::default_enabled(),
            &session,
            &command_state(&raw),
            &s,
            &request("proceed", "shop:proceed"),
        ),
        Some(AutoPlayAction::Proceed)
    );
}

#[test]
fn shop_candidates_floor_none_not_entered() {
    let raw = json!({
        "available_commands": ["choose", "leave"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_SCREEN", "choice_list": ["purge"]}
    });
    let session = AutoPlaySession {
        last_shop_room_floor: Some(5),

        ..Default::default()
    };
    let s = state(raw.clone());
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &session,
        &command_state(&raw),
        &s,
    );
    assert!(candidates.iter().any(|c| c.action_id == "shop:choice:0"));
}

#[test]
fn shop_room_screen_type_produces_candidates() {
    let raw = json!({
        "available_commands": ["choose", "leave"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_ROOM", "choice_list": ["purge", "Strike"]}
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.iter().any(|c| c.action_id == "shop:choice:0"));
    assert!(candidates.iter().any(|c| c.action_id == "shop:leave"));
}

#[test]
fn resolve_shop_room_leave_returns_leave() {
    let raw = json!({
        "available_commands": ["choose", "leave"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_ROOM", "choice_list": ["purge"]}
    });
    let control = AutoPlayControl::default_enabled();
    assert_eq!(
        resolve(&raw, &control, &request("leave", "shop:leave")),
        Some(AutoPlayAction::Leave)
    );
}

#[test]
fn resolve_requested_shop_leave_wrong_kind() {
    let cmd = CommandState {
        ready_for_command: true,
        available_commands: vec!["choose".into(), "leave".into()],
        choice_list: vec!["purge".into()],
    };
    let req = ActionRequest {
        kind: "skip".into(),
        action_id: "shop:leave".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_shop(&cmd, &req), None);
}

#[test]
fn resolve_requested_shop_proceed_wrong_kind() {
    let cmd = CommandState {
        ready_for_command: true,
        available_commands: vec!["proceed".into(), "leave".into()],
        choice_list: vec![],
    };
    let req = ActionRequest {
        kind: "skip".into(),
        action_id: "shop:proceed".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_shop(&cmd, &req), None);
}
