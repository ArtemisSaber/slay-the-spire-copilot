use std::collections::HashMap;

use super::types::{
    DEFAULT_MAX_TOKENS, DEFAULT_MAX_TOKENS_FAST, DEFAULT_MODEL, DEFAULT_OPENAI_BASE_URL,
    DEFAULT_TEMPERATURE, EnvAssignment, PLACEHOLDER_API_KEY, ProviderSetup,
};

pub(crate) fn provider_assignments(
    setup: ProviderSetup,
    existing: &HashMap<String, String>,
) -> Vec<EnvAssignment> {
    vec![
        assignment("LLM_PROVIDER", setup.provider),
        assignment("LLM_BASE_URL", setup.base_url),
        assignment("LLM_API_KEY", setup.api_key),
        assignment("LLM_MODEL", setup.heavy.clone()),
        assignment("LLM_MODEL_FAST", setup.fast),
        assignment("LLM_MODEL_MEDIUM", setup.medium),
        assignment("LLM_MODEL_HEAVY", setup.heavy),
        assignment(
            "LLM_MAX_TOKENS",
            existing_or_default(existing, "LLM_MAX_TOKENS", DEFAULT_MAX_TOKENS),
        ),
        assignment(
            "LLM_MAX_TOKENS_FAST",
            existing_or_default(existing, "LLM_MAX_TOKENS_FAST", DEFAULT_MAX_TOKENS_FAST),
        ),
        assignment(
            "LLM_MAX_TOKENS_MEDIUM",
            existing_or_default(existing, "LLM_MAX_TOKENS_MEDIUM", ""),
        ),
        assignment(
            "LLM_MAX_TOKENS_HEAVY",
            existing_or_default(existing, "LLM_MAX_TOKENS_HEAVY", ""),
        ),
        assignment(
            "LLM_TEMPERATURE",
            existing_or_default(existing, "LLM_TEMPERATURE", DEFAULT_TEMPERATURE),
        ),
        assignment(
            "LLM_DISABLE_FAST_THINKING",
            existing_or_default(
                existing,
                "LLM_DISABLE_FAST_THINKING",
                setup.disable_fast_thinking_default.unwrap_or(""),
            ),
        ),
    ]
}

pub(crate) fn mock_assignments(existing: &HashMap<String, String>) -> Vec<EnvAssignment> {
    vec![
        assignment("LLM_PROVIDER", "mock"),
        assignment(
            "LLM_BASE_URL",
            existing_or_default(existing, "LLM_BASE_URL", DEFAULT_OPENAI_BASE_URL),
        ),
        assignment(
            "LLM_API_KEY",
            existing_or_default(existing, "LLM_API_KEY", PLACEHOLDER_API_KEY),
        ),
        assignment(
            "LLM_MODEL",
            existing_or_default(existing, "LLM_MODEL", DEFAULT_MODEL),
        ),
        assignment(
            "LLM_MODEL_FAST",
            existing_or_default(existing, "LLM_MODEL_FAST", DEFAULT_MODEL),
        ),
        assignment(
            "LLM_MODEL_MEDIUM",
            existing_or_default(existing, "LLM_MODEL_MEDIUM", ""),
        ),
        assignment(
            "LLM_MODEL_HEAVY",
            existing_or_default(existing, "LLM_MODEL_HEAVY", ""),
        ),
        assignment(
            "LLM_MAX_TOKENS",
            existing_or_default(existing, "LLM_MAX_TOKENS", DEFAULT_MAX_TOKENS),
        ),
        assignment(
            "LLM_MAX_TOKENS_FAST",
            existing_or_default(existing, "LLM_MAX_TOKENS_FAST", DEFAULT_MAX_TOKENS_FAST),
        ),
        assignment(
            "LLM_MAX_TOKENS_MEDIUM",
            existing_or_default(existing, "LLM_MAX_TOKENS_MEDIUM", ""),
        ),
        assignment(
            "LLM_MAX_TOKENS_HEAVY",
            existing_or_default(existing, "LLM_MAX_TOKENS_HEAVY", ""),
        ),
        assignment(
            "LLM_TEMPERATURE",
            existing_or_default(existing, "LLM_TEMPERATURE", DEFAULT_TEMPERATURE),
        ),
        assignment(
            "LLM_DISABLE_FAST_THINKING",
            existing_or_default(existing, "LLM_DISABLE_FAST_THINKING", ""),
        ),
    ]
}

pub(crate) fn existing_or_default(
    existing: &HashMap<String, String>,
    key: &str,
    default_value: &str,
) -> String {
    existing
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| default_value.to_string())
}

pub(crate) fn assignment(key: &'static str, value: impl Into<String>) -> EnvAssignment {
    EnvAssignment {
        key,
        value: value.into(),
    }
}
