use super::*;

#[test]
fn shop_prompt_contains_mode_tag() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::ShopScreen),
        gold: Some(190),
        shop_cards: vec![CardInfo {
            name: "断魂斩".into(),
            cost: 2,
            price: Some(79),
            ..card("Sever Soul", "断魂斩", 2, "ATTACK")
        }],
        shop_relics: vec![RelicInfo {
            name: "铲子".into(),
            description: "现在你可以在休息处 挖掘 遗物。".into(),
            price: Some(286),
            ..RelicInfo {
                id: "Shovel".into(),
                name: "铲子".into(),
                description: "挖掘".into(),
                counter: None,
                price: None,
            }
        }],
        shop_potions: vec![PotionInfo {
            id: None,
            slot: 0,
            name: "再生药水".into(),
            description: "获得 5 层 再生 。".into(),
            price: Some(79),
            can_use: false,
            can_discard: false,
            requires_target: false,
        }],
        purge_available: true,
        purge_cost: Some(75),
        master_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        deck_names: vec!["打击".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);

    assert!(prompt.contains("[mode: shop]"));
    assert!(prompt.contains("=== 商店 ==="));
    assert!(prompt.contains("=== 待购卡牌 ==="));
    assert!(prompt.contains("断魂斩(2费/攻击)"));
    assert!(prompt.contains("79 gold"));
    assert!(prompt.contains("=== 待购遗物 ==="));
    assert!(prompt.contains("A. 铲子"));
    assert!(prompt.contains("286 gold"));
    assert!(prompt.contains("=== 待购药水 ==="));
    assert!(prompt.contains("再生药水"));
    assert!(prompt.contains("79 gold"));
    assert!(prompt.contains("=== 删牌服务 ==="));
    assert!(prompt.contains("Remove a card for 75 gold"));
    assert!(prompt.contains("打击"));
}

#[test]
fn shop_state_parses_from_fixture() {
    let raw = crate::test_utils::load_fixture("shop-state.json");
    let locale = test_locale();
    let state = NormalizedState::from_raw(&raw, &locale);

    assert_eq!(
        state.screen_type.as_ref().map(|st| st.as_str()),
        Some("SHOP_SCREEN")
    );
    assert_eq!(state.gold, Some(190));
    assert_eq!(state.current_hp, Some(33));
    assert_eq!(state.max_hp, Some(80));

    assert_eq!(state.shop_cards.len(), 7);
    assert!(
        state
            .shop_cards
            .iter()
            .any(|c| c.name == "断魂斩" && c.price == Some(79))
    );
    assert!(
        state
            .shop_cards
            .iter()
            .any(|c| c.name == "武装" && c.price == Some(51))
    );

    assert_eq!(state.shop_relics.len(), 3);
    assert!(
        state
            .shop_relics
            .iter()
            .any(|r| r.name == "铲子" && r.price == Some(286))
    );
    assert!(
        state
            .shop_relics
            .iter()
            .any(|r| r.name == "硫磺" && r.price == Some(151))
    );

    assert_eq!(state.shop_potions.len(), 3);
    assert!(
        state
            .shop_potions
            .iter()
            .any(|p| p.name == "鲜血药水" && p.price == Some(50))
    );

    assert!(state.purge_available);
    assert_eq!(state.purge_cost, Some(75));
}

#[test]
fn shop_prompt_from_fixture_includes_real_data() {
    let raw = crate::test_utils::load_fixture("shop-state.json");
    let locale = test_locale();
    let state = NormalizedState::from_raw(&raw, &locale);
    let prompt = build_prompt(&state, &locale, false);

    assert!(prompt.contains("[mode: shop]"));
    assert!(prompt.contains("=== 商店 ==="));
    assert!(prompt.contains("血量：33/80(41%)"));
    assert!(prompt.contains("金币：190"));
    assert!(prompt.contains("=== 待购卡牌 ==="));
    assert!(prompt.contains("A. 断魂斩(2费/攻击)"));
    assert!(prompt.contains("B. 金刚臂(2费/攻击)"));
    assert!(prompt.contains("C. 武装(1费/技能)"));
    assert!(prompt.contains("=== 待购遗物 ==="));
    assert!(prompt.contains("A. 铲子"));
    assert!(prompt.contains("=== 待购药水 ==="));
    assert!(prompt.contains("A. 再生药水"));
    assert!(prompt.contains("=== 删牌服务 ==="));
    assert!(prompt.contains("Remove a card for 75 gold"));
    assert!(prompt.contains("打击"));
    assert!(prompt.contains("痛击"));
}

#[test]
fn shop_prompt_without_purge_omits_removal_section() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::ShopScreen),
        gold: Some(100),
        shop_cards: vec![],
        shop_relics: vec![],
        shop_potions: vec![],
        purge_available: false,
        purge_cost: None,
        master_cards: vec![],
        deck_names: vec![],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);

    assert!(prompt.contains("[mode: shop]"));
    assert!(!prompt.contains("=== 待购卡牌 ==="));
    assert!(!prompt.contains("=== 删牌服务 ==="));
    assert!(!prompt.contains("Remove a card"));
}

#[test]
fn shop_prompt_purge_not_available_omits_removal_but_shows_deck() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::ShopScreen),
        gold: Some(100),
        shop_cards: vec![],
        shop_relics: vec![],
        shop_potions: vec![],
        purge_available: false,
        purge_cost: None,
        master_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        deck_names: vec!["打击".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 卡组 ==="));
    assert!(!prompt.contains("=== 删牌服务 ==="));
}

#[test]
fn build_shop_items_price_none() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::ShopScreen),
        gold: Some(200),
        shop_cards: vec![CardInfo {
            name: "打击".into(),
            cost: 1,
            price: None,
            ..card("Strike_R", "打击", 1, "ATTACK")
        }],
        shop_relics: vec![RelicInfo {
            id: "Shovel".into(),
            name: "铲子".into(),
            description: "挖掘".into(),
            counter: None,
            price: None,
        }],
        shop_potions: vec![PotionInfo {
            id: None,
            slot: 0,
            name: "再生药水".into(),
            description: "获得 5 层 再生 。".into(),
            price: None,
            can_use: false,
            can_discard: false,
            requires_target: false,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("? gold"));
}

#[test]
fn build_shop_purge_cost_none() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::ShopScreen),
        gold: Some(200),
        purge_available: true,
        purge_cost: None,
        master_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        deck_names: vec!["打击".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("Remove a card for unknown gold"));
}

#[test]
fn build_shop_all_empty_no_section_header() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::ShopScreen),
        gold: Some(100),
        shop_cards: vec![],
        shop_relics: vec![],
        shop_potions: vec![],
        purge_available: false,
        purge_cost: None,
        master_cards: vec![],
        deck_names: vec![],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(!prompt.contains("=== 商店 ==="));
    assert!(prompt.contains("[mode: shop]"));
}
