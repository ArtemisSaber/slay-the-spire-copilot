use super::*;
use crate::locales::Locale;
use crate::state::{DangerFlags, DangerLevel, MonsterInfo, NormalizedState, RelicInfo};
use crate::test_utils::test_locale;

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
        let prompt = scenario.system_prompt(&test_locale());
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
        let prompt = scenario.system_prompt(&test_locale());
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
    let prompt = AdviceScenario::BossCardReward.system_prompt(&test_locale());
    assert!(prompt.contains("血量"));
    assert!(prompt.contains("回满血"));
    assert!(prompt.contains("下一幕"));
}

#[test]
fn normal_card_reward_prompt_supports_skip() {
    let prompt = AdviceScenario::CardReward.system_prompt(&test_locale());
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
    let prompt = AdviceScenario::CombatEntry.system_prompt(&test_locale());
    assert!(prompt.contains("策略助手"));
    assert!(prompt.contains("游戏事实"));
    assert!(prompt.contains("易伤"));
    assert!(prompt.contains("格挡在回合结束时清零"));
    assert!(prompt.contains("通用规则"));
}

#[test]
fn unified_system_prompt_contains_all_modes() {
    let prompt = AdviceScenario::CombatEntry.system_prompt(&test_locale());
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
    let prompt = AdviceScenario::CombatEntry.system_prompt(&test_locale());
    assert!(!prompt.contains("[mode: postmortem]"));
    assert!(!prompt.contains("复盘"));
}

#[test]
fn postmortem_uses_standalone_system_prompt() {
    let prompt = AdviceScenario::Postmortem.system_prompt(&test_locale());
    assert!(prompt.contains("复盘"));
    assert!(!prompt.contains("[mode: "));
}

#[test]
fn unified_system_prompt_modes_separated() {
    let prompt = AdviceScenario::CombatEntry.system_prompt(&test_locale());
    let sections: Vec<_> = prompt.matches("[mode:").collect();
    assert_eq!(sections.len(), 10);
    assert!(prompt.contains("\n---\n\n[mode:"));
}

#[test]
fn all_scenarios_return_same_unified_prompt() {
    let combat = AdviceScenario::CombatEntry.system_prompt(&test_locale());
    let card = AdviceScenario::CardReward.system_prompt(&test_locale());
    assert_eq!(combat, card);
}

fn test_state() -> NormalizedState {
    NormalizedState {
        screen_type: Some("NONE".into()),
        room_type: Some("MonsterRoom".into()),
        character: Some("IRONCLAD".into()),
        seed: None,
        ascension_level: None,
        floor: Some(1),
        current_hp: Some(68),
        max_hp: Some(75),
        gold: Some(99),
        energy: Some(3),
        block: Some(0),
        powers: vec![],
        hand: vec![],
        monsters: vec![],
        card_reward_choices: vec![],
        boss_relic_choices: vec![],
        event_id: None,
        event_name: None,
        event_body: None,
        event_choices: vec![],
        relics: vec![],
        potions: vec![],
        deck_names: vec![],
        incoming_damage: 0,
        rest_options: vec![],
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Safe,
        },
        skip_available: false,
        shop_cards: vec![],
        shop_relics: vec![],
        shop_potions: vec![],
        purge_available: false,
        purge_cost: None,
        hand_cards: vec![],
        draw_pile: vec![],
        discard_pile: vec![],
        exhaust_cards: vec![],
        master_cards: vec![],
        map_nodes: vec![],
        map_first_node_chosen: None,
        map_current_x: None,
        map_current_y: None,
        hand_select_max_cards: None,
        hand_select_can_pick_zero: false,
        hand_select_selected: vec![],
        current_action: None,
        card_in_play: None,
        grid_cards: vec![],
        grid_selected_cards: vec![],
        grid_for_upgrade: false,
        grid_for_transform: false,
        grid_for_purge: false,
        grid_num_cards: None,
        empty_potion_slots: 0,
        ..Default::default()
    }
}

