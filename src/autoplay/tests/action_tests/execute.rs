use super::*;

#[test]
fn execute_action_writes_protocol_command() {
    let mut buf = Vec::new();

    execute_action_to(&mut buf, &AutoPlayAction::Choose(2));
    execute_action_to(
        &mut buf,
        &AutoPlayAction::Play {
            hand_index: 1,
            target_index: Some(0),
        },
    );
    execute_action_to(
        &mut buf,
        &AutoPlayAction::Drink {
            slot_index: 0,
            target_index: None,
        },
    );
    execute_action_to(
        &mut buf,
        &AutoPlayAction::Drink {
            slot_index: 1,
            target_index: Some(2),
        },
    );
    execute_action_to(&mut buf, &AutoPlayAction::End);
    execute_action_to(&mut buf, &AutoPlayAction::Skip);
    execute_action_to(&mut buf, &AutoPlayAction::Proceed);
    execute_action_to(&mut buf, &AutoPlayAction::Leave);

    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(
        lines,
        vec![
            "choose 2",
            "play 2 0",
            "potion use 0",
            "potion use 1 2",
            "end",
            "skip",
            "proceed",
            "leave"
        ]
    );
}

#[test]
fn recording_shop_leave_marks_the_floor_completed() {
    let raw = json!({
        "available_commands": ["leave"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_SCREEN", "floor": 5}
    });
    let mut session = AutoPlaySession::default();
    let state = state(raw);

    record_executed_action(&mut session, &state, &AutoPlayAction::Leave);

    assert_eq!(session.completed_shop_floor, Some(5));
}

#[test]
fn recording_shop_purchase_does_not_complete_the_floor() {
    let raw = json!({
        "available_commands": ["choose", "leave"],
        "ready_for_command": true,
        "game_state": {"screen_type": "SHOP_SCREEN", "floor": 5}
    });
    let mut session = AutoPlaySession::default();
    let state = state(raw);

    record_executed_action(&mut session, &state, &AutoPlayAction::Choose(0));

    assert_eq!(session.completed_shop_floor, None);
}

// ─── execute_action_to Edge cases ─────────────────────────────────

#[test]
fn execute_each_action_type_in_isolation() {
    let mut buf = Vec::new();
    execute_action_to(
        &mut buf,
        &AutoPlayAction::Play {
            hand_index: 0,
            target_index: None,
        },
    );
    assert_eq!(String::from_utf8(buf).unwrap().trim(), "play 1");

    let mut buf = Vec::new();
    execute_action_to(
        &mut buf,
        &AutoPlayAction::Play {
            hand_index: 1,
            target_index: Some(0),
        },
    );
    assert_eq!(String::from_utf8(buf).unwrap().trim(), "play 2 0");

    let mut buf = Vec::new();
    execute_action_to(&mut buf, &AutoPlayAction::Choose(0));
    assert_eq!(String::from_utf8(buf).unwrap().trim(), "choose 0");

    let mut buf = Vec::new();
    execute_action_to(
        &mut buf,
        &AutoPlayAction::Drink {
            slot_index: 0,
            target_index: None,
        },
    );
    assert_eq!(String::from_utf8(buf).unwrap().trim(), "potion use 0");

    let mut buf = Vec::new();
    execute_action_to(&mut buf, &AutoPlayAction::End);
    assert_eq!(String::from_utf8(buf).unwrap().trim(), "end");

    let mut buf = Vec::new();
    execute_action_to(&mut buf, &AutoPlayAction::Skip);
    assert_eq!(String::from_utf8(buf).unwrap().trim(), "skip");

    let mut buf = Vec::new();
    execute_action_to(&mut buf, &AutoPlayAction::Proceed);
    assert_eq!(String::from_utf8(buf).unwrap().trim(), "proceed");

    let mut buf = Vec::new();
    execute_action_to(&mut buf, &AutoPlayAction::Leave);
    assert_eq!(String::from_utf8(buf).unwrap().trim(), "leave");
}
