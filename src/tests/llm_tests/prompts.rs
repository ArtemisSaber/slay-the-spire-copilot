use super::*;

#[test]
fn scenario_system_prompts_are_defined() {
    for scenario in [
        AdviceScenario::CardReward,
        AdviceScenario::BossCardReward,
        AdviceScenario::BossRelic,
        AdviceScenario::Rest,
        AdviceScenario::EventChoice,
        AdviceScenario::CombatEntry,
        AdviceScenario::Generic,
        AdviceScenario::MapSuggestion,
        AdviceScenario::MapCrossroad,
    ] {
        let prompt = scenario.system_prompt(test_locale());
        assert!(!prompt.is_empty());
        assert!(prompt.contains("杀戮尖塔"));
        assert!(prompt.contains("推荐："));
        assert!(prompt.contains("理由："));
        assert!(prompt.contains("风险："));
    }
}

#[test]
fn scenario_system_prompts_include_few_shot_examples() {
    for scenario in [
        AdviceScenario::CardReward,
        AdviceScenario::BossCardReward,
        AdviceScenario::BossRelic,
        AdviceScenario::Rest,
        AdviceScenario::EventChoice,
        AdviceScenario::CombatEntry,
        AdviceScenario::MapSuggestion,
        AdviceScenario::MapCrossroad,
        AdviceScenario::Generic,
        AdviceScenario::Postmortem,
    ] {
        let prompt = scenario.system_prompt(test_locale());
        assert!(
            prompt.contains("示例：") || prompt.contains("例：") || prompt.contains("Example:"),
            "{scenario:?} should include a few-shot example"
        );
    }
}

#[test]
fn all_locales_define_few_shot_examples() {
    for lang in ["en", "zh", "ja", "ko"] {
        let locale = Locale::load(lang);
        for scenario in [
            AdviceScenario::CardReward,
            AdviceScenario::BossCardReward,
            AdviceScenario::BossRelic,
            AdviceScenario::Rest,
            AdviceScenario::EventChoice,
            AdviceScenario::CombatEntry,
            AdviceScenario::MapSuggestion,
            AdviceScenario::MapCrossroad,
            AdviceScenario::Generic,
            AdviceScenario::Postmortem,
        ] {
            assert!(
                !scenario.few_shot_example(&locale).trim().is_empty(),
                "{lang} {scenario:?} should define a few-shot example"
            );
        }
    }
}

#[test]
fn map_system_prompt_warns_against_candidate_number_only() {
    let locale = Locale::load("en");
    let prompt = AdviceScenario::MapSuggestion.system_prompt(&locale);
    assert!(prompt.contains("Recommendation label"));
    assert!(prompt.contains("do not answer with Candidate number alone"));
    assert!(prompt.contains("Root 3 (3rd from left)"));
}

#[test]
fn boss_reward_system_prompt_ignores_current_hp() {
    let prompt = AdviceScenario::BossCardReward.system_prompt(test_locale());
    assert!(prompt.contains("血量"));
    assert!(prompt.contains("回满血"));
    assert!(prompt.contains("下一幕"));
}

#[test]
fn normal_card_reward_prompt_supports_skip() {
    let prompt = AdviceScenario::CardReward.system_prompt(test_locale());
    assert!(prompt.contains("跳过"));
}

#[test]
fn postmortem_system_prompt_is_defined() {
    let locale = test_locale();
    let prompt = &locale.system_prompts.postmortem;
    assert!(!prompt.is_empty());
    assert!(prompt.contains("复盘"));
    assert!(prompt.contains("Markdown"));
    assert!(prompt.contains("日志"));
}

#[test]
fn unified_system_prompt_contains_preamble() {
    let prompt = AdviceScenario::CombatEntry.system_prompt(test_locale());
    assert!(prompt.contains("策略助手"));
    assert!(prompt.contains("游戏事实"));
    assert!(prompt.contains("易伤"));
    assert!(prompt.contains("格挡在回合结束时清零"));
    assert!(prompt.contains("通用规则"));
}

#[test]
fn unified_system_prompt_contains_all_modes() {
    let prompt = AdviceScenario::CombatEntry.system_prompt(test_locale());
    for mode in [
        "combat",
        "card_reward",
        "boss_card_reward",
        "rest",
        "boss_relic",
        "event_choice",
        "map_suggestion",
        "map_crossroad",
        "generic",
    ] {
        assert!(
            prompt.contains(&format!("[mode: {mode}]")),
            "missing mode: {mode}"
        );
    }
}

#[test]
fn unified_system_prompt_excludes_postmortem() {
    let prompt = AdviceScenario::CombatEntry.system_prompt(test_locale());
    assert!(!prompt.contains("[mode: postmortem]"));
    assert!(!prompt.contains("复盘"));
}

#[test]
fn postmortem_uses_standalone_system_prompt() {
    let prompt = AdviceScenario::Postmortem.system_prompt(test_locale());
    assert!(prompt.contains("复盘"));
    assert!(!prompt.contains("[mode: "));
}

#[test]
fn unified_system_prompt_modes_separated() {
    let prompt = AdviceScenario::CombatEntry.system_prompt(test_locale());
    let sections: Vec<_> = prompt.matches("[mode:").collect();
    assert_eq!(sections.len(), 10);
    assert!(prompt.contains("\n---\n\n[mode:"));
}

#[test]
fn all_scenarios_return_same_unified_prompt() {
    let combat = AdviceScenario::CombatEntry.system_prompt(test_locale());
    let card = AdviceScenario::CardReward.system_prompt(test_locale());
    assert_eq!(combat, card);
}
