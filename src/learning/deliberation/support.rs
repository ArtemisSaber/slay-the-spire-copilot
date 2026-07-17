use crate::locales::Locale;

const MAX_FEEDBACK_CHARS: usize = 1_024;

pub(super) fn error_feedback(context: &str, error: &anyhow::Error) -> String {
    format!("{context}: {error:#}")
        .chars()
        .take(MAX_FEEDBACK_CHARS)
        .collect()
}

pub(super) fn deterministic_report<'a>(base_prompt: &'a str, locale: &Locale) -> &'a str {
    let Some((_, review_input)) = base_prompt.split_once(&locale.postmortem.machine_summary) else {
        return base_prompt;
    };
    review_input
        .trim_start()
        .split_once("\n\n")
        .map_or(review_input.trim(), |(_, report)| report.trim())
}
