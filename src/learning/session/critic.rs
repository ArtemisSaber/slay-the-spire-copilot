use super::LearningSession;
use crate::learning::bundle::embedded_bundle;
use crate::learning::lesson::{
    Lesson, LessonEvent, LessonEventKind, LessonScope, StrategicLessonProposal,
};

mod contract;
mod evidence;
mod mode;
mod prompt;

use contract::{CriticEnvelope, CriticResult, safe_text, valid_result_shape};
use evidence::{allowed_decisions, source_case_ids, supplied_cases};
use mode::{CriticMode, context_for_run};
use prompt::{critic_appendix, truncate_utf8, valid_report};

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
    pub fn build_critic_prompt(&self, base_report_prompt: &str, run_id: &str) -> Option<String> {
        let context = context_for_run(self, run_id)?;
        let cases = supplied_cases(&self.snapshot.cases, run_id, context.mode);
        if cases.is_empty() {
            return None;
        }
        let base = truncate_utf8(base_report_prompt, MAX_REPORT_BYTES);
        let appendix = critic_appendix(&context, &cases);
        let prompt = format!("{base}\n\nLEARNING_CRITIC_ENVELOPE_V3\n{appendix}");
        (prompt.len() <= MAX_PROMPT_BYTES).then_some(prompt)
    }

    pub fn ingest_critic_response(
        &mut self,
        response: &str,
        run_id: &str,
    ) -> anyhow::Result<CriticIngest> {
        let Ok(envelope) = serde_json::from_str::<CriticEnvelope>(response.trim()) else {
            return Ok(self.empty_ingest(response));
        };
        let Some(context) = context_for_run(self, run_id) else {
            return Ok(self.invalid_ingest(envelope.report_markdown, envelope.lesson.is_some()));
        };
        let cases = supplied_cases(&self.snapshot.cases, run_id, context.mode);
        let allowed = allowed_decisions(&cases);
        let report_valid = valid_report(&envelope.report_markdown, MAX_REPORT_BYTES);
        let shape_valid =
            envelope.schema_version == ENVELOPE_VERSION && valid_result_shape(&envelope);
        let regeneration_valid = match context.mode {
            CriticMode::Regenerate(parent) => {
                envelope
                    .rejected_lesson_analysis
                    .as_deref()
                    .is_some_and(|analysis| safe_text(analysis, 1_024))
                    && envelope.lesson.as_ref().is_none_or(|candidate| {
                        parent.strategy.as_ref().is_none_or(|strategy| {
                            candidate_text(candidate) != normalize(&strategy.text)
                        })
                    })
            }
            CriticMode::Initial | CriticMode::ReportOnly => {
                envelope.rejected_lesson_analysis.is_none()
            }
        };
        let mode_valid = !matches!(context.mode, CriticMode::ReportOnly)
            || envelope.result == CriticResult::NoLesson;
        if !report_valid || !shape_valid || !regeneration_valid || !mode_valid {
            return Ok(self.invalid_ingest(envelope.report_markdown, envelope.lesson.is_some()));
        }
        let Some(candidate) = envelope.lesson else {
            if let Some(parent_id) = context.mode.parent().map(|parent| parent.lesson_id.clone()) {
                self.resolve_regeneration_without_replacement(&parent_id)?;
            }
            return Ok(self.valid_ingest(envelope.report_markdown, 0, 0));
        };
        if !candidate.validate(&allowed, run_id) {
            return Ok(self.valid_ingest(envelope.report_markdown, 0, 1));
        }
        let decision_ids: Vec<_> = candidate
            .source_decision_ids()
            .map(str::to_string)
            .collect();
        let Some(source_case_ids) = source_case_ids(&cases, decision_ids.into_iter()) else {
            return Ok(self.valid_ingest(envelope.report_markdown, 0, 1));
        };
        let cited: Vec<_> = source_case_ids
            .iter()
            .filter_map(|id| self.snapshot.cases.iter().find(|case| &case.case_id == id))
            .collect();
        let Some(scope) = strategic_scope(&cited, &context.benchmark) else {
            return Ok(self.valid_ingest(envelope.report_markdown, 0, 1));
        };
        let (parent_lesson_id, generation) = context
            .mode
            .parent()
            .and_then(|parent| {
                parent
                    .lifecycle
                    .as_ref()
                    .map(|lifecycle| (Some(parent.lesson_id.clone()), lifecycle.generation + 1))
            })
            .unwrap_or((None, 1));
        let confidence_millis = candidate.confidence_millis;
        let strategy = candidate.into_strategy();
        let proposal = StrategicLessonProposal {
            language: self.provenance.locale.clone(),
            scope,
            strategy,
            source_case_ids,
            critic_model_profile_sha256: self.provenance.model_profile_sha256.clone(),
            confidence_millis,
            benchmark: context.benchmark.clone(),
            parent_lesson_id,
            generation,
        };
        let Ok(lesson) = Lesson::propose_strategy(proposal, &self.snapshot.cases) else {
            return Ok(self.valid_ingest(envelope.report_markdown, 0, 1));
        };
        let event = LessonEvent::new(
            LessonEventKind::Proposed,
            lesson,
            None,
            crate::journal::timestamp_ms(),
        )
        .map_err(|error| anyhow::anyhow!("strategic lesson event failed: {error:?}"))?;
        self.store.append_lesson_events(&[event])?;
        let bundle = embedded_bundle()
            .map_err(|error| anyhow::anyhow!("embedded bundle invalid: {error:?}"))?;
        self.snapshot = self.store.rebuild(&[bundle])?;
        self.store.write_status(
            &self.snapshot,
            self.config.mode,
            self.last_eligibility.as_ref(),
        )?;
        Ok(self.valid_ingest(envelope.report_markdown, 1, 0))
    }

    fn valid_ingest(&self, report: String, accepted: usize, rejected: usize) -> CriticIngest {
        CriticIngest {
            report_markdown: report,
            response_valid: true,
            accepted_lessons: accepted,
            rejected_lessons: rejected,
            snapshot_id: self.snapshot.snapshot_id.clone(),
        }
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

    fn invalid_ingest(&self, report: String, had_lesson: bool) -> CriticIngest {
        CriticIngest {
            report_markdown: report,
            response_valid: false,
            accepted_lessons: 0,
            rejected_lessons: usize::from(had_lesson),
            snapshot_id: self.snapshot.snapshot_id.clone(),
        }
    }
}

fn strategic_scope(
    cited: &[&crate::learning::case::DecisionCase],
    benchmark: &crate::learning::lesson::LessonBenchmark,
) -> Option<LessonScope> {
    let first = cited
        .iter()
        .find(|case| case.run_id == benchmark.origin_run_id)?;
    let encounters = if cited
        .iter()
        .all(|case| case.situation.encounter_ids == first.situation.encounter_ids)
    {
        first.situation.encounter_ids.clone()
    } else {
        vec![]
    };
    Some(LessonScope {
        character: benchmark.character.clone(),
        objective: benchmark.objective,
        ascension_bands: vec![first.situation.ascension_band],
        encounter_ids: encounters,
    })
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn candidate_text(candidate: &contract::CandidateLesson) -> String {
    normalize(candidate.text())
}
