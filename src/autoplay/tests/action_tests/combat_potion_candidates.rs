use super::*;

#[test]
fn combat_candidates_skip_unusable_potion() {
    use crate::state::PotionInfo;
    let s = NormalizedState {
        screen_type: Some(ScreenType::None),
        potions: vec![PotionInfo {
            id: None,
            slot: 0,
            name: "Bad".into(),
            description: "".into(),
            price: None,
            can_use: false,
            can_discard: true,
            requires_target: false,
        }],
        monsters: vec![crate::state::MonsterInfo {
            name: "Test".into(),
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
    let raw = json!({
        "available_commands": ["end", "potion"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "hand": [],
                "monsters": [{"name": "Test", "current_hp": 10, "max_hp": 10, "block": 0, "intent": "ATTACK", "is_gone": false}],
                "player": {"energy": 0, "block": 0, "powers": []},
                "turn": 1
            }
        }
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &s,
    );
    assert!(candidates.iter().all(|c| c.kind != "drink"));
}

#[test]
fn combat_candidates_targeted_potion() {
    use crate::state::PotionInfo;
    let s = NormalizedState {
        screen_type: Some(ScreenType::None),
        potions: vec![PotionInfo {
            id: None,
            slot: 0,
            name: "Fire Potion".into(),
            description: "Deal 20 damage.".into(),
            price: None,
            can_use: true,
            can_discard: true,
            requires_target: true,
        }],
        monsters: vec![crate::state::MonsterInfo {
            name: "Test".into(),
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
    let raw = json!({
        "available_commands": ["end", "potion"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "hand": [],
                "monsters": [{"name": "Test", "current_hp": 10, "max_hp": 10, "block": 0, "intent": "ATTACK", "is_gone": false}],
                "player": {"energy": 0, "block": 0, "powers": []},
                "turn": 1
            }
        }
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &s,
    );
    let potion_candidate = candidates.iter().find(|c| c.kind == "drink").unwrap();
    assert_eq!(potion_candidate.action_id, "combat:potion:0");
    assert_eq!(potion_candidate.target_required, Some(true));
}