fn monster() -> MonsterInfo {
    MonsterInfo {
        name: "大颚虫".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(40),
        max_hp: Some(40),
        block: Some(0),
        intent: Some("ATTACK".into()),
        damage: Some(12),
        hits: Some(1),
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    }
}

#[test]
fn scenario_resolver_detects_boss_card_reward_floors() {
    for floor in [16, 33, 50] {
        let state = NormalizedState {
            screen_type: Some("CARD_REWARD".into()),
            floor: Some(floor),
            ..test_state()
        };
        assert_eq!(
            AdviceScenario::from_state(&state),
            AdviceScenario::BossCardReward
        );
    }
}

#[test]
fn scenario_resolver_detects_ordinary_card_reward() {
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        floor: Some(14),
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::CardReward
    );
}

#[test]
fn scenario_resolver_detects_rest() {
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        ..test_state()
    };
    assert_eq!(AdviceScenario::from_state(&state), AdviceScenario::Rest);
}

#[test]
fn scenario_resolver_detects_boss_relic() {
    let state = NormalizedState {
        screen_type: Some("BOSS_REWARD".into()),
        boss_relic_choices: vec![RelicInfo {
            id: "Runic Dome".into(),
            name: "符文圆顶".into(),
            description: String::new(),
            counter: None,
            price: None,
        }],
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::BossRelic
    );
}

#[test]
fn scenario_resolver_detects_event_choice() {
    let state = NormalizedState {
        screen_type: Some("EVENT".into()),
        event_choices: vec!["获得遗物".into()],
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::EventChoice
    );
}

#[test]
fn scenario_resolver_detects_combat_entry() {
    let state = NormalizedState {
        monsters: vec![monster()],
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::CombatEntry
    );
}

#[test]
fn scenario_resolver_defaults_to_generic() {
    assert_eq!(
        AdviceScenario::from_state(&test_state()),
        AdviceScenario::Generic
    );
}

#[test]
fn scenario_resolver_detects_map_suggestion() {
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        map_first_node_chosen: Some(false),
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::MapSuggestion
    );
}

#[test]
fn scenario_resolver_detects_map_crossroad() {
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        map_first_node_chosen: Some(true),
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::MapCrossroad
    );
}

#[test]
fn log_includes_prompt_and_response() {
    let dir = tempfile::tempdir().unwrap();
    let prompt = "test-prompt-🦀🤣🦖";
    let response = "test-response-吃葡萄不吐葡萄皮";
    log_prompt_to(dir.path(), prompt, response);

    let log_path = dir.path().join("logs").join("prompts.log");
    let contents = std::fs::read_to_string(&log_path).unwrap();
    assert!(contents.contains(prompt));
    assert!(contents.contains(response));
    assert!(contents.contains("[system]"));
    assert!(contents.contains("[user]"));
    assert!(contents.contains("[assistant]"));
}

#[test]
fn effort_from_screen_type_card_reward_is_heavy() {
    assert!(matches!(
        Effort::from_screen_type("CARD_REWARD", false),
        Effort::Heavy
    ));
}

#[test]
fn effort_from_screen_type_map_is_heavy() {
    assert!(matches!(
        Effort::from_screen_type("MAP", false),
        Effort::Heavy
    ));
}

#[test]
fn effort_from_screen_type_none_is_fast() {
    assert!(matches!(
        Effort::from_screen_type("NONE", false),
        Effort::Fast
    ));
}

#[test]
fn effort_from_screen_type_hand_select_is_fast() {
    assert!(matches!(
        Effort::from_screen_type("HAND_SELECT", false),
        Effort::Fast
    ));
}

#[test]
fn effort_from_screen_type_grid_is_medium() {
    assert!(matches!(
        Effort::from_screen_type("GRID", false),
        Effort::Medium
    ));
}

