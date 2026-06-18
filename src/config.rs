use std::env;

pub struct Config {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "mock".to_string());

        let base_url = env::var("LLM_BASE_URL").ok();

        let api_key = env::var("LLM_API_KEY").ok();

        let model = env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        Config {
            provider,
            base_url,
            api_key,
            model,
        }
    }
}
