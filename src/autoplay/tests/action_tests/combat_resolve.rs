use super::*;

#[test]
fn resolve_combat_missing_target_for_targeted_card() {
    use crate::state::CardInfo;
    let s = NormalizedState {
        hand: vec![CardInfo {
            id: "Strike".into(),
            name: "Strike".into(),
            cost: 1,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: Some("s-1".into()),
            description: "".into(),
            price: None,
            playable: true,
            has_target: true,
        }],
        monsters: vec![crate::state::MonsterInfo {
            name: "Worm".into(),
            index: 0,
            current_hp: Some(10),
            max_hp: Some(10),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(5),
            hits: Some(1),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
            monster_id: None,
        }],
        energy: Some(3),
        ..NormalizedState::default()
    };
    let req = ActionRequest {
        kind: "play".into(),
        action_id: "combat:play:s-1".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_combat(&s, &req), None);
}

#[test]
fn resolve_combat_invalid_monster_index() {
    use crate::state::CardInfo;
    let s = NormalizedState {
        hand: vec![CardInfo {
            id: "Strike".into(),
            name: "Strike".into(),
            cost: 1,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: Some("s-1".into()),
            description: "".into(),
            price: None,
            playable: true,
            has_target: true,
        }],
        monsters: vec![crate::state::MonsterInfo {
            name: "Worm".into(),
            index: 0,
            current_hp: Some(10),
            max_hp: Some(10),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(5),
            hits: Some(1),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
            monster_id: None,
        }],
        energy: Some(3),
        ..NormalizedState::default()
    };
    let req = ActionRequest {
        kind: "play".into(),
        action_id: "combat:play:s-1".into(),
        target_index: Some(99),
    };
    assert_eq!(resolve_requested_combat(&s, &req), None);
}

#[test]
fn resolve_combat_uuid_not_found() {
    use crate::state::CardInfo;
    let s = NormalizedState {
        hand: vec![CardInfo {
            id: "Defend".into(),
            name: "Defend".into(),
            cost: 1,
            card_type: "SKILL".into(),
            upgraded: false,
            uuid: Some("d-1".into()),
            description: "".into(),
            price: None,
            playable: true,
            has_target: false,
        }],
        energy: Some(3),
        ..NormalizedState::default()
    };
    let req = ActionRequest {
        kind: "play".into(),
        action_id: "combat:play:bad-uuid".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_combat(&s, &req), None);
}

#[test]
fn resolve_combat_rejects_unknown_kind() {
    let s = NormalizedState {
        energy: Some(3),
        ..NormalizedState::default()
    };
    let req = ActionRequest {
        kind: "unknown".into(),
        action_id: "combat:end".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_combat(&s, &req), None);
}

#[test]
fn resolve_combat_play_targetless_returns_play_without_target() {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [
                    {"id": "Defend", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "def-1", "has_target": false}
                ],
                "monsters": [
                    {"name": "Worm", "current_hp": 10, "max_hp": 10, "block": 0, "intent": "ATTACK", "is_gone": false}
                ]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_combat = true;
    assert_eq!(
        resolve_requested_action(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
            &request("play", "combat:play:def-1"),
        ),
        Some(AutoPlayAction::Play {
            hand_index: 0,
            target_index: None
        })
    );
}
