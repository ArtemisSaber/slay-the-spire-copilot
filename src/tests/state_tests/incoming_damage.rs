use super::*;

fn normalize_monsters(monsters: Value) -> NormalizedState {
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": monsters,
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
    NormalizedState::from_raw(&raw, test_locale())
}

#[test]
fn incoming_damage_uses_hit_count_from_real_communication_mod_state() {
    let raw = load_fixture("comm-f16t16-hexaghost-turn.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.monsters[0].damage, Some(6));
    assert_eq!(state.monsters[0].hits, Some(6));
    assert_eq!(state.incoming_damage, 36);
    assert!(state.danger.incoming_lethal);
}

#[test]
fn incoming_damage_multiplies_hits_and_sums_monsters() {
    let state = normalize_monsters(json!([
        {
            "name": "Multi Attacker",
            "intent": "ATTACK",
            "move_adjusted_damage": 4,
            "move_hits": 3
        },
        {
            "name": "Single Attacker",
            "intent": "ATTACK",
            "move_adjusted_damage": 8,
            "move_hits": 1
        }
    ]));

    assert_eq!(state.incoming_damage, 20);
}

#[test]
fn incoming_damage_defaults_missing_hit_count_to_one() {
    let state = normalize_monsters(json!([{
        "name": "Legacy Attacker",
        "intent": "ATTACK",
        "move_adjusted_damage": 9
    }]));

    assert_eq!(state.monsters[0].hits, None);
    assert_eq!(state.incoming_damage, 9);
}

#[test]
fn incoming_damage_excludes_none_intent_and_non_positive_values() {
    let state = normalize_monsters(json!([
        {
            "name": "Attacker",
            "intent": "ATTACK",
            "move_adjusted_damage": 12
        },
        {
            "name": "Sleeper",
            "intent": "NONE",
            "move_adjusted_damage": 99,
            "move_hits": 6
        },
        {
            "name": "Zero Damage",
            "intent": "ATTACK",
            "move_adjusted_damage": 0,
            "move_hits": 4
        },
        {
            "name": "Zero Hits",
            "intent": "ATTACK",
            "move_adjusted_damage": 20,
            "move_hits": 0
        }
    ]));

    assert_eq!(state.incoming_damage, 12);
}

#[test]
fn incoming_damage_saturates_overflowing_external_values() {
    let state = normalize_monsters(json!([{
        "name": "Malformed Attacker",
        "intent": "ATTACK",
        "move_adjusted_damage": i64::MAX,
        "move_hits": 2
    }]));

    assert_eq!(state.incoming_damage, i64::MAX);
}
