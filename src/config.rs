use std::env;

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

        let provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "mock".to_string());

        let base_url = env::var("LLM_BASE_URL").ok();
        let api_key = env::var("LLM_API_KEY").ok();

        let fallback_model = env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        let model_fast = env::var("LLM_MODEL_FAST").unwrap_or_else(|_| fallback_model.clone());
        let model_medium = env::var("LLM_MODEL_MEDIUM").unwrap_or_else(|_| fallback_model.clone());
        let model_heavy = env::var("LLM_MODEL_HEAVY").unwrap_or_else(|_| fallback_model.clone());

        let ceiling = env::var("LLM_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50000);

        let max_tokens_heavy = env::var("LLM_MAX_TOKENS_HEAVY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(ceiling);
        let max_tokens_medium = env::var("LLM_MAX_TOKENS_MEDIUM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(max_tokens_heavy.min(10000));
        let max_tokens_fast = env::var("LLM_MAX_TOKENS_FAST")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(max_tokens_heavy.min(3000));

        let temperature = env::var("LLM_TEMPERATURE")
            .ok()
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
}

#[cfg(test)]
#[path = "tests/config_tests.rs"]
mod tests;
