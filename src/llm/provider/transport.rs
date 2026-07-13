use super::OpenAiConfig;
use crate::llm::Effort;
use anyhow::Context;
use std::time::Instant;

pub(super) struct ProviderRequest<'a> {
    pub(super) base_url: &'a str,
    pub(super) api_key: Option<&'a str>,
    pub(super) temperature: f64,
    pub(super) client: &'a reqwest::Client,
    pub(super) fast: &'a OpenAiConfig,
    pub(super) medium: &'a OpenAiConfig,
    pub(super) heavy: &'a OpenAiConfig,
    pub(super) system_prompt: &'a str,
    pub(super) prompt: &'a str,
    pub(super) effort: Effort,
}

pub(super) async fn query_openai_compatible(
    request: ProviderRequest<'_>,
) -> anyhow::Result<String> {
    let config = selected_config(&request);
    let url = format!("{}/chat/completions", request.base_url);
    let started = Instant::now();
    tracing::info!(
        "LLM request effort={} model={} prompt_chars={} system_chars={} max_tokens={} disable_thinking={}",
        request.effort.as_str(),
        config.model,
        request.prompt.len(),
        request.system_prompt.len(),
        config.max_tokens,
        config.disable_thinking,
    );

    let body = crate::llm::requests::chat_completion_body(
        config,
        request.system_prompt,
        request.prompt,
        request.temperature,
    );
    let api_key = request.api_key.unwrap_or_default();
    let response = request
        .client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("failed to send LLM request")?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        let error_body = crate::llm::requests::sanitize_err_body(&error_body, request.api_key);
        anyhow::bail!("LLM API error {status}: {error_body}");
    }

    let json: serde_json::Value = response
        .json()
        .await
        .context("failed to parse LLM response")?;
    let content = crate::llm::requests::chat_response_text(&json)?;
    tracing::info!(
        "LLM response effort={} model={} duration_ms={} response_chars={}",
        request.effort.as_str(),
        config.model,
        started.elapsed().as_millis(),
        content.len(),
    );
    Ok(content)
}

pub(super) async fn query_pollinations_free(
    request: ProviderRequest<'_>,
) -> anyhow::Result<String> {
    let config = selected_config(&request);
    let started = Instant::now();
    tracing::info!(
        "Pollinations free request effort={} model={} prompt_chars={} system_chars={} max_tokens={}",
        request.effort.as_str(),
        config.model,
        request.prompt.len(),
        request.system_prompt.len(),
        config.max_tokens,
    );

    let body = crate::llm::requests::chat_completion_body(
        config,
        request.system_prompt,
        request.prompt,
        request.temperature,
    );
    let response = request
        .client
        .post(request.base_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("failed to send Pollinations free request")?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        let error_body = crate::llm::requests::sanitize_err_body(&error_body, None);
        anyhow::bail!("Pollinations free API error {status}: {error_body}");
    }

    let json: serde_json::Value = response
        .json()
        .await
        .context("failed to parse Pollinations free response")?;
    let content = crate::llm::requests::chat_response_text(&json)?;
    tracing::info!(
        "Pollinations free response effort={} model={} duration_ms={} response_chars={}",
        request.effort.as_str(),
        config.model,
        started.elapsed().as_millis(),
        content.len(),
    );
    Ok(content)
}

pub(super) async fn query_anthropic(request: ProviderRequest<'_>) -> anyhow::Result<String> {
    let config = selected_config(&request);
    let url = format!("{}/v1/messages", request.base_url);
    let started = Instant::now();
    tracing::info!(
        "Anthropic request effort={} model={} prompt_chars={} system_chars={} max_tokens={}",
        request.effort.as_str(),
        config.model,
        request.prompt.len(),
        request.system_prompt.len(),
        config.max_tokens,
    );

    let body = crate::llm::requests::anthropic_messages_body(
        config,
        request.system_prompt,
        request.prompt,
    );
    let api_key = request.api_key.unwrap_or_default();
    let response = request
        .client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("failed to send Anthropic request")?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        let error_body = crate::llm::requests::sanitize_err_body(&error_body, request.api_key);
        anyhow::bail!("Anthropic API error {status}: {error_body}");
    }

    let json: serde_json::Value = response
        .json()
        .await
        .context("failed to parse Anthropic response")?;
    let content = crate::llm::requests::anthropic_response_text(&json)?;
    tracing::info!(
        "Anthropic response effort={} model={} duration_ms={} response_chars={}",
        request.effort.as_str(),
        config.model,
        started.elapsed().as_millis(),
        content.len(),
    );
    Ok(content)
}

fn selected_config<'a>(request: &ProviderRequest<'a>) -> &'a OpenAiConfig {
    match request.effort {
        Effort::Fast => request.fast,
        Effort::Medium => request.medium,
        Effort::Heavy => request.heavy,
    }
}
