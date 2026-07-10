use super::*;

#[test]
fn monster_is_gone_filtered_out() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [
                    {
                        "name": "Active Enemy",
                        "id": "ActiveEnemy",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0,
                        "intent": "ATTACK",
                        "is_gone": false
                    },
                    {
                        "name": "Gone Enemy",
                        "id": "GoneEnemy",
                        "current_hp": 0,
                        "max_hp": 15,
                        "block": 0,
                        "intent": "NONE",
                        "is_gone": true
                    }
                ],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.monsters.len(), 1);
    assert_eq!(state.monsters[0].name, "Active Enemy");
}

#[test]
fn monster_is_gone_defaults_to_false() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [
                    {
                        "name": "Test Enemy",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0
                    }
                ],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.monsters.len(), 1);
}

#[test]
fn monster_not_scaling_without_scaling_powers() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Normal Enemy",
                    "id": "NormalEnemy",
                    "current_hp": 30,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "ATTACK",
                    "powers": [
                        {"id": "Vulnerable", "name": "Vulnerable", "amount": 1}
                    ]
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(!state.monsters[0].is_scaling);
}
