use std::collections::HashSet;

use super::LearningSession;
use crate::learning::audit::select_audit_cases;
use crate::learning::bundle::embedded_bundle;
use crate::learning::lesson::{Lesson, LessonEvent, LessonEventKind, LessonStatus};

mod contract;
mod prompt;

use contract::{
    CandidateLesson, CriticEnvelope, analysis_outcome, candidate_matches_run_outcome,
    primary_case_id, valid_run_analysis,
};
use prompt::{critic_appendix, critic_case_summary, truncate_utf8, valid_report};

const MAX_PROMPT_BYTES: usize = 20_000;
const MAX_REPORT_BYTES: usize = 8_000;
const MAX_AUDIT_CASES: usize = 10;
const MAX_PROPOSALS: usize = 5;
const MIN_AUTOMATIC_CONFIDENCE: u16 = 600;
const ENVELOPE_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticIngest {
    pub report_markdown: String,
    pub response_valid: bool,
    pub accepted_lessons: usize,
    pub rejected_lessons: usize,
    pub snapshot_id: String,
}

impl LearningSession {
    pub fn build_critic_prompt(&self, base_report_prompt: &str, run_id: &str) -> Option<String> {
        let audit = select_audit_cases(&self.snapshot.cases, run_id, MAX_AUDIT_CASES);
        let outcome = analysis_outcome(&audit)?;
        let primary_case_id = primary_case_id(&audit, outcome)?.to_string();
        let base = truncate_utf8(base_report_prompt, MAX_REPORT_BYTES);
        let mut kept = audit;
        loop {
            let cases: Vec<_> = kept.iter().map(critic_case_summary).collect();
            let appendix = critic_appendix(&cases, outcome, &primary_case_id);
            let prompt = format!("{base}\n\nLEARNING_CRITIC_ENVELOPE_V2\n{appendix}");
            if prompt.len() <= MAX_PROMPT_BYTES {
                return Some(prompt);
            }
            if kept.len() == 1 {
                return None;
            }
            kept.pop();
        }
    }

    pub fn ingest_critic_response(
        &mut self,
        response: &str,
        run_id: &str,
    ) -> anyhow::Result<CriticIngest> {
        let Ok(envelope) = serde_json::from_str::<CriticEnvelope>(response.trim()) else {
            return Ok(self.empty_ingest(response));
        };
        let total = envelope.lesson_proposals.len().min(MAX_PROPOSALS);
        let audit = select_audit_cases(&self.snapshot.cases, run_id, MAX_AUDIT_CASES);
        let Some(outcome) = analysis_outcome(&audit) else {
            return Ok(self.invalid_ingest(envelope.report_markdown, total));
        };
        let Some(expected_primary) = primary_case_id(&audit, outcome) else {
            return Ok(self.invalid_ingest(envelope.report_markdown, total));
        };
        if envelope.schema_version != ENVELOPE_VERSION
            || !valid_report(&envelope.report_markdown, MAX_REPORT_BYTES)
            || !valid_run_analysis(&envelope.run_analysis, &audit, outcome, expected_primary)
        {
            return Ok(self.invalid_ingest(envelope.report_markdown, total));
        }

        let allowed: HashSet<_> = audit
            .iter()
            .map(|audit_case| audit_case.case.case_id.as_str())
            .collect();
        let mut events = Vec::new();
        let mut families = HashSet::new();
        for (index, raw_candidate) in envelope
            .lesson_proposals
            .into_iter()
            .take(MAX_PROPOSALS)
            .enumerate()
        {
            let Ok(candidate) = serde_json::from_value::<CandidateLesson>(raw_candidate) else {
                continue;
            };
            if candidate.confidence_millis < MIN_AUTOMATIC_CONFIDENCE
                || candidate.source_case_ids.is_empty()
                || candidate
                    .source_case_ids
                    .iter()
                    .any(|id| !allowed.contains(id.as_str()))
                || !candidate_matches_run_outcome(&candidate, outcome, expected_primary)
            {
                continue;
            }
            let proposal = candidate.into_proposal(
                self.provenance.locale.clone(),
                self.provenance.model_profile_sha256.clone(),
            );
            let Ok(lesson) = Lesson::propose(proposal, &self.snapshot.cases) else {
                continue;
            };
            if !families.insert(lesson.family_key.clone()) {
                continue;
            }
            let kind = if lesson.status == LessonStatus::Proposed {
                LessonEventKind::Proposed
            } else {
                LessonEventKind::Recalculated
            };
            if let Ok(event) = LessonEvent::new(
                kind,
                lesson,
                None,
                crate::journal::timestamp_ms() + index as u128,
            ) {
                events.push(event);
            }
        }
        let accepted = events.len();
        if !events.is_empty() {
            self.store.append_lesson_events(&events)?;
            let bundle = embedded_bundle()
                .map_err(|error| anyhow::anyhow!("embedded bundle invalid: {error:?}"))?;
            self.snapshot = self.store.rebuild(&[bundle])?;
            self.store.write_status(
                &self.snapshot,
                self.config.mode,
                self.last_eligibility.as_ref(),
            )?;
        }
        Ok(CriticIngest {
            report_markdown: envelope.report_markdown,
            response_valid: true,
            accepted_lessons: accepted,
            rejected_lessons: total.saturating_sub(accepted),
            snapshot_id: self.snapshot.snapshot_id.clone(),
        })
    }

    fn empty_ingest(&self, report: &str) -> CriticIngest {
        CriticIngest {
            report_markdown: report.to_string(),
            response_valid: false,
            accepted_lessons: 0,
            rejected_lessons: 0,
            snapshot_id: self.snapshot.snapshot_id.clone(),
        }
    }

    fn invalid_ingest(&self, report_markdown: String, rejected_lessons: usize) -> CriticIngest {
        CriticIngest {
            report_markdown,
            response_valid: false,
            accepted_lessons: 0,
            rejected_lessons,
            snapshot_id: self.snapshot.snapshot_id.clone(),
        }
    }
}
