use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::learning::lesson::{StrategicEvidence, StrategicHypothesis};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CriticEnvelope {
    pub(super) schema_version: u32,
    pub(super) report_markdown: String,
    pub(super) result: CriticResult,
    pub(super) lesson: Option<CandidateLesson>,
    pub(super) rejected_lesson_analysis: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CriticResult {
    Lesson,
    NoLesson,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CandidateLesson {
    text: String,
    applies_when: String,
    expected_effect: String,
    evidence: Vec<CandidateEvidence>,
    uncertainty: String,
    pub(super) confidence_millis: u16,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CandidateEvidence {
    run_id: String,
    decision_ids: Vec<String>,
    observed_chain: String,
}

impl CandidateLesson {
    pub(super) fn text(&self) -> &str {
        &self.text
    }

    pub(super) fn validate(
        &self,
        allowed: &HashMap<&str, HashSet<&str>>,
        current_run_id: &str,
    ) -> bool {
        if self.confidence_millis > 1_000
            || self.evidence.is_empty()
            || self.evidence.len() > 3
            || !safe_text(&self.text, 1_024)
            || !safe_text(&self.applies_when, 1_024)
            || !safe_text(&self.expected_effect, 1_024)
            || !safe_text(&self.uncertainty, 1_024)
        {
            return false;
        }
        let mut cites_current = false;
        let mut unique = HashSet::new();
        for evidence in &self.evidence {
            let Some(decisions) = allowed.get(evidence.run_id.as_str()) else {
                return false;
            };
            if evidence.decision_ids.is_empty()
                || evidence.decision_ids.len() > 12
                || !safe_text(&evidence.observed_chain, 1_024)
                || evidence
                    .decision_ids
                    .iter()
                    .any(|id| !decisions.contains(id.as_str()) || !unique.insert(id.as_str()))
            {
                return false;
            }
            cites_current |= evidence.run_id == current_run_id;
        }
        cites_current
    }

    pub(super) fn source_decision_ids(&self) -> impl Iterator<Item = &str> {
        self.evidence
            .iter()
            .flat_map(|evidence| evidence.decision_ids.iter().map(String::as_str))
    }

    pub(super) fn review_claims(&self) -> BTreeMap<String, String> {
        let mut claims = BTreeMap::from([
            ("lesson.text".into(), self.text.clone()),
            ("lesson.applies_when".into(), self.applies_when.clone()),
            (
                "lesson.expected_effect".into(),
                self.expected_effect.clone(),
            ),
            ("lesson.uncertainty".into(), self.uncertainty.clone()),
        ]);
        for (index, evidence) in self.evidence.iter().enumerate() {
            claims.insert(
                format!("lesson.evidence[{index}].observed_chain"),
                evidence.observed_chain.clone(),
            );
        }
        claims
    }

    pub(super) fn into_strategy(self) -> StrategicHypothesis {
        StrategicHypothesis {
            text: self.text,
            applies_when: self.applies_when,
            expected_effect: self.expected_effect,
            evidence: self
                .evidence
                .into_iter()
                .map(|evidence| StrategicEvidence {
                    run_id: evidence.run_id,
                    decision_ids: evidence.decision_ids,
                    observed_chain: evidence.observed_chain,
                })
                .collect(),
            uncertainty: self.uncertainty,
        }
    }
}

pub(super) fn valid_result_shape(envelope: &CriticEnvelope) -> bool {
    matches!(
        (envelope.result, envelope.lesson.is_some()),
        (CriticResult::Lesson, true) | (CriticResult::NoLesson, false)
    )
}

pub(super) fn safe_text(value: &str, maximum: usize) -> bool {
    let lower = value.to_ascii_lowercase();
    !value.trim().is_empty()
        && value.len() <= maximum
        && !value
            .chars()
            .any(|character| character.is_control() && character != '\n' && character != '\t')
        && !["```", "<script", "http://", "https://"]
            .iter()
            .any(|needle| lower.contains(needle))
}
