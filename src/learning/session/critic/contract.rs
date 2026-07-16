use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::learning::audit::{AuditCase, AuditRole};
use crate::learning::lesson::{
    ActionPattern, Guidance, GuidanceKind, LessonProposal, LessonScope, LessonTrigger, OutcomeCode,
};

const MAX_CONTRIBUTING_CASES: usize = 5;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CriticEnvelope {
    pub(super) schema_version: u32,
    pub(super) report_markdown: String,
    pub(super) run_analysis: RunAnalysis,
    #[serde(default)]
    pub(super) lesson_proposals: Vec<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AnalysisOutcome {
    Victory,
    Defeat,
}

impl AnalysisOutcome {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Victory => "victory",
            Self::Defeat => "defeat",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RunAnalysis {
    outcome: AnalysisOutcome,
    primary_case_id: String,
    contributing_case_ids: Vec<String>,
    explanation: String,
    confidence_millis: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CandidateLesson {
    scope: LessonScope,
    trigger: LessonTrigger,
    action_pattern: ActionPattern,
    outcome_code: OutcomeCode,
    guidance: Guidance,
    rationale: String,
    pub(super) source_case_ids: Vec<String>,
    pub(super) confidence_millis: u16,
}

impl CandidateLesson {
    pub(super) fn into_proposal(self, language: String, critic_profile: String) -> LessonProposal {
        LessonProposal {
            language,
            scope: self.scope,
            trigger: self.trigger,
            action_pattern: self.action_pattern,
            outcome_code: self.outcome_code,
            guidance: self.guidance,
            rationale: self.rationale,
            source_case_ids: self.source_case_ids,
            critic_model_profile_sha256: critic_profile,
            confidence_millis: self.confidence_millis,
        }
    }
}

pub(super) fn analysis_outcome(audit: &[AuditCase<'_>]) -> Option<AnalysisOutcome> {
    let first = audit.first()?;
    if first.case.outcome.run_victory == Some(false) {
        Some(AnalysisOutcome::Defeat)
    } else if first.case.outcome.run_victory == Some(true) {
        Some(AnalysisOutcome::Victory)
    } else if audit
        .iter()
        .any(|audit_case| audit_case.case.outcome.combat_won == Some(false))
    {
        Some(AnalysisOutcome::Defeat)
    } else {
        Some(AnalysisOutcome::Victory)
    }
}

pub(super) fn primary_case_id<'a>(
    audit: &'a [AuditCase<'a>],
    outcome: AnalysisOutcome,
) -> Option<&'a str> {
    match outcome {
        AnalysisOutcome::Defeat => audit
            .iter()
            .find(|audit_case| {
                matches!(
                    audit_case.role,
                    AuditRole::DirectDeathTransition | AuditRole::TerminalDecision
                )
            })
            .map(|audit_case| audit_case.case.case_id.as_str()),
        AnalysisOutcome::Victory => audit
            .iter()
            .find(|audit_case| {
                matches!(
                    audit_case.role,
                    AuditRole::DirectVictoryTransition | AuditRole::TerminalDecision
                )
            })
            .map(|audit_case| audit_case.case.case_id.as_str()),
    }
}

pub(super) fn valid_run_analysis(
    analysis: &RunAnalysis,
    audit: &[AuditCase<'_>],
    outcome: AnalysisOutcome,
    expected_primary: &str,
) -> bool {
    if analysis.outcome != outcome
        || analysis.primary_case_id != expected_primary
        || analysis.confidence_millis > 1_000
        || !safe_text(&analysis.explanation, 1_024)
        || analysis.contributing_case_ids.len() > MAX_CONTRIBUTING_CASES
    {
        return false;
    }
    let roles: HashMap<_, _> = audit
        .iter()
        .map(|audit_case| (audit_case.case.case_id.as_str(), audit_case.role))
        .collect();
    let mut unique = HashSet::new();
    analysis.contributing_case_ids.iter().all(|id| {
        id != expected_primary
            && unique.insert(id.as_str())
            && roles
                .get(id.as_str())
                .is_some_and(|role| role.is_causal_candidate())
    })
}

pub(super) fn candidate_matches_run_outcome(
    candidate: &CandidateLesson,
    outcome: AnalysisOutcome,
    primary_case_id: &str,
) -> bool {
    if matches!(
        candidate.outcome_code,
        OutcomeCode::PotionSpent | OutcomeCode::PotionPreserved
    ) {
        return false;
    }
    match outcome {
        AnalysisOutcome::Defeat => {
            candidate.outcome_code == OutcomeCode::CombatDeath
                && matches!(
                    candidate.guidance.kind,
                    GuidanceKind::Avoid | GuidanceKind::Caution
                )
                && candidate
                    .source_case_ids
                    .iter()
                    .any(|id| id == primary_case_id)
        }
        AnalysisOutcome::Victory => {
            candidate.outcome_code == OutcomeCode::CombatWin
                && matches!(
                    candidate.guidance.kind,
                    GuidanceKind::Prefer | GuidanceKind::Consider
                )
                && candidate
                    .source_case_ids
                    .iter()
                    .any(|id| id == primary_case_id)
        }
    }
}

fn safe_text(value: &str, maximum: usize) -> bool {
    let lower = value.to_ascii_lowercase();
    !value.trim().is_empty()
        && value.len() <= maximum
        && !value
            .chars()
            .any(|character| character.is_control() && character != '\n')
        && !["```", "<script", "http://", "https://"]
            .iter()
            .any(|needle| lower.contains(needle))
}
