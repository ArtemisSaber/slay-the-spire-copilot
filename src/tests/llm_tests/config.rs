use super::*;

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
        auto_play_auto_start: false,
        memory: Default::default(),
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
        auto_play_auto_start: false,
        memory: Default::default(),
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
        auto_play_auto_start: false,
        memory: Default::default(),
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
        auto_play_auto_start: false,
        memory: Default::default(),
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
        auto_play_auto_start: false,
        memory: Default::default(),
    };
    let result = LlmProvider::from_config(&config);
    assert!(matches!(result, Ok(LlmProvider::Anthropic { .. })));
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
        auto_play_auto_start: false,
        memory: Default::default(),
    };
    let result = LlmProvider::from_config(&config);
    assert!(matches!(result, Ok(LlmProvider::Mock)));
}
