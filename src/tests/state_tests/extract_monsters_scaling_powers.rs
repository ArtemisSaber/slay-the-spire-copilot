use super::*;

#[test]
fn monster_is_scaling_detects_strength() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Scaling Enemy",
                    "id": "ScalingEnemy",
                    "current_hp": 30,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "ATTACK",
                    "powers": [
                        {"id": "Strength", "name": "Strength", "amount": 3}
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
    assert!(state.monsters[0].is_scaling);
}

#[test]
fn monster_is_scaling_detects_metallicize() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Armored Enemy",
                    "id": "ArmoredEnemy",
                    "current_hp": 30,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "DEFEND",
                    "powers": [
                        {"id": "Metallicize", "name": "Metallicize", "amount": 4}
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
    assert!(state.monsters[0].is_scaling);
}

#[test]
fn monster_is_scaling_detects_regeneration() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Regen Enemy",
                    "id": "RegenEnemy",
                    "current_hp": 30,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "BUFF",
                    "powers": [
                        {"id": "Regeneration", "name": "Regeneration", "amount": 5}
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
    assert!(state.monsters[0].is_scaling);
}

#[test]
fn monster_is_scaling_detects_plated_armor() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Shelled Enemy",
                    "id": "ShelledEnemy",
                    "current_hp": 30,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "ATTACK_BUFF",
                    "powers": [
                        {"id": "Plated Armor", "name": "Plated Armor", "amount": 10}
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
    assert!(state.monsters[0].is_scaling);
}
