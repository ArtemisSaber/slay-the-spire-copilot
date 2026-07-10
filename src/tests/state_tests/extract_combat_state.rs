use super::*;

#[test]
fn orbs_from_raw_json() {
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
                    "orbs": [
                        {"id": "Lightning", "amount": 2},
                        {"id": "Frost", "amount": 1}
                    ]
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.orbs.len(), 2);
    assert_eq!(state.orbs[0].id, "Lightning");
    assert_eq!(state.orbs[0].amount, 2);
    assert_eq!(state.orbs[1].id, "Frost");
    assert_eq!(state.orbs[1].amount, 1);
}

#[test]
fn orbs_empty_when_missing() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(state.orbs.is_empty());
}

#[test]
fn stance_from_powers_wrath() {
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
                    "powers": [
                        {"id": "Wrath", "name": "Wrath", "amount": 1}
                    ],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.stance.as_deref(), Some("Wrath"));
}

#[test]
fn stance_from_powers_calm() {
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
                    "powers": [
                        {"id": "Calm", "name": "Calm", "amount": 1}
                    ],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.stance.as_deref(), Some("Calm"));
}

#[test]
fn stance_from_powers_divinity() {
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
                    "powers": [
                        {"id": "Divinity", "name": "Divinity", "amount": 1}
                    ],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.stance.as_deref(), Some("Divinity"));
}

#[test]
fn stance_none_when_no_stance_powers() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert_eq!(state.stance, None);
}

#[test]
fn stance_none_when_stance_amount_zero() {
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
                    "powers": [
                        {"id": "Wrath", "name": "Wrath", "amount": 0}
                    ],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.stance, None);
}
