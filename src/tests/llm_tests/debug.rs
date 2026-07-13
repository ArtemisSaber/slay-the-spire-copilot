use super::*;

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
