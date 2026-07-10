use super::*;

#[test]
fn resolve_combat_drink_potion_not_found() {
    use crate::state::PotionInfo;
    let s = NormalizedState {
        potions: vec![PotionInfo {
            slot: 0,
            name: "Heal".into(),
            description: "".into(),
            price: None,
            can_use: true,
            can_discard: true,
            requires_target: false,
        }],
        ..NormalizedState::default()
    };
    let req = ActionRequest {
        kind: "drink".into(),
        action_id: "combat:potion:5".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_combat(&s, &req), None);
}

#[test]
fn resolve_combat_drink_potion_cannot_use() {
    use crate::state::PotionInfo;
    let s = NormalizedState {
        potions: vec![PotionInfo {
            slot: 0,
            name: "Heal".into(),
            description: "".into(),
            price: None,
            can_use: false,
            can_discard: true,
            requires_target: false,
        }],
        ..NormalizedState::default()
    };
    let req = ActionRequest {
        kind: "drink".into(),
        action_id: "combat:potion:0".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_combat(&s, &req), None);
}

#[test]
fn resolve_combat_drink_missing_target_for_targeted_potion() {
    use crate::state::PotionInfo;
    let s = NormalizedState {
        potions: vec![PotionInfo {
            slot: 0,
            name: "Fire".into(),
            description: "".into(),
            price: None,
            can_use: true,
            can_discard: true,
            requires_target: true,
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
        ..NormalizedState::default()
    };
    let req = ActionRequest {
        kind: "drink".into(),
        action_id: "combat:potion:0".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_combat(&s, &req), None);
}

#[test]
fn resolve_combat_drink_invalid_monster_target() {
    use crate::state::PotionInfo;
    let s = NormalizedState {
        potions: vec![PotionInfo {
            slot: 0,
            name: "Fire".into(),
            description: "".into(),
            price: None,
            can_use: true,
            can_discard: true,
            requires_target: true,
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
        ..NormalizedState::default()
    };
    let req = ActionRequest {
        kind: "drink".into(),
        action_id: "combat:potion:0".into(),
        target_index: Some(99),
    };
    assert_eq!(resolve_requested_combat(&s, &req), None);
}

#[test]
fn resolve_combat_drink_targetless_potion_returns_drink_without_target() {
    use crate::state::PotionInfo;
    let s = NormalizedState {
        screen_type: Some(ScreenType::None),
        potions: vec![PotionInfo {
            slot: 0,
            name: "Heal".into(),
            description: "".into(),
            price: None,
            can_use: true,
            can_discard: true,
            requires_target: false,
        }],
        ..NormalizedState::default()
    };
    let raw = json!({
        "available_commands": ["potion"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "hand": [], "monsters": [], "player": {"energy": 3, "block": 0, "powers": []}, "turn": 1
            }
        }
    });
    assert_eq!(
        resolve_requested_action(
            &AutoPlayControl::default_enabled(),
            &AutoPlaySession::default(),
            &command_state(&raw),
            &s,
            &request("drink", "combat:potion:0"),
        ),
        Some(AutoPlayAction::Drink {
            slot_index: 0,
            target_index: None
        })
    );
}

#[test]
fn resolve_combat_drink_bad_slot_parse() {
    let req = ActionRequest {
        kind: "drink".into(),
        action_id: "combat:potion:abc".into(),
        target_index: None,
    };
    assert_eq!(
        resolve_requested_combat(&NormalizedState::default(), &req),
        None
    );
}

#[test]
fn resolve_combat_drink_targeted_potion_success() {
    use crate::state::PotionInfo;
    let s = NormalizedState {
        potions: vec![PotionInfo {
            slot: 0,
            name: "Fire Potion".into(),
            description: "".into(),
            price: None,
            can_use: true,
            can_discard: true,
            requires_target: true,
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
        ..NormalizedState::default()
    };
    let req = targeted_request("drink", "combat:potion:0", 0);
    assert_eq!(
        resolve_requested_combat(&s, &req),
        Some(AutoPlayAction::Drink {
            slot_index: 0,
            target_index: Some(0)
        })
    );
}
