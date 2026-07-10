use super::*;

#[test]
fn stable_hash_includes_boss_relic_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "BOSS_REWARD",
            "screen_state": {
                "relics": [
                    {"id": "Snecko Eye", "name": "Snecko Eye", "description": "Draw 7, confuse."},
                    {"id": "Runic Dome", "name": "Runic Dome", "description": "+1 energy, no intents."}
                ]
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 16,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_shop_fields() {
    let raw = load_fixture("shop-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_grid_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "GRID",
            "screen_state": {
                "cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "for_upgrade": true,
                "for_transform": false,
                "for_purge": false,
                "num_cards": 1
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_hand_select_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {
                "max_cards": 1,
                "can_pick_zero": true
            },
            "combat_state": {
                "hand": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "monsters": [],
                "card_in_play": {
                    "id": "Burning Pact",
                    "name": "Burning Pact",
                    "cost": 1,
                    "type": "SKILL"
                },
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
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_event_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_id": "GoldenIdol",
                "event_name": "Golden Idol",
                "body": "A golden idol sits on a pedestal.",
                "options": [
                    {"label": "Take", "text": "Take the idol and become cursed."},
                    {"label": "Leave", "text": "Leave it alone."}
                ]
            },
            "current_hp": 52,
            "max_hp": 75,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_map_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {
                "first_node_chosen": true,
                "current_node": {"x": 3, "y": 7}
            },
            "map": [
                {"symbol": "M", "x": 3, "y": 7, "children": [{"x": 4, "y": 8}]}
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_empty_potion_slots() {
    let raw = load_fixture("comm-gapped-potions.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
    assert!(state.empty_potion_slots > 0);
}

#[test]
fn stable_hash_empty_potion_slots_zero_not_included() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "potions": [
                {"id": "Energy Potion", "name": "Energy Potion"}
            ]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.empty_potion_slots, 0);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_current_action() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {},
            "current_action": "PlayCard",
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
    assert_eq!(state.current_action.as_deref(), Some("PlayCard"));
}

#[test]
fn stable_hash_excludes_current_action_when_none() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 60,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(state.current_action.is_none());
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}
