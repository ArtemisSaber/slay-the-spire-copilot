use super::*;

#[test]
fn advice_scenario_as_str_all_variants() {
    assert_eq!(AdviceScenario::CardReward.as_str(), "card_reward");
    assert_eq!(AdviceScenario::BossCardReward.as_str(), "boss_card_reward");
    assert_eq!(AdviceScenario::BossRelic.as_str(), "boss_relic");
    assert_eq!(AdviceScenario::Rest.as_str(), "rest");
    assert_eq!(AdviceScenario::EventChoice.as_str(), "event_choice");
    assert_eq!(AdviceScenario::Shop.as_str(), "shop");
    assert_eq!(AdviceScenario::CombatEntry.as_str(), "combat_entry");
    assert_eq!(AdviceScenario::MapSuggestion.as_str(), "map_suggestion");
    assert_eq!(AdviceScenario::MapCrossroad.as_str(), "map_crossroad");
    assert_eq!(AdviceScenario::Generic.as_str(), "generic");
    assert_eq!(AdviceScenario::Postmortem.as_str(), "postmortem");
}

#[test]
fn effort_as_str_returns_correct_strings() {
    assert_eq!(Effort::Fast.as_str(), "fast");
    assert_eq!(Effort::Medium.as_str(), "medium");
    assert_eq!(Effort::Heavy.as_str(), "heavy");
}

#[test]
fn unified_system_prompt_contains_shop_mode() {
    let prompt = unified_system_prompt(test_locale());
    assert!(prompt.contains("[mode: shop]"));
}

#[test]
fn unified_system_prompt_contains_all_ten_modes() {
    let prompt = unified_system_prompt(test_locale());
    for mode in [
        "combat",
        "card_reward",
        "boss_card_reward",
        "rest",
        "boss_relic",
        "event_choice",
        "shop",
        "map_suggestion",
        "map_crossroad",
        "generic",
    ] {
        assert!(
            prompt.contains(&format!("[mode: {mode}]")),
            "missing mode: {mode}"
        );
    }
    assert!(!prompt.contains("[mode: postmortem]"));
}

#[test]
fn autoplay_action_system_prompt_contains_planner_header() {
    let prompt = autoplay_action_system_prompt(test_locale());
    assert!(prompt.contains("AUTO_PLAY_ACTION_PLANNER"));
    assert!(prompt.contains("scenario.combat.monsters[].index"));
    assert!(prompt.contains("descriptions"));
    assert!(!prompt.contains("localized_status_context"));
    assert!(!prompt.to_ascii_lowercase().contains("uuid"));
    assert!(!prompt.is_empty());
}

#[test]
fn autoplay_action_system_prompt_does_not_define_card_reward_policy() {
    let prompt = autoplay_action_system_prompt(test_locale());

    assert!(!prompt.contains("For scenario.kind card_reward"));
    assert!(!prompt.contains("Treat Skip as the baseline"));
    assert!(!prompt.contains("marginal_net_gain"));
}

#[test]
fn card_reward_candidate_selector_system_prompt_is_forced_pick_only() {
    let prompt = card_reward_candidate_system_prompt();

    assert!(prompt.contains("CARD_REWARD_CANDIDATE_SELECTOR_V1"));
    assert!(prompt.contains("Assume one offered card must be added"));
    assert!(prompt.contains("Do not evaluate Skip"));
    assert!(prompt.contains("starter cards"));
    assert!(prompt.contains("not a deck-size threshold"));
    assert!(!prompt.contains("20 cards"));
    assert!(!prompt.contains("30 cards"));
}

#[test]
fn card_reward_comparison_system_prompt_is_order_and_ref_neutral() {
    let prompt = card_reward_comparison_system_prompt();

    assert!(prompt.contains("CARD_REWARD_RESULTING_STATE_JUDGE_V1"));
    assert!(prompt.contains("Evaluate each resulting state independently"));
    assert!(prompt.contains("references and order are arbitrary"));
    assert!(prompt.contains("indifferent"));
    assert!(prompt.contains("uncertain"));
    assert!(!prompt.contains("Deck A"));
    assert!(!prompt.contains("Deck B"));
}

#[test]
fn unified_preambles_do_not_globalize_boss_heal() {
    for (language, global_heal_rule, boss_reward_heal_rule) in [
        ("en", "Full heal after Boss", "HP fully heals next act"),
        ("zh", "Boss 战后回满血", "Boss 战后下一幕会回满血"),
        ("ja", "Boss後は全回復", "ボス戦後、次のActでHPが全回復"),
        (
            "ko",
            "Boss 전투 후 완전 회복",
            "보스 전투 후 다음 액트에서 HP가 완전 회복",
        ),
    ] {
        let locale = Locale::load(language);
        assert!(
            !locale.unified_preamble.contains(global_heal_rule),
            "{language} preamble still exposes the boss-only heal rule globally"
        );
        assert!(
            locale
                .system_prompts
                .boss_card_reward
                .contains(boss_reward_heal_rule),
            "{language} boss card reward prompt lost its scoped heal rule"
        );
    }
}

#[test]
fn unified_preambles_do_not_impose_a_lean_deck_policy() {
    for (language, lean_deck_rule, thick_deck_rule) in [
        (
            "en",
            "Skip card picks to keep deck lean",
            "Thick decks lower key draw consistency",
        ),
        (
            "zh",
            "可跳过选牌，保持卡组精简",
            "卡组过厚会降低关键牌上手率",
        ),
        (
            "ja",
            "カード報酬はスキップ可能、デッキのスリム化が重要",
            "デッキが厚いとキーカードの引け率が下がる",
        ),
        (
            "ko",
            "카드 보상 건너뛰기 가능, 덱을 간결하게 유지",
            "두꺼운 덱은 핵심 카드 드로우 확률 저하",
        ),
    ] {
        let locale = Locale::load(language);
        assert!(
            !locale.unified_preamble.contains(lean_deck_rule),
            "{language} preamble still makes lean decks a global rule"
        );
        assert!(
            !locale.unified_preamble.contains(thick_deck_rule),
            "{language} preamble still makes deck thickness a global rule"
        );
    }
}
