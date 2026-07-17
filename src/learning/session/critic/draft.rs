use std::collections::{BTreeMap, HashSet};

use serde_json::Value;

use super::contract::{CandidateLesson, CriticEnvelope, CriticResult, valid_result_shape};
use super::evidence::{allowed_decisions, source_case_ids, supplied_cases};
use super::mode::context_for_run;
use super::{CriticIngest, ENVELOPE_VERSION, MAX_REPORT_BYTES};
use crate::learning::bundle::embedded_bundle;
use crate::learning::lesson::{Lesson, LessonEvent, LessonEventKind, StrategicLessonProposal};
use crate::learning::session::LearningSession;

mod validation;

use validation::{regeneration_is_valid, strategic_scope};

pub(crate) struct CriticDraft {
    report_markdown: String,
    candidate: CandidateLesson,
    lesson: Lesson,
    allowed_decision_ids: HashSet<String>,
}

impl CriticDraft {
    pub(crate) fn report_markdown(&self) -> &str {
        &self.report_markdown
    }

    pub(crate) fn candidate_json(&self) -> Value {
        serde_json::to_value(&self.candidate).expect("candidate lesson is serializable")
    }

    pub(crate) fn allowed_decision_ids(&self) -> &HashSet<String> {
        &self.allowed_decision_ids
    }

    pub(crate) fn review_claims(&self) -> BTreeMap<String, String> {
        self.candidate.review_claims()
    }
}

pub(crate) struct CriticDraftRejection {
    report_markdown: String,
    response_valid: bool,
    rejected_lessons: usize,
    feedback: &'static str,
    abstained: bool,
}

impl CriticDraftRejection {
    pub(crate) fn report_markdown(&self) -> &str {
        &self.report_markdown
    }

    pub(crate) fn response_valid(&self) -> bool {
        self.response_valid
    }

    pub(crate) fn feedback(&self) -> &'static str {
        self.feedback
    }

    pub(crate) fn rejected_lessons(&self) -> usize {
        self.rejected_lessons
    }

    pub(crate) fn abstained(&self) -> bool {
        self.abstained
    }

    #[cfg(test)]
    fn into_ingest(self, learning: &LearningSession) -> CriticIngest {
        learning.deliberation_ingest(
            self.report_markdown,
            self.response_valid,
            self.rejected_lessons,
        )
    }

    fn malformed(response: &str) -> Self {
        Self {
            report_markdown: response.to_string(),
            response_valid: false,
            rejected_lessons: 0,
            feedback: "Return one strict lesson-proposer JSON envelope matching schema version 3.",
            abstained: false,
        }
    }

    fn invalid_envelope(report_markdown: String, had_lesson: bool) -> Self {
        Self {
            report_markdown,
            response_valid: false,
            rejected_lessons: usize::from(had_lesson),
            feedback: "The proposer must return a lesson candidate matching its report, mode, and regeneration contract.",
            abstained: false,
        }
    }

    fn invalid_candidate(report_markdown: String, feedback: &'static str) -> Self {
        Self {
            report_markdown,
            response_valid: true,
            rejected_lessons: 1,
            feedback,
            abstained: false,
        }
    }

    fn abstention(report_markdown: String) -> Self {
        Self {
            report_markdown,
            response_valid: false,
            rejected_lessons: 0,
            feedback: "The proposer must return a lesson candidate; abstention is not an allowed result.",
            abstained: true,
        }
    }
}

impl LearningSession {
    pub(crate) fn prepare_critic_response(
        &self,
        response: &str,
        run_id: &str,
    ) -> Result<CriticDraft, CriticDraftRejection> {
        let Ok(envelope) = serde_json::from_str::<CriticEnvelope>(response.trim()) else {
            return Err(CriticDraftRejection::malformed(response));
        };
        let Some(context) = context_for_run(self, run_id) else {
            return Err(CriticDraftRejection::invalid_envelope(
                envelope.report_markdown,
                envelope.lesson.is_some(),
            ));
        };
        if envelope.result == CriticResult::Abstain {
            return Err(CriticDraftRejection::abstention(envelope.report_markdown));
        }
        let cases = supplied_cases(&self.snapshot.cases, run_id, context.mode);
        let allowed = allowed_decisions(&cases);
        let allowed_decision_ids = allowed
            .values()
            .flat_map(|ids| ids.iter().map(|id| (*id).to_string()))
            .collect();
        if !super::prompt::valid_report(&envelope.report_markdown, MAX_REPORT_BYTES)
            || envelope.schema_version != ENVELOPE_VERSION
            || !valid_result_shape(&envelope)
            || !regeneration_is_valid(&envelope, context.mode)
        {
            return Err(CriticDraftRejection::invalid_envelope(
                envelope.report_markdown,
                envelope.lesson.is_some(),
            ));
        }
        let Some(candidate) = envelope.lesson else {
            return Err(CriticDraftRejection::invalid_envelope(
                envelope.report_markdown,
                false,
            ));
        };
        if !candidate.validate(&allowed, run_id) {
            return Err(CriticDraftRejection::invalid_candidate(
                envelope.report_markdown,
                "Use only supplied run and decision IDs and keep every lesson field within the contract.",
            ));
        }
        let decision_ids = candidate.source_decision_ids().map(str::to_string);
        let Some(source_case_ids) = source_case_ids(&cases, decision_ids) else {
            return Err(CriticDraftRejection::invalid_candidate(
                envelope.report_markdown,
                "The lesson evidence could not be mapped to supplied decision cases.",
            ));
        };
        let cited: Vec<_> = source_case_ids
            .iter()
            .filter_map(|id| self.snapshot.cases.iter().find(|case| &case.case_id == id))
            .collect();
        let Some(scope) = strategic_scope(&cited, &context.benchmark) else {
            return Err(CriticDraftRejection::invalid_candidate(
                envelope.report_markdown,
                "The cited evidence cannot support a valid strategic lesson scope.",
            ));
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
        let proposal = StrategicLessonProposal {
            language: self.provenance.locale.clone(),
            scope,
            strategy: candidate.clone().into_strategy(),
            source_case_ids,
            critic_model_profile_sha256: self.provenance.model_profile_sha256.clone(),
            confidence_millis: candidate.confidence_millis,
            benchmark: context.benchmark.clone(),
            parent_lesson_id,
            generation,
        };
        let Ok(lesson) = Lesson::propose_strategy(proposal, &self.snapshot.cases) else {
            return Err(CriticDraftRejection::invalid_candidate(
                envelope.report_markdown,
                "The proposed lesson is internally incoherent with its cited evidence.",
            ));
        };
        Ok(CriticDraft {
            report_markdown: envelope.report_markdown,
            candidate,
            lesson,
            allowed_decision_ids,
        })
    }

    pub(crate) fn commit_critic_draft(
        &mut self,
        draft: CriticDraft,
    ) -> anyhow::Result<CriticIngest> {
        let event = LessonEvent::new(
            LessonEventKind::Proposed,
            draft.lesson,
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
        Ok(self.valid_ingest(draft.report_markdown, 1, 0))
    }

    #[cfg(test)]
    pub fn ingest_critic_response(
        &mut self,
        response: &str,
        run_id: &str,
    ) -> anyhow::Result<CriticIngest> {
        match self.prepare_critic_response(response, run_id) {
            Ok(draft) => self.commit_critic_draft(draft),
            Err(rejection) => Ok(rejection.into_ingest(self)),
        }
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
}