#[test]
fn effort_from_screen_type_other_is_medium() {
    assert!(matches!(
        Effort::from_screen_type("REST", false),
        Effort::Medium
    ));
    assert!(matches!(
        Effort::from_screen_type("UNKNOWN", false),
        Effort::Medium
    ));
}

#[test]
fn effort_card_reward_fast_when_in_combat() {
    assert!(matches!(
        Effort::from_screen_type("CARD_REWARD", true),
        Effort::Fast
    ));
}

#[test]
fn effort_grid_fast_when_in_combat() {
    assert!(matches!(
        Effort::from_screen_type("GRID", true),
        Effort::Fast
    ));
}

#[test]
fn effort_unknown_fast_when_in_combat() {
    assert!(matches!(
        Effort::from_screen_type("SHOP_SCREEN", true),
        Effort::Fast
    ));
}

#[tokio::test]
async fn mock_provider_returns_structured_response() {
    let provider = LlmProvider::Mock;
    let result = provider
        .query_advice(
            "test prompt",
            Effort::Fast,
            AdviceScenario::Generic,
            &test_locale(),
        )
        .await;
    assert!(result.is_ok());
    let text = result.unwrap();
    assert!(!text.is_empty());
    assert!(text.contains("推荐："));
    assert!(text.contains("理由："));
    assert!(text.contains("风险："));
    assert!(text.contains("吐槽："));
}

#[tokio::test]
async fn mock_provider_returns_postmortem_report() {
    let provider = LlmProvider::Mock;
    let result = provider
        .query_postmortem("deterministic summary", &test_locale())
        .await;
    assert!(result.is_ok());
    let text = result.unwrap();
    assert!(text.contains("# 本局复盘"));
    assert!(text.contains("## 总览"));
    assert!(text.contains("## 下次改进"));
}

#[test]
fn chat_completion_body_can_disable_thinking() {
    let cfg = OpenAiConfig {
        model: "deepseek-v4-flash".into(),
        max_tokens: 300,
        disable_thinking: true,
    };

    let body = chat_completion_body(&cfg, "system", "user", 0.7);

    assert_eq!(body["thinking"]["type"], "disabled");
}

#[test]
fn chat_completion_body_omits_thinking_when_not_configured() {
    let cfg = OpenAiConfig {
        model: "gpt-5-nano".into(),
        max_tokens: 300,
        disable_thinking: false,
    };

    let body = chat_completion_body(&cfg, "system", "user", 0.7);

    assert!(body.get("thinking").is_none());
}

#[test]
fn anthropic_messages_body_uses_system_and_user_prompt() {
    let cfg = OpenAiConfig {
        model: "claude-sonnet-4-6".into(),
        max_tokens: 500,
        disable_thinking: false,
    };

    let body = anthropic_messages_body(&cfg, "system", "user");

    assert_eq!(body["model"], "claude-sonnet-4-6");
    assert_eq!(body["system"], "system");
    assert_eq!(body["messages"][0]["role"], "user");
    assert_eq!(body["messages"][0]["content"], "user");
    assert_eq!(body["max_tokens"], 500);
    assert!(body.get("temperature").is_none());
}

#[test]
fn anthropic_response_text_collects_text_blocks() {
    let json = serde_json::json!({
        "content": [
            {"type": "text", "text": "hello"},
            {"type": "thinking", "thinking": "..."},
            {"type": "text", "text": " world"}
        ]
    });

    let text = anthropic_response_text(&json).unwrap();

    assert_eq!(text, "hello world");
}

#[test]
fn chat_response_text_extracts_openai_compatible_content() {
    let json = serde_json::json!({
        "choices": [
            {"message": {"content": "hello"}}
        ]
    });

    let text = chat_response_text(&json).unwrap();

    assert_eq!(text, "hello");
}

