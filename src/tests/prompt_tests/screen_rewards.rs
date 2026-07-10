use super::*;

#[test]
fn card_reward_shows_card_choices() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::CardReward),
        character: Some("IRONCLAD".into()),
        floor: Some(3),
        current_hp: Some(62),
        max_hp: Some(75),
        card_reward_choices: vec![
            card("Uppercut", "上勾拳", 2, "ATTACK"),
            card("Anger", "愤怒", 1, "ATTACK"),
        ],
        master_cards: vec![card("Strike_R", "打击", 1, "ATTACK"); 4],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("A. 上勾拳"));
    assert!(prompt.contains("B. 愤怒"));
    assert!(prompt.contains("2费"));
    assert!(prompt.contains("1费"));
}

#[test]
fn card_reward_prompt_marks_pick_or_skip_task() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::CardReward),
        floor: Some(14),
        card_reward_choices: vec![card("Uppercut", "上勾拳", 2, "ATTACK")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.starts_with("[mode: card_reward]"));
    assert!(prompt.contains("上勾拳"));
}

#[test]
fn boss_card_reward_prompt_includes_full_heal_note() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::CardReward),
        floor: Some(16),
        current_hp: Some(3),
        max_hp: Some(75),
        card_reward_choices: vec![card("Demon Form", "Demon Form", 3, "POWER")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(
        prompt.contains(
            "注意：这是 Boss 战后的选牌。下一幕开始会回满血，不要把当前血量当成选牌依据。"
        )
    );
    assert!(prompt.contains("下一幕"));
    assert!(prompt.starts_with("[mode: boss_card_reward]"));
}

#[test]
fn ordinary_card_reward_prompt_omits_full_heal_note() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::CardReward),
        floor: Some(14),
        current_hp: Some(3),
        max_hp: Some(75),
        card_reward_choices: vec![card("Uppercut", "上勾拳", 2, "ATTACK")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(
        !prompt.contains(
            "注意：这是 Boss 战后的选牌。下一幕开始会回满血，不要把当前血量当成选牌依据。"
        )
    );
}

#[test]
fn card_reward_shows_skip_when_available() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::CardReward),
        card_reward_choices: vec![card("Uppercut", "Uppercut", 2, "ATTACK")],
        master_cards: vec![card("Strike_R", "Strike", 1, "ATTACK")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("跳过. 都不选"));
}

#[test]
fn card_reward_no_skip_when_unavailable() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::CardReward),
        card_reward_choices: vec![card("Uppercut", "Uppercut", 2, "ATTACK")],
        master_cards: vec![card("Strike_R", "Strike", 1, "ATTACK")],
        skip_available: false,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(!prompt.contains("跳过. 都不选"));
}

#[test]
fn boss_relic_prompt_lists_choices() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::BossReward),
        boss_relic_choices: vec![
            RelicInfo {
                id: "Snecko Eye".into(),
                name: "蛇眼".into(),
                description: "".into(),
                counter: None,
                price: None,
            },
            RelicInfo {
                id: "Runic Dome".into(),
                name: "符文圆顶".into(),
                description: "".into(),
                counter: None,
                price: None,
            },
            RelicInfo {
                id: "Cursed Key".into(),
                name: "诅咒钥匙".into(),
                description: "".into(),
                counter: None,
                price: None,
            },
        ],
        master_cards: vec![card("Bash", "Bash", 2, "ATTACK")],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== Boss 遗物 ==="));
    assert!(prompt.contains("A. 蛇眼"));
    assert!(prompt.contains("B. 符文圆顶"));
    assert!(prompt.contains("C. 诅咒钥匙"));
    assert!(prompt.starts_with("[mode: boss_relic]"));
}

#[test]
fn build_prompt_routes_card_reward() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::CardReward),
        card_reward_choices: vec![card("Strike_R", "Strike", 1, "ATTACK")],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 选牌 ==="));
    assert!(!prompt.contains("=== 手牌"));
}

#[test]
fn build_prompt_routes_boss_relic() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::BossReward),
        boss_relic_choices: vec![RelicInfo {
            id: "Snecko Eye".into(),
            name: "蛇眼".into(),
            description: String::new(),
            counter: None,
            price: None,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("Boss 遗物"));
}

#[test]
fn build_boss_relic_floor_17_hp_note() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::BossReward),
        floor: Some(17),
        boss_relic_choices: vec![RelicInfo {
            id: "Snecko Eye".into(),
            name: "蛇眼".into(),
            description: "".into(),
            counter: None,
            price: None,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("下一幕开始会回满血"));
}

#[test]
fn build_boss_relic_floor_34_hp_note() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::BossReward),
        floor: Some(34),
        boss_relic_choices: vec![RelicInfo {
            id: "Snecko Eye".into(),
            name: "蛇眼".into(),
            description: "".into(),
            counter: None,
            price: None,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("下一幕开始会回满血"));
}
