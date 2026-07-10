use super::*;

#[test]
fn normalize_card_reward_state() {
    let raw = load_fixture("card-reward-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(
        state.screen_type.as_ref().map(|st| st.as_str()),
        Some("CARD_REWARD")
    );
    assert_eq!(state.card_reward_choices.len(), 3);
    assert!(state.card_reward_choices.iter().any(|c| c.id == "Uppercut"));
    assert!(state.card_reward_choices.iter().any(|c| c.id == "Anger"));
    assert!(state.card_reward_choices.iter().any(|c| c.id == "Headbutt"));

    assert!(state.hand.is_empty());
    assert!(state.monsters.is_empty());
    assert_eq!(state.incoming_damage, 0);
    assert_eq!(state.danger.level, DangerLevel::Safe);

    assert_eq!(state.master_cards.len(), 1);
    assert!(state.skip_available);
}

#[test]
fn normalize_rest_state() {
    let raw = load_fixture("rest-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(
        state.screen_type.as_ref().map(|st| st.as_str()),
        Some("REST")
    );
    assert_eq!(
        state.rest_options,
        vec!["rest".to_string(), "smith".to_string(), "toke".to_string()]
    );
    assert_eq!(state.current_hp, Some(25));
    assert_eq!(state.max_hp, Some(75));
    assert_eq!(state.floor, Some(5));
    assert_eq!(state.gold, Some(120));

    assert_eq!(state.master_cards.len(), 9);
    assert!(state.potions.is_empty());
    assert!(state.monsters.is_empty());
    assert!(state.hand_cards.is_empty());
    assert_eq!(state.danger.level, DangerLevel::Caution);
}

#[test]
fn normalize_boss_relic_choices() {
    let raw = serde_json::json!({
        "in_game": true,
        "game_state": {
            "screen_type": "BOSS_REWARD",
            "screen_state": {
                "relics": [
                    {"id": "Snecko Eye", "name": "Snecko Eye"},
                    {"id": "Runic Dome", "name": "Runic Dome"},
                    {"id": "Cursed Key", "name": "Cursed Key"}
                ]
            },
            "deck": [],
            "relics": [],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 17,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(
        state.screen_type.as_ref().map(|st| st.as_str()),
        Some("BOSS_REWARD")
    );
    assert_eq!(state.boss_relic_choices.len(), 3);
    assert!(
        state
            .boss_relic_choices
            .iter()
            .any(|r| r.name == "Snecko Eye")
    );
    assert!(
        state
            .boss_relic_choices
            .iter()
            .any(|r| r.name == "Runic Dome")
    );
    assert!(
        state
            .boss_relic_choices
            .iter()
            .any(|r| r.name == "Cursed Key")
    );
}

#[test]
fn normalize_shop_state() {
    let raw = load_fixture("shop-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(
        state.screen_type.as_ref().map(|st| st.as_str()),
        Some("SHOP_SCREEN")
    );
    assert_eq!(state.floor, Some(5));
    assert_eq!(state.gold, Some(190));
    assert!(state.purge_available);
    assert_eq!(state.purge_cost, Some(75));
    assert!(!state.shop_cards.is_empty());
    assert!(!state.shop_potions.is_empty());

    let card_ids: Vec<&str> = state.shop_cards.iter().map(|c| c.id.as_str()).collect();
    assert!(card_ids.contains(&"Sever Soul"));
    assert!(card_ids.contains(&"Armaments"));
    assert!(card_ids.contains(&"Fire Breathing"));

    for card in &state.shop_cards {
        assert!(
            card.price.is_some(),
            "shop card {} should have a price",
            card.id
        );
    }

    assert!(!state.master_cards.is_empty());
    assert!(state.hand.is_empty());
    assert!(state.monsters.is_empty());
}

#[test]

fn normalize_combat_state() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(
        state.screen_type.as_ref().map(|st| st.as_str()),
        Some("NONE")
    );
    assert_eq!(state.character.as_deref(), Some("IRONCLAD"));
    assert_eq!(state.seed, Some(-3047511808784702860));
    assert_eq!(state.ascension_level, Some(20));
    assert_eq!(state.floor, Some(1));
    assert_eq!(state.current_hp, Some(68));
    assert_eq!(state.max_hp, Some(75));
    assert_eq!(state.gold, Some(99));
    assert_eq!(state.energy, Some(3));
    assert_eq!(state.block, Some(6));

    assert_eq!(state.hand.len(), 3);
    assert_eq!(state.monsters.len(), 1);
    assert_eq!(state.relics.len(), 2);
    assert_eq!(state.potions.len(), 1);
    assert!(state.card_reward_choices.is_empty());

    assert_eq!(state.incoming_damage, 12);
    assert_eq!(state.danger.level, DangerLevel::Caution);
    assert!(state.danger.any_monster_attacking);

    let jaw_worm = &state.monsters[0];
    assert_eq!(jaw_worm.name, "Jaw Worm");
    assert_eq!(jaw_worm.index, 0);
    assert_eq!(jaw_worm.intent.as_deref(), Some("ATTACK"));
    assert!(jaw_worm.is_scaling);

    assert_eq!(state.hand_cards.len(), 3);
    assert_eq!(state.draw_pile.len(), 2);
    assert_eq!(state.discard_pile.len(), 0);
    assert_eq!(state.exhaust_cards.len(), 0);
    assert_eq!(state.master_cards.len(), 1);
}