#[test]
fn from_config_unknown_provider() {
    let config = crate::config::Config {
        provider: "unknown-provider".into(),
        base_url: None,
        api_key: None,
        model_fast: "m".into(),
        model_medium: "m".into(),
        model_heavy: "m".into(),
        max_tokens_fast: 100,
        max_tokens_medium: 500,
        max_tokens_heavy: 1000,
        temperature: 0.5,
        disable_fast_thinking: false,
        auto_play: false,
    };
    let result = LlmProvider::from_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unknown"));
}

#[test]
fn from_config_missing_base_url() {
    let config = crate::config::Config {
        provider: "openai-compatible".into(),
        base_url: None,
        api_key: None,
        model_fast: "m".into(),
        model_medium: "m".into(),
        model_heavy: "m".into(),
        max_tokens_fast: 100,
        max_tokens_medium: 500,
        max_tokens_heavy: 1000,
        temperature: 0.5,
        disable_fast_thinking: false,
        auto_play: false,
    };
    let result = LlmProvider::from_config(&config);
    assert!(result.is_err());
}

#[test]
fn from_config_pollinations_free_accepts_no_api_key() {
    let config = crate::config::Config {
        provider: "pollinations-free".into(),
        base_url: None,
        api_key: None,
        model_fast: "openai-fast".into(),
        model_medium: "openai-fast".into(),
        model_heavy: "openai-fast".into(),
        max_tokens_fast: 100,
        max_tokens_medium: 500,
        max_tokens_heavy: 1000,
        temperature: 0.5,
        disable_fast_thinking: false,
        auto_play: false,
    };
    let result = LlmProvider::from_config(&config);
    assert!(matches!(result, Ok(LlmProvider::PollinationsFree { .. })));
}

#[test]
fn from_config_anthropic_requires_api_key() {
    let config = crate::config::Config {
        provider: "anthropic".into(),
        base_url: Some("https://api.anthropic.com".into()),
        api_key: None,
        model_fast: "claude-haiku-4-5".into(),
        model_medium: "claude-sonnet-4-6".into(),
        model_heavy: "claude-sonnet-4-6".into(),
        max_tokens_fast: 100,
        max_tokens_medium: 500,
        max_tokens_heavy: 1000,
        temperature: 0.5,
        disable_fast_thinking: false,
        auto_play: false,
    };
    let result = LlmProvider::from_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("LLM_API_KEY"));
}

#[test]
fn from_config_anthropic_accepts_valid_config() {
    let config = crate::config::Config {
        provider: "anthropic".into(),
        base_url: None,
        api_key: Some("sk-ant-test".into()),
        model_fast: "claude-haiku-4-5".into(),
        model_medium: "claude-sonnet-4-6".into(),
        model_heavy: "claude-sonnet-4-6".into(),
        max_tokens_fast: 100,
        max_tokens_medium: 500,
        max_tokens_heavy: 1000,
        temperature: 0.5,
        disable_fast_thinking: false,
        auto_play: false,
    };
    let result = LlmProvider::from_config(&config);
    assert!(matches!(result, Ok(LlmProvider::Anthropic { .. })));
}

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
    let prompt = unified_system_prompt(&test_locale());
    assert!(prompt.contains("[mode: shop]"));
}

#[test]
fn unified_system_prompt_contains_all_ten_modes() {
    let prompt = unified_system_prompt(&test_locale());
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
    let prompt = autoplay_action_system_prompt(&test_locale());
    assert!(prompt.contains("AUTO_PLAY_ACTION_PLANNER"));
    assert!(!prompt.is_empty());
}

#[test]
fn mock_autoplay_fallback_test_marker_returns_invalid_action() {
    let prompt = r#"{"localized_status_context":"fallback_test_marker","available_actions":[{"kind":"play","action_id":"card:strike","label":"Strike"}],"rejected_attempts":["previous fail"]}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["action_id"], "event:99");
    assert_eq!(json["actions"][0]["label"], "Invalid");
}

