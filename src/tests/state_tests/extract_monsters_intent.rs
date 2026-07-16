use super::*;

#[test]
fn monster_id_from_combat_fixture() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    let jaw_worm = &state.monsters[0];
    assert_eq!(jaw_worm.monster_id.as_deref(), Some("JawWorm"));
}

#[test]
fn monster_id_defaults_to_none_when_missing() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Test Enemy",
                    "current_hp": 10,
                    "max_hp": 20,
                    "block": 0
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
    assert_eq!(state.monsters[0].monster_id, None);
}

#[test]
fn monster_intent_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Test Enemy",
                    "id": "TestEnemy",
                    "current_hp": 10,
                    "max_hp": 20,
                    "block": 0,
                    "intent": "ATTACK_BUFF",
                    "move_adjusted_damage": 15,
                    "move_hits": 2
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

    assert_eq!(state.monsters[0].intent.as_deref(), Some("ATTACK_BUFF"));
    assert_eq!(state.monsters[0].damage, Some(15));
    assert_eq!(state.monsters[0].hits, Some(2));
}
