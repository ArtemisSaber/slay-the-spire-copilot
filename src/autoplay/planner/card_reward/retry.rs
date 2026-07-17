use anyhow::{Context, anyhow};
use serde_json::{Value, json};

pub(super) const MAX_STAGE_ATTEMPTS: usize = super::super::MAX_LLM_ATTEMPTS;

pub(super) fn prompt_for_attempt(
    base_prompt: &str,
    attempt: usize,
    previous_error: Option<&anyhow::Error>,
) -> anyhow::Result<String> {
    let Some(previous_error) = previous_error else {
        return Ok(base_prompt.to_string());
    };
    let mut payload: Value = serde_json::from_str(base_prompt)
        .context("failed to parse the card reward base prompt for retry")?;
    payload["retry_context"] = json!({
        "attempt": attempt,
        "max_attempts": MAX_STAGE_ATTEMPTS,
        "previous_attempt_error": format!("{previous_error:#}"),
        "instruction": "Correct the previous response. Return exactly one strict JSON object matching the supplied schema, with every required field and no surrounding prose."
    });
    serde_json::to_string_pretty(&payload).context("failed to build card reward retry prompt")
}

pub(super) fn exhausted(stage: &str, last_error: Option<anyhow::Error>) -> anyhow::Error {
    let context = format!("{stage} failed after {MAX_STAGE_ATTEMPTS} attempts");
    match last_error {
        Some(error) => error.context(context),
        None => anyhow!(context),
    }
}
