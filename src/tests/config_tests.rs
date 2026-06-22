use super::Config;
use std::collections::HashMap;

#[test]
fn defaults_when_no_env() {
    let c = Config::from_map(&HashMap::new());
    assert_eq!(c.provider, "mock");
    assert!(c.base_url.is_none());
    assert!(c.api_key.is_none());
    assert_eq!(c.model_fast, "gpt-4o-mini");
    assert_eq!(c.model_medium, "gpt-4o-mini");
    assert_eq!(c.model_heavy, "gpt-4o-mini");
    assert_eq!(c.max_tokens_heavy, 50000);
    assert_eq!(c.max_tokens_medium, 10000);
    assert_eq!(c.max_tokens_fast, 300);
    assert_eq!(c.temperature, 0.7);
    assert!(!c.disable_fast_thinking);
}

#[test]
fn provider_base_url_and_api_key() {
    let c = Config::from_map(&HashMap::from([
        ("LLM_PROVIDER", "openai-compatible"),
        ("LLM_BASE_URL", "https://api.example.com"),
        ("LLM_API_KEY", "sk-test"),
    ]));
    assert_eq!(c.provider, "openai-compatible");
    assert_eq!(c.base_url.as_deref(), Some("https://api.example.com"));
    assert_eq!(c.api_key.as_deref(), Some("sk-test"));
}

#[test]
fn model_chain_fallback() {
    let c = Config::from_map(&HashMap::from([("LLM_MODEL", "claude-3")]));
    assert_eq!(c.model_fast, "claude-3");
    assert_eq!(c.model_medium, "claude-3");
    assert_eq!(c.model_heavy, "claude-3");
}

#[test]
fn blank_model_overrides_fall_back_to_base_model() {
    let c = Config::from_map(&HashMap::from([
        ("LLM_MODEL", "fallback-model"),
        ("LLM_MODEL_FAST", ""),
        ("LLM_MODEL_MEDIUM", ""),
        ("LLM_MODEL_HEAVY", ""),
    ]));
    assert_eq!(c.model_fast, "fallback-model");
    assert_eq!(c.model_medium, "fallback-model");
    assert_eq!(c.model_heavy, "fallback-model");
}

#[test]
fn per_tier_model_overrides() {
    let c = Config::from_map(&HashMap::from([
        ("LLM_MODEL", "fallback-model"),
        ("LLM_MODEL_FAST", "gpt-4o-mini"),
        ("LLM_MODEL_HEAVY", "deepseek-v3"),
    ]));
    assert_eq!(c.model_fast, "gpt-4o-mini");
    assert_eq!(c.model_medium, "fallback-model");
    assert_eq!(c.model_heavy, "deepseek-v3");
}

#[test]
fn max_tokens_ceiling_and_clamping() {
    let c = Config::from_map(&HashMap::from([
        ("LLM_MAX_TOKENS", "2000"),
        ("LLM_MAX_TOKENS_HEAVY", "8000"),
    ]));
    assert_eq!(c.max_tokens_heavy, 8000);
    assert_eq!(c.max_tokens_medium, 8000);
    assert_eq!(c.max_tokens_fast, 300);
}

#[test]
fn individual_token_overrides() {
    let c = Config::from_map(&HashMap::from([
        ("LLM_MAX_TOKENS", "50000"),
        ("LLM_MAX_TOKENS_FAST", "500"),
        ("LLM_MAX_TOKENS_MEDIUM", "2000"),
        ("LLM_MAX_TOKENS_HEAVY", "16000"),
    ]));
    assert_eq!(c.max_tokens_fast, 500);
    assert_eq!(c.max_tokens_medium, 2000);
    assert_eq!(c.max_tokens_heavy, 16000);
}

#[test]
fn temperature_override() {
    let c = Config::from_map(&HashMap::from([("LLM_TEMPERATURE", "0.3")]));
    assert_eq!(c.temperature, 0.3);
}

#[test]
fn deepseek_base_url_disables_fast_thinking_by_default() {
    let c = Config::from_map(&HashMap::from([(
        "LLM_BASE_URL",
        "https://api.deepseek.com",
    )]));
    assert!(c.disable_fast_thinking);
}

#[test]
fn blank_api_fields_are_treated_as_absent() {
    let c = Config::from_map(&HashMap::from([
        ("LLM_PROVIDER", ""),
        ("LLM_BASE_URL", ""),
        ("LLM_API_KEY", ""),
    ]));
    assert_eq!(c.provider, "mock");
    assert!(c.base_url.is_none());
    assert!(c.api_key.is_none());
}

#[test]
fn fast_thinking_override_wins_over_deepseek_default() {
    let c = Config::from_map(&HashMap::from([
        ("LLM_BASE_URL", "https://api.deepseek.com"),
        ("LLM_DISABLE_FAST_THINKING", "false"),
    ]));
    assert!(!c.disable_fast_thinking);
}
