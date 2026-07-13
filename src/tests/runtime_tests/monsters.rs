use super::*;

#[test]
fn has_monsters_mixed_active_and_gone() {
    let state = json!({
        "game_state": {
            "combat_state": {
                "monsters": [
                    {"id": "a", "is_gone": true},
                    {"id": "b", "is_gone": false},
                    {"id": "c", "is_gone": true}
                ]
            }
        }
    });
    assert!(has_monsters(&state));
}

#[test]
fn has_monsters_missing_is_gone_field_treated_as_active() {
    let state = json!({
        "game_state": {
            "combat_state": {
                "monsters": [
                    {"id": "a"},
                    {"id": "b", "is_gone": true}
                ]
            }
        }
    });
    assert!(has_monsters(&state));
}

#[test]
fn has_monsters_all_missing_is_gone_all_active() {
    let state = json!({
        "game_state": {
            "combat_state": {
                "monsters": [
                    {"id": "a"},
                    {"id": "b"}
                ]
            }
        }
    });
    assert!(has_monsters(&state));
}

#[test]
fn has_monsters_combat_state_without_monsters_key() {
    let state = json!({
        "game_state": {
            "combat_state": {"turn": 1}
        }
    });
    assert!(!has_monsters(&state));
}

#[test]
fn has_monsters_monsters_not_an_array() {
    let state = json!({
        "game_state": {
            "combat_state": {
                "monsters": "not-an-array"
            }
        }
    });
    assert!(!has_monsters(&state));
}

#[test]
fn has_monsters_null_monsters() {
    let state = json!({
        "game_state": {
            "combat_state": {
                "monsters": null
            }
        }
    });
    assert!(!has_monsters(&state));
}

#[test]
fn has_monsters_no_combat_state_key() {
    let state = json!({"game_state": {"screen_type": "NONE"}});
    assert!(!has_monsters(&state));
}

#[test]
fn has_monsters_empty_object() {
    assert!(!has_monsters(&json!({})));
}
