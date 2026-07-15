use super::*;

#[test]
fn potion_action_id_uses_raw_slot_with_gaps() {
    use crate::state::PotionInfo;
    let state = NormalizedState {
        screen_type: Some(ScreenType::None),
        potions: vec![
            PotionInfo {
                id: None,
                slot: 1,
                name: "能量药水".into(),
                description: "获得 2 点能量。".into(),
                price: None,
                can_use: true,
                can_discard: true,
                requires_target: false,
            },
            PotionInfo {
                id: None,
                slot: 2,
                name: "格挡药水".into(),
                description: "获得 12 点 格挡 。".into(),
                price: None,
                can_use: true,
                can_discard: true,
                requires_target: false,
            },
        ],
        empty_potion_slots: 1,
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
        energy: Some(0),
        ..NormalizedState::default()
    };
    let raw = serde_json::json!({
        "available_commands": ["end", "potion"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "hand": [],
                "monsters": [{"name": "Test", "current_hp": 10, "max_hp": 10, "block": 0, "intent": "ATTACK", "move_adjusted_damage": 5, "move_hits": 1, "powers": [], "is_gone": false, "half_dead": false}],
                "player": {"energy": 0, "block": 0, "powers": []},
                "turn": 1
            },
            "potions": [
                {"id": "Potion Slot", "can_use": false},
                {"id": "Energy Potion", "can_use": true, "name": "能量药水"},
                {"id": "Block Potion", "can_use": true, "name": "格挡药水"}
            ]
        }
    });
    let command_state = command_state(&raw);
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );
    let potion_candidates: Vec<_> = candidates.iter().filter(|c| c.kind == "drink").collect();
    assert_eq!(potion_candidates.len(), 2);
    assert_eq!(
        potion_candidates[0].action_id, "combat:potion:1",
        "energy potion should use raw slot 1"
    );
    assert_eq!(
        potion_candidates[1].action_id, "combat:potion:2",
        "block potion should use raw slot 2"
    );
}
