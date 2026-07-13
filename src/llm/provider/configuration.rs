use super::{LlmProvider, OpenAiConfig};
use crate::config::Config;
use anyhow::Context;

pub(super) fn provider_from_config(config: &Config) -> anyhow::Result<LlmProvider> {
    match config.provider.as_str() {
        "mock" => Ok(LlmProvider::Mock),
        "openai-compatible" => openai_compatible_provider(config),
        "pollinations-free" => pollinations_free_provider(config),
        "anthropic" => anthropic_provider(config),
        other => anyhow::bail!("unknown LLM_PROVIDER: {other}"),
    }
}

fn openai_compatible_provider(config: &Config) -> anyhow::Result<LlmProvider> {
    let raw_base_url = config
        .base_url
        .as_ref()
        .context("LLM_BASE_URL is required for openai-compatible provider")?;
    let base_url = super::super::requests::validate_base_url(raw_base_url)?;
    let api_key = config
        .api_key
        .as_ref()
        .context("LLM_API_KEY is required for openai-compatible provider")?
        .clone();
    let (fast, medium, heavy) = tiered_configs(config, config.disable_fast_thinking);
    Ok(LlmProvider::OpenAiCompatible {
        base_url,
        api_key,
        temperature: config.temperature,
        client: reqwest::Client::new(),
        fast,
        medium,
        heavy,
    })
}

fn pollinations_free_provider(config: &Config) -> anyhow::Result<LlmProvider> {
    let base_url = super::super::requests::validate_base_url(
        config
            .base_url
            .as_deref()
            .unwrap_or("https://text.pollinations.ai/openai"),
    )?;
    let (fast, medium, heavy) = tiered_configs(config, false);
    Ok(LlmProvider::PollinationsFree {
        base_url,
        temperature: config.temperature,
        client: reqwest::Client::new(),
        fast,
        medium,
        heavy,
    })
}

fn anthropic_provider(config: &Config) -> anyhow::Result<LlmProvider> {
    let base_url = super::super::requests::validate_base_url(
        config
            .base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com"),
    )?;
    let api_key = config
        .api_key
        .as_ref()
        .context("LLM_API_KEY is required for anthropic provider")?
        .clone();
    let (fast, medium, heavy) = tiered_configs(config, false);
    Ok(LlmProvider::Anthropic {
        base_url,
        api_key,
        client: reqwest::Client::new(),
        fast,
        medium,
        heavy,
    })
}

fn tiered_configs(
    config: &Config,
    disable_fast_thinking: bool,
) -> (OpenAiConfig, OpenAiConfig, OpenAiConfig) {
    (
        OpenAiConfig {
            model: config.model_fast.clone(),
            max_tokens: config.max_tokens_fast,
            disable_thinking: disable_fast_thinking,
        },
        OpenAiConfig {
            model: config.model_medium.clone(),
            max_tokens: config.max_tokens_medium,
            disable_thinking: false,
        },
        OpenAiConfig {
            model: config.model_heavy.clone(),
            max_tokens: config.max_tokens_heavy,
            disable_thinking: false,
        },
    )
}
