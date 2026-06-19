use super::Config;

fn clear_config_env() {
    for var in &[
        "LLM_PROVIDER",
        "LLM_BASE_URL",
        "LLM_API_KEY",
        "LLM_MODEL",
        "LLM_MODEL_FAST",
        "LLM_MODEL_MEDIUM",
        "LLM_MODEL_HEAVY",
        "LLM_MAX_TOKENS",
        "LLM_MAX_TOKENS_FAST",
        "LLM_MAX_TOKENS_MEDIUM",
        "LLM_MAX_TOKENS_HEAVY",
        "LLM_TEMPERATURE",
    ] {
        unsafe { std::env::remove_var(var) };
    }
}

fn set_env(key: &str, val: &str) {
    unsafe { std::env::set_var(key, val) };
}

#[test]
fn config_from_env() {
    // Sub-test: all defaults
    clear_config_env();
    let c = Config::from_env();
    assert_eq!(c.provider, "mock");
    assert!(c.base_url.is_none());
    assert!(c.api_key.is_none());
    assert_eq!(c.model_fast, "gpt-4o-mini");
    assert_eq!(c.model_medium, "gpt-4o-mini");
    assert_eq!(c.model_heavy, "gpt-4o-mini");
    assert_eq!(c.max_tokens_heavy, 50000);
    assert_eq!(c.max_tokens_medium, 10000);
    assert_eq!(c.max_tokens_fast, 3000);
    assert_eq!(c.temperature, 0.7);

    // Sub-test: provider + base url + api key
    clear_config_env();
    set_env("LLM_PROVIDER", "openai-compatible");
    set_env("LLM_BASE_URL", "https://api.example.com");
    set_env("LLM_API_KEY", "sk-test");
    let c = Config::from_env();
    assert_eq!(c.provider, "openai-compatible");
    assert_eq!(c.base_url.as_deref(), Some("https://api.example.com"));
    assert_eq!(c.api_key.as_deref(), Some("sk-test"));

    // Sub-test: model chain fallback
    clear_config_env();
    set_env("LLM_MODEL", "claude-3");
    let c = Config::from_env();
    assert_eq!(c.model_fast, "claude-3");
    assert_eq!(c.model_medium, "claude-3");
    assert_eq!(c.model_heavy, "claude-3");

    // Sub-test: per-tier model overrides
    clear_config_env();
    set_env("LLM_MODEL", "fallback-model");
    set_env("LLM_MODEL_FAST", "gpt-4o-mini");
    set_env("LLM_MODEL_HEAVY", "deepseek-v3");
    let c = Config::from_env();
    assert_eq!(c.model_fast, "gpt-4o-mini");
    assert_eq!(c.model_medium, "fallback-model");
    assert_eq!(c.model_heavy, "deepseek-v3");

    // Sub-test: max tokens ceiling and clamping
    clear_config_env();
    set_env("LLM_MAX_TOKENS", "2000");
    set_env("LLM_MAX_TOKENS_HEAVY", "8000");
    let c = Config::from_env();
    assert_eq!(c.max_tokens_heavy, 8000);
    assert_eq!(c.max_tokens_medium, 8000);
    assert_eq!(c.max_tokens_fast, 3000);

    // Sub-test: individual token overrides
    clear_config_env();
    set_env("LLM_MAX_TOKENS", "50000");
    set_env("LLM_MAX_TOKENS_FAST", "500");
    set_env("LLM_MAX_TOKENS_MEDIUM", "2000");
    set_env("LLM_MAX_TOKENS_HEAVY", "16000");
    let c = Config::from_env();
    assert_eq!(c.max_tokens_fast, 500);
    assert_eq!(c.max_tokens_medium, 2000);
    assert_eq!(c.max_tokens_heavy, 16000);

    // Sub-test: temperature override
    clear_config_env();
    set_env("LLM_TEMPERATURE", "0.3");
    let c = Config::from_env();
    assert_eq!(c.temperature, 0.3);
}
