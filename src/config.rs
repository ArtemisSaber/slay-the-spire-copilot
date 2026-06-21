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
}

impl Config {
    pub fn from_env() -> Self {
        #[cfg(not(test))]
        dotenvy::dotenv().ok();

        Self::from_lookup(|key| env::var(key).ok())
    }

    pub fn from_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let provider = lookup("LLM_PROVIDER").unwrap_or_else(|| "mock".to_string());

        let base_url = lookup("LLM_BASE_URL");
        let api_key = lookup("LLM_API_KEY");

        let fallback_model = lookup("LLM_MODEL").unwrap_or_else(|| "gpt-4o-mini".to_string());

        let model_fast = lookup("LLM_MODEL_FAST").unwrap_or_else(|| fallback_model.clone());
        let model_medium = lookup("LLM_MODEL_MEDIUM").unwrap_or_else(|| fallback_model.clone());
        let model_heavy = lookup("LLM_MODEL_HEAVY").unwrap_or_else(|| fallback_model.clone());

        let ceiling = lookup("LLM_MAX_TOKENS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(50000);

        let max_tokens_heavy = lookup("LLM_MAX_TOKENS_HEAVY")
            .and_then(|v| v.parse().ok())
            .unwrap_or(ceiling);
        let max_tokens_medium = lookup("LLM_MAX_TOKENS_MEDIUM")
            .and_then(|v| v.parse().ok())
            .unwrap_or(max_tokens_heavy.min(10000));
        let max_tokens_fast = lookup("LLM_MAX_TOKENS_FAST")
            .and_then(|v| v.parse().ok())
            .unwrap_or(max_tokens_heavy.min(3000));

        let temperature = lookup("LLM_TEMPERATURE")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.7);

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
