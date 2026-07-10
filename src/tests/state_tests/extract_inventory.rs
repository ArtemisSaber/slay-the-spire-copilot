use super::*;

#[test]
fn potion_slot_counting_with_gaps() {
    let raw = load_fixture("comm-gapped-potions.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.potions.len(), 2);
    assert_eq!(state.empty_potion_slots, 1);

    let potion_names: Vec<&str> = state.potions.iter().map(|p| p.name.as_str()).collect();
    assert!(!potion_names.contains(&"Potion Slot"));
    assert!(!potion_names.contains(&"?"));
    assert!(!potion_names.is_empty());
}

#[test]
fn potion_slot_counting_with_no_empty_slots() {
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
                {"requires_target": false, "can_use": true, "can_discard": true, "name": "Energy Potion", "id": "Energy Potion"},
                {"requires_target": false, "can_use": true, "can_discard": true, "name": "Block Potion", "id": "Block Potion"},
                {"requires_target": false, "can_use": true, "can_discard": true, "name": "Fire Potion", "id": "Fire Potion"}
            ]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.potions.len(), 3);
    assert_eq!(state.empty_potion_slots, 0);
}

#[test]
fn potions_empty_array_produces_no_slots() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "potions": []
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(state.potions.is_empty());
    assert_eq!(state.empty_potion_slots, 0);
}

#[test]
fn relic_info_from_string_array() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "relics": ["Burning Blood", "Neow's Lament"]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.relics.len(), 2);
    assert_eq!(state.relics[0].name, "Burning Blood");
    assert_eq!(state.relics[0].id, "");
    assert_eq!(state.relics[1].name, "Neow's Lament");
}

#[test]
fn relic_info_counter_filtered_when_negative() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "relics": [
                {"id": "Burning Blood", "name": "Burning Blood", "counter": -1},
                {"id": "Ornamental Fan", "name": "Ornamental Fan", "counter": 3},
                {"id": "NeowsBlessing", "name": "Neow's Blessing", "counter": -2}
            ]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    let burning = state
        .relics
        .iter()
        .find(|r| r.id == "Burning Blood")
        .unwrap();
    assert_eq!(burning.counter, None);

    let fan = state
        .relics
        .iter()
        .find(|r| r.id == "Ornamental Fan")
        .unwrap();
    assert_eq!(fan.counter, Some(3));

    let neow = state
        .relics
        .iter()
        .find(|r| r.id == "NeowsBlessing")
        .unwrap();
    assert_eq!(neow.counter, None);
}

#[test]
fn relic_info_defaults_for_missing_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "relics": [
                {"id": "TestRelic"}
            ]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.relics.len(), 1);
    assert_eq!(state.relics[0].id, "TestRelic");
    assert_eq!(state.relics[0].name, "?");
    assert_eq!(state.relics[0].description, "");
    assert_eq!(state.relics[0].counter, None);
    assert_eq!(state.relics[0].price, None);
}

#[test]
fn relic_info_with_description() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "relics": [
                {
                    "id": "Burning Blood",
                    "name": "Burning Blood",
                    "description": "At the end of combat, heal 6 HP."
                }
            ]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(
        state.relics[0].description,
        "At the end of combat, heal 6 HP."
    );
}

#[test]
fn relic_info_shop_relic_with_price() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "SHOP_SCREEN",
            "screen_state": {
                "cards": [],
                "potions": [],
                "relics": [
                    {
                        "id": "Bronze Scales",
                        "name": "Bronze Scales",
                        "description": "When you are attacked, deal 3 damage back.",
                        "price": 150
                    },
                    {
                        "id": "Oddly Smooth Stone",
                        "name": "Oddly Smooth Stone",
                        "description": "Gain 1 Dexterity.",
                        "price": 251
                    }
                ]
            },
            "current_hp": 70,
            "max_hp": 75,
            "gold": 300,
            "floor": 10,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.shop_relics.len(), 2);
    assert_eq!(state.shop_relics[0].name, "Bronze Scales");
    assert_eq!(state.shop_relics[0].price, Some(150));
    assert_eq!(state.shop_relics[1].name, "Oddly Smooth Stone");
    assert_eq!(state.shop_relics[1].price, Some(251));
}
