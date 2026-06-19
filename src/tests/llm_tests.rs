use super::*;

#[test]
fn system_prompt_is_defined() {
    assert!(!SYSTEM_PROMPT.is_empty());
    assert!(SYSTEM_PROMPT.contains("杀戮尖塔"));
    assert!(SYSTEM_PROMPT.contains("推荐："));
    assert!(SYSTEM_PROMPT.contains("理由："));
    assert!(SYSTEM_PROMPT.contains("风险："));
    assert!(SYSTEM_PROMPT.contains("吐槽："));
}

#[test]
fn log_includes_prompt_and_response() {
    let prompt = "test-prompt-🦀🤣🦖";
    let response = "test-response-吃葡萄不吐葡萄皮";
    log_prompt(prompt, response);

    let log_path = crate::logging::project_root().join("logs").join("prompts.log");
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
    let result = provider.query("test prompt", Effort::Fast).await;
    assert!(result.is_ok());
    let text = result.unwrap();
    assert!(!text.is_empty());
    assert!(text.contains("推荐："));
    assert!(text.contains("理由："));
    assert!(text.contains("风险："));
    assert!(text.contains("吐槽："));
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
