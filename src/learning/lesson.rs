use crate::learning::case::DecisionCase;
use crate::learning::descriptor::{AscensionBand, BlockThreatBucket, TurnBucket};
use crate::learning::eligibility::RunObjective;
use serde::{Deserialize, Serialize};

mod event;
mod identity;
mod matching;
mod validation;

pub use event::{LessonEvent, LessonEventKind};
pub(crate) use validation::lesson_guidance_is_coherent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonStatus {
    Proposed,
    Supported,
    Validated,
    Contested,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    PlayCard,
    UsePotion,
    EndTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeCode {
    CombatDeath,
    CombatWin,
    HighCombatHpLoss,
    LowCombatHpLoss,
    PotionSpent,
    PotionPreserved,
    TurnDamageTaken,
    CombatCompletedQuickly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuidanceKind {
    Caution,
    Consider,
    Avoid,
    Prefer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LessonScope {
    pub character: String,
    pub objective: RunObjective,
    pub ascension_bands: Vec<AscensionBand>,
    pub encounter_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LessonTrigger {
    pub turn_buckets: Vec<TurnBucket>,
    pub block_threat_buckets: Vec<BlockThreatBucket>,
    pub required_card_ids: Vec<String>,
    pub required_enemy_power_ids: Vec<String>,
    pub required_ranker_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPattern {
    pub kind: ActionKind,
    pub card_types: Vec<String>,
    pub card_ids: Vec<String>,
    pub potion_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guidance {
    pub kind: GuidanceKind,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportStats {
    pub independent_cases: usize,
    pub dependent_cases: usize,
    pub contradicting_cases: usize,
    pub distinct_independent_seeds: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Critic {
    pub model_profile_sha256: String,
    pub confidence_millis: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub schema_version: u32,
    pub lesson_id: String,
    pub family_key: String,
    pub status: LessonStatus,
    pub language: String,
    pub outcome_predicate_version: u32,
    pub scope: LessonScope,
    pub trigger: LessonTrigger,
    pub action_pattern: ActionPattern,
    pub outcome_code: OutcomeCode,
    pub guidance: Guidance,
    pub rationale: String,
    pub source_case_ids: Vec<String>,
    pub support: SupportStats,
    pub critic: Critic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonProposal {
    pub language: String,
    pub scope: LessonScope,
    pub trigger: LessonTrigger,
    pub action_pattern: ActionPattern,
    pub outcome_code: OutcomeCode,
    pub guidance: Guidance,
    pub rationale: String,
    pub source_case_ids: Vec<String>,
    pub critic_model_profile_sha256: String,
    pub confidence_millis: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LessonError {
    UnknownSourceCase,
    SourceDoesNotMatch,
    OutcomeNotObserved,
    InvalidActionPattern,
    IncoherentGuidance,
    UnsupportedOutcome,
    InvalidIdentifier,
    UnsafeText,
    InvalidConfidence,
    InvalidHumanStatus,
    Serialization,
}

impl Lesson {
    pub fn propose(
        mut proposal: LessonProposal,
        cases: &[DecisionCase],
    ) -> Result<Self, LessonError> {
        validation::validate_and_canonicalize(&mut proposal)?;
        let cited = proposal
            .source_case_ids
            .iter()
            .map(|id| {
                cases
                    .iter()
                    .find(|case| &case.case_id == id)
                    .ok_or(LessonError::UnknownSourceCase)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if cited
            .iter()
            .any(|case| !matching::matches_proposal(&proposal, case))
        {
            return Err(LessonError::SourceDoesNotMatch);
        }
        if !cited.iter().any(|case| {
            matching::matches_proposal(&proposal, case)
                && matching::outcome_matches(proposal.outcome_code, case) == Some(true)
        }) {
            return Err(LessonError::OutcomeNotObserved);
        }
        let mut lesson = Self {
            schema_version: 1,
            lesson_id: String::new(),
            family_key: String::new(),
            status: LessonStatus::Proposed,
            language: proposal.language,
            outcome_predicate_version: 1,
            scope: proposal.scope,
            trigger: proposal.trigger,
            action_pattern: proposal.action_pattern,
            outcome_code: proposal.outcome_code,
            guidance: proposal.guidance,
            rationale: proposal.rationale,
            source_case_ids: proposal.source_case_ids,
            support: SupportStats::default(),
            critic: Critic {
                model_profile_sha256: proposal.critic_model_profile_sha256,
                confidence_millis: proposal.confidence_millis,
            },
        };
        lesson.family_key = identity::family_key(&lesson)?;
        lesson.lesson_id = identity::lesson_id(&lesson)?;
        lesson.recalculate_support(cases);
        Ok(lesson)
    }

    pub fn recalculate_support(&mut self, cases: &[DecisionCase]) {
        self.support = matching::support(self, cases);
        if matches!(
            self.status,
            LessonStatus::Validated | LessonStatus::Contested | LessonStatus::Retired
        ) {
            return;
        }
        let total = self.support.independent_cases
            + self.support.dependent_cases
            + self.support.contradicting_cases;
        if self.support.contradicting_cases >= 3
            && self.support.contradicting_cases * 100 >= total.max(1) * 40
        {
            self.status = LessonStatus::Contested;
        } else if self.support.independent_cases >= 5
            && self.support.distinct_independent_seeds >= 5
            && self.support.contradicting_cases <= self.support.independent_cases / 3
        {
            self.status = LessonStatus::Supported;
        } else {
            self.status = LessonStatus::Proposed;
        }
    }

    pub fn set_human_status(&mut self, status: LessonStatus) -> Result<(), LessonError> {
        if !matches!(
            status,
            LessonStatus::Validated | LessonStatus::Contested | LessonStatus::Retired
        ) {
            return Err(LessonError::InvalidHumanStatus);
        }
        self.status = status;
        Ok(())
    }

    pub fn verify_identity(&self) -> bool {
        self.schema_version == 1
            && self.outcome_predicate_version == 1
            && identity::family_key(self).is_ok_and(|key| key == self.family_key)
            && identity::lesson_id(self).is_ok_and(|id| id == self.lesson_id)
    }

    pub(crate) fn matches_situation(
        &self,
        situation: &crate::learning::descriptor::SituationDescriptor,
    ) -> bool {
        matching::lesson_matches_situation(self, situation)
    }

    pub(crate) fn is_independent_support(&self, case: &DecisionCase) -> bool {
        matching::lesson_matches_case(self, case)
            && matching::outcome_matches(self.outcome_code, case) == Some(true)
            && !case
                .retrieved_memory_ids
                .iter()
                .any(|id| id == &self.lesson_id)
    }
}
