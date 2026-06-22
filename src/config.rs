use std::env;

#[cfg(test)]
use std::collections::HashMap;

pub struct Config {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_fast: String,
    pub model_medium: String,
    pub model_heavy: String,
    pub max_tokens_fast: u32,
    pub max_tokens_medium: u32,
    pub max_tokens_heavy: u32,
    pub temperature: f64,
    pub disable_fast_thinking: bool,
}

impl Config {
    pub fn from_env() -> Self {
        #[cfg(not(test))]
        dotenvy::dotenv().ok();

        Self::from_lookup(|key| env::var(key).ok())
    }

    pub fn from_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let mut lookup_non_empty = |key: &str| {
            lookup(key)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        };

        let provider = lookup_non_empty("LLM_PROVIDER").unwrap_or_else(|| "mock".to_string());

        let base_url = lookup_non_empty("LLM_BASE_URL");
        let api_key = lookup_non_empty("LLM_API_KEY");

        let fallback_model =
            lookup_non_empty("LLM_MODEL").unwrap_or_else(|| "gpt-4o-mini".to_string());

        let model_fast =
            lookup_non_empty("LLM_MODEL_FAST").unwrap_or_else(|| fallback_model.clone());
        let model_medium =
            lookup_non_empty("LLM_MODEL_MEDIUM").unwrap_or_else(|| fallback_model.clone());
        let model_heavy =
            lookup_non_empty("LLM_MODEL_HEAVY").unwrap_or_else(|| fallback_model.clone());

        let ceiling = lookup_non_empty("LLM_MAX_TOKENS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(50000);

        let max_tokens_heavy = lookup_non_empty("LLM_MAX_TOKENS_HEAVY")
            .and_then(|v| v.parse().ok())
            .unwrap_or(ceiling);
        let max_tokens_medium = lookup_non_empty("LLM_MAX_TOKENS_MEDIUM")
            .and_then(|v| v.parse().ok())
            .unwrap_or(max_tokens_heavy.min(10000));
        let max_tokens_fast = lookup_non_empty("LLM_MAX_TOKENS_FAST")
            .and_then(|v| v.parse().ok())
            .unwrap_or(max_tokens_heavy.min(300));

        let temperature = lookup_non_empty("LLM_TEMPERATURE")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.7);

        let disable_fast_thinking = lookup_non_empty("LLM_DISABLE_FAST_THINKING")
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or_else(|| {
                base_url
                    .as_deref()
                    .is_some_and(|url| url.contains("api.deepseek.com"))
            });

        Config {
            provider,
            base_url,
            api_key,
            model_fast,
            model_medium,
            model_heavy,
            max_tokens_fast,
            max_tokens_medium,
            max_tokens_heavy,
            temperature,
            disable_fast_thinking,
        }
    }

    #[cfg(test)]
    pub fn from_map(vars: &HashMap<&str, &str>) -> Self {
        let lookup = |key: &str| vars.get(key).map(|v| v.to_string());
        Self::from_lookup(lookup)
    }
}

#[cfg(test)]
#[path = "tests/config_tests.rs"]
mod tests;
