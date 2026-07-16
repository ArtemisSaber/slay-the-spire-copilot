use super::{
    ActionKind, ActionPattern, Critic, Guidance, GuidanceKind, Lesson, LessonError, LessonScope,
    LessonStatus, LessonTrigger, OutcomeCode, SupportStats,
};
use crate::learning::case::DecisionCase;
use crate::learning::eligibility::RunObjective;
use serde::{Deserialize, Serialize};

mod validation;
mod verification;

const MAX_RECENT_TRIALS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategicHypothesis {
    pub text: String,
    pub applies_when: String,
    pub expected_effect: String,
    pub evidence: Vec<StrategicEvidence>,
    pub uncertainty: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategicEvidence {
    pub run_id: String,
    pub decision_ids: Vec<String>,
    pub observed_chain: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LessonBenchmark {
    pub origin_run_id: String,
    pub character: String,
    pub ascension_level: i64,
    pub objective: RunObjective,
    pub final_floor: i64,
    pub victory: bool,
    pub compatibility_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LessonTrial {
    pub run_id: String,
    pub final_floor: i64,
    pub victory: bool,
    pub degraded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LessonLifecycle {
    pub benchmark: LessonBenchmark,
    pub parent_lesson_id: Option<String>,
    pub generation: u32,
    pub qualifying_runs: u32,
    pub consecutive_degraded_runs: u8,
    pub recent_trials: Vec<LessonTrial>,
    #[serde(default)]
    pub regeneration_resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategicLessonProposal {
    pub language: String,
    pub scope: LessonScope,
    pub strategy: StrategicHypothesis,
    pub source_case_ids: Vec<String>,
    pub critic_model_profile_sha256: String,
    pub confidence_millis: u16,
    pub benchmark: LessonBenchmark,
    pub parent_lesson_id: Option<String>,
    pub generation: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialDisposition {
    Ignored,
    Retained,
    Retired,
}

pub(super) fn verify_structure(lesson: &Lesson) -> bool {
    verification::verify_structure(lesson)
}

pub(super) fn verify_sources(lesson: &Lesson, cases: &[DecisionCase]) -> bool {
    verification::verify_sources(lesson, cases)
}

impl Lesson {
    pub fn propose_strategy(
        mut proposal: StrategicLessonProposal,
        cases: &[DecisionCase],
    ) -> Result<Self, LessonError> {
        validation::validate_proposal(&mut proposal, cases)?;
        let mut lesson = Self {
            schema_version: 2,
            lesson_id: String::new(),
            family_key: String::new(),
            status: LessonStatus::Proposed,
            language: proposal.language,
            outcome_predicate_version: 1,
            scope: proposal.scope,
            trigger: LessonTrigger {
                turn_buckets: vec![],
                block_threat_buckets: vec![],
                required_card_ids: vec![],
                required_enemy_power_ids: vec![],
                required_ranker_tags: vec![],
            },
            action_pattern: ActionPattern {
                kind: ActionKind::StrategicPolicy,
                card_types: vec![],
                card_ids: vec![],
                potion_ids: vec![],
            },
            outcome_code: OutcomeCode::RunProgression,
            guidance: Guidance {
                kind: GuidanceKind::Experimental,
                text: proposal.strategy.text.clone(),
            },
            rationale: proposal.strategy.expected_effect.clone(),
            source_case_ids: proposal.source_case_ids,
            support: SupportStats::default(),
            critic: Critic {
                model_profile_sha256: proposal.critic_model_profile_sha256,
                confidence_millis: proposal.confidence_millis,
            },
            strategy: Some(proposal.strategy),
            lifecycle: Some(LessonLifecycle {
                benchmark: proposal.benchmark,
                parent_lesson_id: proposal.parent_lesson_id,
                generation: proposal.generation,
                qualifying_runs: 0,
                consecutive_degraded_runs: 0,
                recent_trials: vec![],
                regeneration_resolved: false,
            }),
        };
        lesson.family_key = super::identity::family_key(&lesson)?;
        lesson.lesson_id = super::identity::lesson_id(&lesson)?;
        Ok(lesson)
    }

    pub fn record_trial(&mut self, run: &LessonBenchmark) -> Result<TrialDisposition, LessonError> {
        if !matches!(
            self.status,
            LessonStatus::Proposed | LessonStatus::Supported | LessonStatus::Validated
        ) {
            return Ok(TrialDisposition::Ignored);
        }
        let Some(lifecycle) = self.lifecycle.as_mut() else {
            return Ok(TrialDisposition::Ignored);
        };
        if !comparable(&lifecycle.benchmark, run)
            || run.origin_run_id == lifecycle.benchmark.origin_run_id
            || lifecycle
                .recent_trials
                .last()
                .is_some_and(|trial| trial.run_id == run.origin_run_id)
        {
            return Ok(TrialDisposition::Ignored);
        }
        let degraded = degraded(&lifecycle.benchmark, run);
        lifecycle.qualifying_runs = lifecycle.qualifying_runs.saturating_add(1);
        lifecycle.consecutive_degraded_runs = if degraded {
            lifecycle.consecutive_degraded_runs.saturating_add(1)
        } else {
            0
        };
        lifecycle.recent_trials.push(LessonTrial {
            run_id: run.origin_run_id.clone(),
            final_floor: run.final_floor,
            victory: run.victory,
            degraded,
        });
        if lifecycle.recent_trials.len() > MAX_RECENT_TRIALS {
            lifecycle.recent_trials.remove(0);
        }
        if lifecycle.consecutive_degraded_runs >= 2 {
            self.status = LessonStatus::Retired;
            return Ok(TrialDisposition::Retired);
        }
        Ok(TrialDisposition::Retained)
    }
}

fn comparable(baseline: &LessonBenchmark, run: &LessonBenchmark) -> bool {
    baseline.character == run.character
        && baseline.ascension_level == run.ascension_level
        && baseline.objective == run.objective
        && baseline.compatibility_sha256 == run.compatibility_sha256
}

fn degraded(baseline: &LessonBenchmark, run: &LessonBenchmark) -> bool {
    if baseline.victory {
        !run.victory
    } else {
        !run.victory && run.final_floor < baseline.final_floor
    }
}
