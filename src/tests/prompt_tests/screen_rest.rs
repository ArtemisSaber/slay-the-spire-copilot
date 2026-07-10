use super::*;

#[test]
fn rest_prompt_has_translated_options() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        character: Some("IRONCLAD".into()),
        current_hp: Some(25),
        max_hp: Some(75),
        floor: Some(5),
        rest_options: vec!["rest".into(), "smith".into()],
        danger: DangerFlags {
            hp_critical: true,
            level: DangerLevel::Danger,
            ..test_state().danger
        },
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("休息"));
    assert!(prompt.contains("锻造"));
}

#[test]
fn rest_prompt_marks_campfire_decision_task() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        rest_options: vec!["rest".into(), "smith".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.starts_with("[mode: rest]"));
    assert!(prompt.contains("休息"));
    assert!(prompt.contains("锻造"));
}

#[test]
fn rest_prompt_requires_smith_upgrade_target() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        rest_options: vec!["smith".into()],
        master_cards: vec![
            card("Bash", "痛击", 2, "ATTACK"),
            card("Armaments", "武装", 1, "SKILL"),
        ],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 可锻造升级目标 ==="));
    assert!(prompt.contains("痛击"));
    assert!(prompt.contains("武装"));
}

#[test]
fn rest_prompt_advises_rest_when_hp_low() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        current_hp: Some(15),
        max_hp: Some(75),
        rest_options: vec!["rest".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("血量极低，强烈建议休息。"));
}

#[test]
fn rest_prompt_suggests_smith_when_hp_high() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        current_hp: Some(60),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("血量健康，可考虑锻造或挖遗物。"));
}

#[test]
fn rest_shows_upgradeable_cards() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        master_cards: vec![
            card("Strike_R", "Strike", 1, "ATTACK"),
            CardInfo {
                upgraded: true,
                ..card("Defend_R", "Defend", 1, "SKILL")
            },
        ],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 可锻造升级目标 ==="));
}

#[test]
fn rest_no_upgradeable_when_all_upgraded() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        master_cards: vec![CardInfo {
            upgraded: true,
            ..card("Strike_R", "Strike", 1, "ATTACK")
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(!prompt.contains("=== 可升级卡牌 ==="));
}

#[test]
fn build_rest_all_option_types() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec![
            "rest".into(),
            "smith".into(),
            "toke".into(),
            "dig".into(),
            "lift".into(),
            "recall".into(),
            "girya".into(),
        ],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("回忆"));
    assert!(prompt.contains("挖遗物"));
    assert!(prompt.contains("举重"));
    assert!(prompt.contains("回忆钥匙"));
    assert!(prompt.contains("深蹲"));
}

#[test]
fn build_rest_mid_hp_no_advice() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["rest".into(), "smith".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(!prompt.contains("强烈建议休息"));
    assert!(!prompt.contains("可考虑锻造或挖遗物"));
}

#[test]
fn build_rest_filters_curse_status_from_upgrade() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        master_cards: vec![
            card("Strike_R", "打击", 1, "ATTACK"),
            CardInfo {
                card_type: "CURSE".into(),
                ..card("Shame", "羞耻", -2, "CURSE")
            },
            CardInfo {
                card_type: "STATUS".into(),
                ..card("Slimed", "黏液", 1, "STATUS")
            },
        ],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 可锻造升级目标 ==="));
    assert!(prompt.contains("打击"));
    assert!(!prompt.contains("羞耻"));
    assert!(!prompt.contains("黏液"));
}

#[test]
fn build_rest_dedup_same_card_id() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        master_cards: vec![
            card("Strike_R", "打击", 1, "ATTACK"),
            card("Strike_R", "打击", 1, "ATTACK"),
            card("Defend_R", "防御", 1, "SKILL"),
        ],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert_eq!(prompt.matches("打击(1费/攻击)").count(), 1);
    assert!(prompt.contains("防御"));
}

#[test]
fn build_rest_unknown_option_fallback() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["custom_action".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("custom_action"));
}
