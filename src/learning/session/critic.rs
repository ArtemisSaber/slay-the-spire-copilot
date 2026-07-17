use super::LearningSession;

mod contract;
mod draft;
mod evidence;
mod mode;
mod prompt;

pub(crate) use draft::{CriticDraft, CriticDraftRejection};
use evidence::{supplied_cases, trim_oldest_case};
use mode::{CriticMode, context_for_run};
use prompt::{critic_appendix, fact_reviewer_appendix, truncate_utf8};
use serde_json::Value;

const MAX_PROMPT_BYTES: usize = 40_000;
const MAX_REPORT_BYTES: usize = 8_000;
const ENVELOPE_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticIngest {
    pub report_markdown: String,
    pub response_valid: bool,
    pub accepted_lessons: usize,
    pub rejected_lessons: usize,
    pub snapshot_id: String,
}

impl LearningSession {
    pub(crate) fn critic_is_report_only(&self, run_id: &str) -> bool {
        context_for_run(self, run_id)
            .is_some_and(|context| matches!(context.mode, CriticMode::ReportOnly))
    }

    pub(crate) fn resolve_lesson_fallback(&mut self, run_id: &str) -> anyhow::Result<()> {
        let parent_id = context_for_run(self, run_id)
            .and_then(|context| context.mode.parent().map(|parent| parent.lesson_id.clone()));
        if let Some(parent_id) = parent_id {
            self.resolve_regeneration_without_replacement(&parent_id)?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn build_critic_prompt(&self, base_report_prompt: &str, run_id: &str) -> Option<String> {
        self.build_critic_prompt_with_feedback(base_report_prompt, run_id, None, &[])
    }

    pub(crate) fn build_critic_prompt_with_feedback(
        &self,
        base_report_prompt: &str,
        run_id: &str,
        review_rejection: Option<&Value>,
        retry_feedback: &[String],
    ) -> Option<String> {
        let context = context_for_run(self, run_id)?;
        if matches!(context.mode, CriticMode::ReportOnly) {
            return None;
        }
        let mut cases = supplied_cases(&self.snapshot.cases, run_id, context.mode);
        if cases.is_empty() {
            return None;
        }
        let base = truncate_utf8(base_report_prompt, MAX_REPORT_BYTES);
        let original_count = cases.len();
        loop {
            let appendix = critic_appendix(&context, &cases, review_rejection, retry_feedback);
            let prompt = format!("{base}\n\nLEARNING_CRITIC_ENVELOPE_V3\n{appendix}");
            if prompt.len() <= MAX_PROMPT_BYTES {
                if cases.len() < original_count {
                    tracing::info!(
                        "trimmed learning critic evidence from {} to {} cases for run {}",
                        original_count,
                        cases.len(),
                        run_id,
                    );
                }
                return Some(prompt);
            }
            if !trim_oldest_case(&mut cases, run_id) {
                tracing::warn!(
                    "learning critic evidence for run {} cannot fit within {} bytes",
                    run_id,
                    MAX_PROMPT_BYTES,
                );
                return None;
            }
        }
    }

    pub(crate) fn build_fact_review_prompt(
        &self,
        deterministic_report: &str,
        run_id: &str,
        draft: &CriticDraft,
        retry_feedback: &[String],
    ) -> Option<String> {
        let context = context_for_run(self, run_id)?;
        let candidate = draft.candidate_json();
        let required_claims = draft.review_claims();
        let mut cases = supplied_cases(&self.snapshot.cases, run_id, context.mode);
        let report = truncate_utf8(deterministic_report, MAX_REPORT_BYTES);
        loop {
            let appendix = fact_reviewer_appendix(
                &context,
                &cases,
                &candidate,
                &required_claims,
                report,
                retry_feedback,
            );
            let prompt = format!("LESSON_FACT_REVIEWER_V3\n{appendix}");
            if prompt.len() <= MAX_PROMPT_BYTES {
                return Some(prompt);
            }
            if !trim_oldest_case(&mut cases, run_id) {
                tracing::warn!(
                    "lesson fact review evidence for run {} cannot fit within {} bytes",
                    run_id,
                    MAX_PROMPT_BYTES,
                );
                return None;
            }
        }
    }

    pub(crate) fn deliberation_ingest(
        &self,
        report: String,
        response_valid: bool,
        rejected: usize,
    ) -> CriticIngest {
        CriticIngest {
            report_markdown: report,
            response_valid,
            accepted_lessons: 0,
            rejected_lessons: rejected,
            snapshot_id: self.snapshot.snapshot_id.clone(),
        }
    }
}
