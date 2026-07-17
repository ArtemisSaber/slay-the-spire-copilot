use crate::learning::session::LearningSession;
use crate::llm::LlmProvider;
use crate::locales::Locale;

use super::{DeliberationOutcome, DeliberationResult};

const MAX_ATTEMPTS: usize = 3;
const MAX_REPORT_BYTES: usize = 8_000;

pub(super) struct ReportOnlyResult {
    pub(super) markdown: String,
    pub(super) api_calls: usize,
    pub(super) response_valid: bool,
}

pub(super) async fn deliberate(
    learning: &LearningSession,
    provider: &LlmProvider,
    locale: &Locale,
    prompt: &str,
) -> DeliberationResult {
    let report = generate(provider, locale, prompt).await;
    let outcome = if report.response_valid {
        DeliberationOutcome::ReportOnly
    } else {
        DeliberationOutcome::Failed
    };
    DeliberationResult {
        ingest: learning.deliberation_ingest(report.markdown, report.response_valid, 0),
        api_calls: report.api_calls,
        outcome,
    }
}

async fn generate(provider: &LlmProvider, locale: &Locale, prompt: &str) -> ReportOnlyResult {
    let mut latest = String::new();
    for api_calls in 1..=MAX_ATTEMPTS {
        match provider.query_postmortem(prompt, locale).await {
            Ok(report) if valid_report(&report) => {
                return ReportOnlyResult {
                    markdown: report,
                    api_calls,
                    response_valid: true,
                };
            }
            Ok(report) => latest = report,
            Err(error) => latest = format!("postmortem query failed: {error:#}"),
        }
    }
    ReportOnlyResult {
        markdown: latest,
        api_calls: MAX_ATTEMPTS,
        response_valid: false,
    }
}

fn valid_report(report: &str) -> bool {
    !report.trim().is_empty()
        && report.len() <= MAX_REPORT_BYTES
        && !report
            .chars()
            .any(|character| character.is_control() && character != '\n' && character != '\t')
}
