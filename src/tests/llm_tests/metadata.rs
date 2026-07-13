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
    assert!(!prompt.is_empty());
}
