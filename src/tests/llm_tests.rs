use super::*;
use crate::state::{DangerFlags, DangerLevel, MonsterInfo, NormalizedState, RelicInfo};

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
    ] {
        let prompt = scenario.system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("杀戮尖塔"));
        assert!(prompt.contains("推荐："));
        assert!(prompt.contains("理由："));
        assert!(prompt.contains("风险："));
    }
}

#[test]
fn boss_reward_system_prompt_ignores_current_hp() {
    let prompt = AdviceScenario::BossCardReward.system_prompt();
    assert!(prompt.contains("当前血量"));
    assert!(prompt.contains("不要"));
    assert!(prompt.contains("回满血"));
    assert!(prompt.contains("16"));
    assert!(prompt.contains("33"));
    assert!(prompt.contains("50"));
}

#[test]
fn normal_card_reward_prompt_supports_skip() {
    let prompt = AdviceScenario::CardReward.system_prompt();
    assert!(prompt.contains("跳过"));
}

#[test]
fn postmortem_system_prompt_is_defined() {
    assert!(!POSTMORTEM_SYSTEM_PROMPT.is_empty());
    assert!(POSTMORTEM_SYSTEM_PROMPT.contains("复盘"));
    assert!(POSTMORTEM_SYSTEM_PROMPT.contains("Markdown"));
    assert!(POSTMORTEM_SYSTEM_PROMPT.contains("不要编造"));
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
        hand_cards: vec![],
        draw_pile: vec![],
        discard_pile: vec![],
        exhaust_cards: vec![],
        master_cards: vec![],
    }
}

fn monster() -> MonsterInfo {
    MonsterInfo {
        name: "大颚虫".into(),
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
            name: "符文圆顶".into(),
            description: String::new(),
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
        Effort::from_screen_type("CARD_REWARD"),
        Effort::Heavy
    ));
}

#[test]
fn effort_from_screen_type_none_is_fast() {
    assert!(matches!(Effort::from_screen_type("NONE"), Effort::Fast));
}

#[test]
fn effort_from_screen_type_other_is_medium() {
    assert!(matches!(Effort::from_screen_type("REST"), Effort::Medium));
    assert!(matches!(
        Effort::from_screen_type("UNKNOWN"),
        Effort::Medium
    ));
}

#[tokio::test]
async fn mock_provider_returns_structured_response() {
    let provider = LlmProvider::Mock;
    let result = provider
        .query_advice("test prompt", Effort::Fast, AdviceScenario::Generic)
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
    let result = provider.query_postmortem("deterministic summary").await;
    assert!(result.is_ok());
    let text = result.unwrap();
    assert!(text.contains("# 本局复盘"));
    assert!(text.contains("## 总览"));
    assert!(text.contains("## 下次改进"));
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
    };
    let result = LlmProvider::from_config(&config);
    assert!(result.is_err());
}