#[test]
fn mock_autoplay_retry_test_marker_no_rejections_returns_invalid_action() {
    let prompt = r#"{"localized_status_context":"retry_test_marker","available_actions":[{"kind":"play","action_id":"card:strike","label":"Strike"}],"rejected_attempts":[]}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["action_id"], "event:99");
}

#[test]
fn mock_autoplay_prefers_card_reward_skip() {
    let prompt = r#"{"available_actions":[{"kind":"choose","action_id":"card_reward:skip","label":"Skip"},{"kind":"choose","action_id":"card_reward:0","label":"Card 0"}],"localized_status_context":""}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["action_id"], "card_reward:skip");
    assert_eq!(json["actions"][0]["kind"], "choose");
}

#[test]
fn mock_autoplay_prefers_play_action() {
    let prompt = r#"{"available_actions":[{"kind":"end","action_id":"end:turn","label":"End Turn"},{"kind":"play","action_id":"card:strike","label":"Strike"}],"localized_status_context":""}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["kind"], "play");
    assert_eq!(json["actions"][0]["action_id"], "card:strike");
}

#[test]
fn mock_autoplay_first_action_fallback() {
    let prompt = r#"{"available_actions":[{"kind":"end","action_id":"end:turn","label":"End Turn"},{"kind":"proceed","action_id":"proceed","label":"Proceed"}],"localized_status_context":""}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["kind"], "end");
    assert_eq!(json["actions"][0]["action_id"], "end:turn");
}

#[test]
fn mock_autoplay_empty_actions_returns_empty() {
    let prompt = r#"{"available_actions":[],"localized_status_context":""}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert!(json["actions"].as_array().unwrap().is_empty());
}

#[test]
fn mock_autoplay_target_required_sets_target_index() {
    let prompt = r#"{"available_actions":[{"kind":"play","action_id":"card:strike","label":"Strike","target_required":true}],"localized_status_context":""}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["target_index"], 0);
}

#[test]
fn mock_autoplay_no_target_required_sets_null_target_index() {
    let prompt = r#"{"available_actions":[{"kind":"play","action_id":"card:strike","label":"Strike","target_required":false}],"localized_status_context":""}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["target_index"], serde_json::Value::Null);
}

#[test]
fn from_config_accepts_mock_provider() {
    let config = crate::config::Config {
        provider: "mock".into(),
        base_url: None,
        api_key: None,
        model_fast: "m".into(),
        model_medium: "m".into(),
        model_heavy: "m".into(),
        max_tokens_fast: 100,
        max_tokens_medium: 500,
        max_tokens_heavy: 1000,
        temperature: 0.5,
        disable_fast_thinking: false,
        auto_play: false,
    };
    let result = LlmProvider::from_config(&config);
    assert!(matches!(result, Ok(LlmProvider::Mock)));
}

#[tokio::test]
async fn mock_provider_query_autoplay_action_returns_json() {
    let provider = LlmProvider::Mock;
    let prompt = r#"{"available_actions":[{"kind":"play","action_id":"card:strike","label":"Strike"}],"localized_status_context":""}"#;
    let result = provider
        .query_autoplay_action(prompt, Effort::Fast, &test_locale())
        .await;
    assert!(result.is_ok());
    let text = result.unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["actions"][0]["kind"], "play");
}

#[tokio::test]
async fn mock_provider_query_advice_error_path() {
    let provider = LlmProvider::Mock;
    let result = provider
        .query_advice(
            "TRIGGER_LLM_ERROR",
            Effort::Fast,
            AdviceScenario::Generic,
            &test_locale(),
        )
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("mock error"));
}

#[test]
fn log_prompt_into_dir_writes_formatted_entry() {
    let dir = tempfile::tempdir().unwrap();
    log_prompt_into_dir(dir.path(), "sys-content", "usr-content", "ast-content");
    let log_path = dir.path().join("logs").join("prompts.log");
    let contents = std::fs::read_to_string(&log_path).unwrap();
    assert!(contents.contains("[system]\nsys-content"));
    assert!(contents.contains("[user]\nusr-content"));
    assert!(contents.contains("[assistant]\nast-content"));
    assert!(contents.contains("\n---\n"));
}

