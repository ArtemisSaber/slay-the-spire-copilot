use super::*;
use crate::test_utils::card;

#[test]
fn stable_hash_same_state_same_hash() {
    let raw = load_fixture("combat-state.json");
    let state1 = NormalizedState::from_raw(&raw, test_locale());
    let state2 = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state1.stable_hash(), state2.stable_hash());
}

#[test]
fn stable_hash_different_state_different_hash() {
    let combat = load_fixture("combat-state.json");
    let reward = load_fixture("card-reward-state.json");

    let state1 = NormalizedState::from_raw(&combat, test_locale());
    let state2 = NormalizedState::from_raw(&reward, test_locale());

    assert_ne!(state1.stable_hash(), state2.stable_hash());
}

#[test]
fn stable_hash_produces_hex() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    let hash = state.stable_hash();

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn stable_projection_matches_golden_contract() {
    let state = NormalizedState {
        screen_type: Some(ScreenType::CardReward),
        room_type: Some(RoomType::MonsterRoom),
        character: Some("IRONCLAD".into()),
        floor: Some(3),
        current_hp: Some(40),
        max_hp: Some(80),
        gold: Some(99),
        energy: Some(2),
        block: Some(5),
        incoming_damage: 7,
        powers: vec![
            PowerInfo {
                id: "z".into(),
                name: "Strength".into(),
                amount: 2,
            },
            PowerInfo {
                id: "a".into(),
                name: "Dexterity".into(),
                amount: 1,
            },
        ],
        hand: vec![
            CardInfo {
                upgraded: true,
                ..card("Bash", "Bash", 2, "ATTACK")
            },
            card("Zap", "Zap", 1, "SKILL"),
        ],
        monsters: vec![
            MonsterInfo {
                name: "Slime".into(),
                current_hp: Some(8),
                max_hp: Some(12),
                intent: Some("ATTACK".into()),
                damage: Some(5),
                ..MonsterInfo::default()
            },
            MonsterInfo {
                name: "Cultist".into(),
                current_hp: Some(20),
                max_hp: Some(48),
                intent: Some("BUFF".into()),
                damage: None,
                ..MonsterInfo::default()
            },
        ],
        card_reward_choices: vec![
            card("Uppercut", "Uppercut", 2, "ATTACK"),
            card("Anger", "Anger", 0, "ATTACK"),
        ],
        relics: vec![RelicInfo {
            id: "Burning Blood".into(),
            name: "Burning Blood".into(),
            description: "Heal after combat.".into(),
            counter: None,
            price: None,
        }],
        deck_names: vec!["Strike".into(), "Bash".into()],
        skip_available: true,
        ..NormalizedState::default()
    };

    let expected = json!({
        "screen_type": "CARD_REWARD",
        "room_type": "MonsterRoom",
        "character": "IRONCLAD",
        "floor": 3,
        "current_hp": 40,
        "max_hp": 80,
        "gold": 99,
        "energy": 2,
        "block": 5,
        "incoming_damage": 7,
        "powers": [
            {"name": "Dexterity", "amount": 1},
            {"name": "Strength", "amount": 2}
        ],
        "hand": [
            {"id": "Bash", "cost": 2, "type": "ATTACK", "upgraded": true},
            {"id": "Zap", "cost": 1, "type": "SKILL", "upgraded": false}
        ],
        "monsters": [
            {"name": "Cultist", "current_hp": 20, "max_hp": 48, "intent": "BUFF"},
            {"name": "Slime", "current_hp": 8, "max_hp": 12, "intent": "ATTACK", "damage": 5}
        ],
        "card_reward_choices": [
            {"id": "Anger"},
            {"id": "Uppercut"}
        ],
        "boss_relic_choices": [],
        "event_choices": [],
        "relics": [
            {"name": "Burning Blood", "description": "Heal after combat."}
        ],
        "potions": [],
        "deck_names": ["Bash", "Strike"],
        "rest_options": [],
        "skip_available": true,
        "shop_cards": [],
        "shop_relics": []
    });

    assert_eq!(state.to_stable_value(), expected);

    let expected_hash = hash_bytes(serde_json::to_string(&expected).unwrap().as_bytes());
    assert_eq!(state.stable_hash(), expected_hash);
}
