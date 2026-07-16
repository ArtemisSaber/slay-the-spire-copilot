use crate::learning::case::DecisionCase;
use crate::learning::descriptor::{AscensionBand, BlockThreatBucket, TurnBucket};
use crate::learning::eligibility::RunObjective;
use serde::{Deserialize, Serialize};

mod event;
mod identity;
mod legacy;
mod matching;
mod strategy;
mod validation;

pub use event::{LessonEvent, LessonEventKind};
pub use strategy::{
    LessonBenchmark, LessonLifecycle, StrategicEvidence, StrategicHypothesis,
    StrategicLessonProposal, TrialDisposition,
};
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
    StrategicPolicy,
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
    RunProgression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuidanceKind {
    Caution,
    Consider,
    Avoid,
    Prefer,
    Experimental,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<StrategicHypothesis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<LessonLifecycle>,
}

#[cfg(test)]
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
    #[cfg(test)]
    SourceDoesNotMatch,
    #[cfg(test)]
    OutcomeNotObserved,
    #[cfg(test)]
    InvalidActionPattern,
    IncoherentGuidance,
    UnsupportedOutcome,
    #[cfg(test)]
    InvalidIdentifier,
    UnsafeText,
    #[cfg(test)]
    InvalidConfidence,
    InvalidLifecycle,
    InvalidHumanStatus,
    Serialization,
}

impl Lesson {
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
        matches!(self.schema_version, 1 | 2)
            && (self.schema_version == 1) == self.strategy.is_none()
            && (self.schema_version == 1) == self.lifecycle.is_none()
            && self.outcome_predicate_version == 1
            && (self.schema_version == 1 || strategy::verify_structure(self))
            && identity::family_key(self).is_ok_and(|key| key == self.family_key)
            && identity::lesson_id(self).is_ok_and(|id| id == self.lesson_id)
    }

    pub fn is_strategic(&self) -> bool {
        self.strategy.is_some() && self.lifecycle.is_some()
    }

    pub(crate) fn sources_are_valid(&self, cases: &[DecisionCase]) -> bool {
        !self.is_strategic() || strategy::verify_sources(self, cases)
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
