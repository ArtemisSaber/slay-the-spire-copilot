use super::*;

#[test]
fn card_reward_filters_potion_slot() {
    let raw = load_fixture("card-reward-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(
        !state
            .potions
            .iter()
            .any(|p| p.name.contains("Potion Slot") || p.name == "?")
    );
}

#[test]
fn card_info_upgraded() {
    let json: Value = serde_json::from_str(
        r#"{"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK","upgrades":1}"#,
    )
    .unwrap();
    let card = CardInfo::from_json(&json);
    assert!(card.upgraded);
}

#[test]
fn card_info_missing_cost_and_type() {
    let json: Value = serde_json::from_str(r#"{"id":"Strike_R","name":"Strike"}"#).unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.cost, 0);
    assert_eq!(card.card_type, "?");
}

#[test]
fn card_info_uuid() {
    let json: Value =
        serde_json::from_str(r#"{"id":"Strike_R","name":"Strike","uuid":"abc-123"}"#).unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.uuid.as_deref(), Some("abc-123"));
}

#[test]
fn card_info_has_target_true() {
    let json: Value = serde_json::from_str(
        r#"{"id":"Bash","name":"Bash","cost":2,"type":"ATTACK","has_target":true}"#,
    )
    .unwrap();
    let card = CardInfo::from_json(&json);
    assert!(card.has_target);
}

#[test]
fn card_info_has_target_defaults_to_false() {
    let json: Value =
        serde_json::from_str(r#"{"id":"Defend_R","name":"Defend","cost":1,"type":"SKILL"}"#)
            .unwrap();
    let card = CardInfo::from_json(&json);
    assert!(!card.has_target);
}

#[test]
fn card_info_upgraded_false_when_zero() {
    let json: Value = serde_json::from_str(
        r#"{"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK","upgrades":0}"#,
    )
    .unwrap();
    let card = CardInfo::from_json(&json);
    assert!(!card.upgraded);
}

#[test]
fn card_info_missing_id() {
    let json: Value =
        serde_json::from_str(r#"{"name":"Mystery","cost":1,"type":"SKILL"}"#).unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.id, "?");
    assert_eq!(card.name, "Mystery");
}

#[test]
fn card_info_missing_name() {
    let json: Value = serde_json::from_str(r#"{"id":"TestCard","cost":2,"type":"POWER"}"#).unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.id, "TestCard");
    assert_eq!(card.name, "?");
}

#[test]
fn card_info_price_parsed() {
    let json: Value = serde_json::from_str(
        r#"{"id":"Armaments","name":"Armaments","cost":1,"type":"SKILL","price":51}"#,
    )
    .unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.price, Some(51));
}

#[test]
fn card_info_price_none_when_missing() {
    let json: Value =
        serde_json::from_str(r#"{"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK"}"#)
            .unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.price, None);
}

#[test]
fn card_info_not_playable() {
    let json: Value = serde_json::from_str(
        r#"{"id":"Barricade","name":"Barricade","cost":3,"type":"POWER","is_playable":false}"#,
    )
    .unwrap();
    let card = CardInfo::from_json(&json);
    assert!(!card.playable);
}

#[test]
fn card_in_play_parsed_from_combat() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {
                "max_cards": 1,
                "can_pick_zero": false
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
                    "type": "SKILL",
                    "upgrades": 0
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

    assert!(state.card_in_play.is_some());
    let card = state.card_in_play.unwrap();
    assert_eq!(card.id, "Burning Pact");
    assert_eq!(card.name, "Burning Pact");
    assert_eq!(card.cost, 1);
    assert_eq!(card.card_type, "SKILL");
}

#[test]
fn card_in_play_none_when_not_present() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(state.card_in_play.is_none());
}

#[test]
fn current_action_parsed_from_game_state() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {
                "max_cards": 1
            },
            "current_action": "PlayCard",
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.current_action.as_deref(), Some("PlayCard"));
}

#[test]
fn current_action_none_when_missing() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(state.current_action.is_none());
}
