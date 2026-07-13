use std::fs;
use std::path::Path;

pub(crate) fn log_prompt_with_system(system_prompt: &str, user_prompt: &str, response: &str) {
    log_prompt_into_dir(
        &crate::logging::project_root(),
        system_prompt,
        user_prompt,
        response,
    );
}

pub(crate) fn prompt_logging_disabled(env_value: Option<String>) -> bool {
    match env_value {
        Some(value) => matches!(
            value.to_ascii_lowercase().as_str(),
            "0" | "false" | "no" | "off"
        ),
        None => false,
    }
}

pub(crate) fn log_prompt_into_dir(
    base: &Path,
    system_prompt: &str,
    user_prompt: &str,
    response: &str,
) {
    if prompt_logging_disabled(std::env::var("LLM_LOG_PROMPTS").ok()) {
        return;
    }
    let log_dir = base.join("logs");
    if let Err(error) = fs::create_dir_all(&log_dir) {
        tracing::warn!("failed to create logs dir: {error}");
    }
    let path = log_dir.join("prompts.log");
    let entry = format!(
        "[system]\n{system_prompt}\n\n[user]\n{user_prompt}\n\n[assistant]\n{response}\n---\n",
    );

    if let Err(error) = crate::logging::append_locked(&path, entry.as_bytes()) {
        tracing::warn!("failed to write prompts log {}: {error}", path.display());
    }
}

#[cfg(test)]
pub(crate) fn log_prompt_to(base: &Path, user_prompt: &str, response: &str) {
    let locale = crate::test_utils::test_locale();
    log_prompt_into_dir(base, &locale.system_prompts.generic, user_prompt, response);
}