#[test]
fn scenario_resolver_detects_shop() {
    let state = NormalizedState {
        screen_type: Some("SHOP_SCREEN".into()),
        ..test_state()
    };
    assert_eq!(AdviceScenario::from_state(&state), AdviceScenario::Shop);
}

#[test]
fn sanitize_err_body_redacts_api_key() {
    let key = "sk-secret-key-12345";
    let body = r#"{"error":"Authorization header: Bearer sk-secret-key-12345"}"#;
    let result = sanitize_err_body(body, Some(key));
    assert!(!result.contains(key), "api key must be redacted");
    assert!(result.contains("<REDACTED>"));
}

#[test]
fn sanitize_err_body_handles_empty_key() {
    let body = r#"{"error":"bad request"}"#;
    let result = sanitize_err_body(body, Some(""));
    assert!(result.contains("bad request"));
    assert!(!result.contains("<REDACTED>"));
}

#[test]
fn sanitize_err_body_handles_none_key() {
    let body = r#"{"error":"rate limited"}"#;
    let result = sanitize_err_body(body, None);
    assert!(result.contains("rate limited"));
}

#[test]
fn sanitize_err_body_truncates_long_body() {
    let long_body = "x".repeat(1000);
    let result = sanitize_err_body(&long_body, None);
    assert!(result.len() < long_body.len());
    assert!(result.ends_with("...(truncated)"));
}

#[test]
fn sanitize_err_body_preserves_short_body() {
    let body = r#"{"error":"not found"}"#;
    let result = sanitize_err_body(body, None);
    assert_eq!(result, body);
}

#[test]
fn llm_provider_debug_does_not_leak_api_key() {
    let provider = LlmProvider::OpenAiCompatible {
        base_url: "https://api.example.com".into(),
        api_key: "sk-super-secret-key".into(),
        temperature: 0.7,
        client: reqwest::Client::new(),
        fast: OpenAiConfig {
            model: "gpt-4o-mini".into(),
            max_tokens: 300,
            disable_thinking: false,
        },
        medium: OpenAiConfig {
            model: "gpt-4o".into(),
            max_tokens: 10000,
            disable_thinking: false,
        },
        heavy: OpenAiConfig {
            model: "gpt-4o".into(),
            max_tokens: 50000,
            disable_thinking: false,
        },
    };
    let debug_output = format!("{provider:?}");
    assert!(
        !debug_output.contains("sk-super-secret-key"),
        "Debug output must not contain the API key"
    );
    assert!(debug_output.contains("<REDACTED>"));
    assert!(debug_output.contains("gpt-4o-mini"));
}

#[test]
fn llm_provider_debug_redacts_anthropic_api_key() {
    let provider = LlmProvider::Anthropic {
        base_url: "https://api.anthropic.com".into(),
        api_key: "sk-ant-secret-key".into(),
        client: reqwest::Client::new(),
        fast: OpenAiConfig {
            model: "claude-sonnet".into(),
            max_tokens: 300,
            disable_thinking: false,
        },
        medium: OpenAiConfig {
            model: "claude-sonnet".into(),
            max_tokens: 10000,
            disable_thinking: false,
        },
        heavy: OpenAiConfig {
            model: "claude-sonnet".into(),
            max_tokens: 50000,
            disable_thinking: false,
        },
    };
    let debug_output = format!("{provider:?}");
    assert!(
        !debug_output.contains("sk-ant-secret-key"),
        "Debug output must not contain the Anthropic API key"
    );
    assert!(debug_output.contains("<REDACTED>"));
}

#[test]
fn llm_provider_debug_mock_has_no_secrets() {
    let provider = LlmProvider::Mock;
    let debug_output = format!("{provider:?}");
    assert!(debug_output.contains("Mock"));
}
