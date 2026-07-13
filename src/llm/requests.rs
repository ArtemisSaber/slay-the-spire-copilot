use super::provider::OpenAiConfig;
use anyhow::Context;

pub(crate) fn sanitize_err_body(body: &str, api_key: Option<&str>) -> String {
    let redacted = match api_key {
        Some(key) if !key.is_empty() => body.replace(key, "<REDACTED>"),
        _ => body.to_string(),
    };
    let truncated: String = redacted.chars().take(500).collect();
    if redacted.len() > truncated.len() {
        format!("{truncated}...(truncated)")
    } else {
        truncated
    }
}

pub(crate) fn validate_base_url(raw: &str) -> anyhow::Result<String> {
    let trimmed = raw.trim_end_matches('/');
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        Ok(trimmed.to_string())
    } else if let Some(scheme) = lower.split("://").next() {
        if scheme.contains(':') || scheme.is_empty() {
            anyhow::bail!("LLM_BASE_URL must be a full http(s) URL, got '{raw}'");
        }
        anyhow::bail!("LLM_BASE_URL must use http or https scheme, got '{scheme}://'");
    } else {
        anyhow::bail!("LLM_BASE_URL must be a full http(s) URL, got '{raw}'");
    }
}

pub(crate) fn chat_completion_body(
    config: &OpenAiConfig,
    system_prompt: &str,
    prompt: &str,
    temperature: f64,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "model": config.model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "max_tokens": config.max_tokens,
        "temperature": temperature
    });
    if config.disable_thinking {
        body["thinking"] = serde_json::json!({ "type": "disabled" });
    }
    body
}

pub(crate) fn anthropic_messages_body(
    config: &OpenAiConfig,
    system_prompt: &str,
    prompt: &str,
) -> serde_json::Value {
    serde_json::json!({
        "model": config.model,
        "system": system_prompt,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "max_tokens": config.max_tokens
    })
}

pub(crate) fn anthropic_response_text(json: &serde_json::Value) -> anyhow::Result<String> {
    let content = json["content"]
        .as_array()
        .context("missing content in Anthropic response")?;
    let text = content
        .iter()
        .filter_map(|block| {
            (block["type"].as_str() == Some("text"))
                .then(|| block["text"].as_str())
                .flatten()
        })
        .collect::<Vec<_>>()
        .join("");
    if text.is_empty() {
        anyhow::bail!("missing text content in Anthropic response");
    }
    Ok(text)
}

pub(crate) fn chat_response_text(json: &serde_json::Value) -> anyhow::Result<String> {
    json["choices"][0]["message"]["content"]
        .as_str()
        .context("missing content in LLM response")
        .map(str::to_string)
}
