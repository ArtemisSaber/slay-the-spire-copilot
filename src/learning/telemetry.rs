use crate::autoplay::action::AutoPlayAction;
use crate::learning::action::SemanticAction;
use crate::learning::descriptor::SituationDescriptor;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionSource {
    Deterministic,
    KillScan,
    Llm,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedAction {
    pub action_id: String,
    pub semantic_action: SemanticAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedRankedAction {
    pub semantic_action: SemanticAction,
    pub score: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedDecision {
    pub decision_id: String,
    pub selected_action_id: String,
    pub selected_semantic_action: SemanticAction,
    pub action: AutoPlayAction,
    pub source: DecisionSource,
    pub available_actions: Vec<RecordedAction>,
    pub ranked_suggestions: Vec<RecordedRankedAction>,
    pub retrieved_memory_ids: Vec<String>,
    pub memory_ids_used: Vec<String>,
    pub knowledge_snapshot_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionIdGenerator {
    run_id: String,
    sequence: u64,
}

impl DecisionIdGenerator {
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            sequence: 0,
        }
    }

    pub fn next(&mut self, floor: Option<i64>, turn: Option<i64>) -> String {
        self.sequence += 1;
        format!(
            "{}:{}:{}:{}",
            self.run_id,
            display_number(floor),
            display_number(turn),
            self.sequence
        )
    }
}

pub fn proposal_event(
    decision: &PlannedDecision,
    observation_hash: &str,
    situation: &SituationDescriptor,
) -> Result<Value, serde_json::Error> {
    let situation_hash = situation.situation_hash()?;
    Ok(json!({
        "schema_version": 1,
        "event": "autoplay_decision_proposed",
        "decision_id": decision.decision_id,
        "state_observation_hash": observation_hash,
        "descriptor_version": situation.descriptor_version,
        "situation_hash": situation_hash,
        "source": decision.source,
        "selected_action_id": decision.selected_action_id,
        "selected_semantic_action": decision.selected_semantic_action,
        "available_actions": decision.available_actions,
        "ranked_suggestions": decision.ranked_suggestions,
        "retrieved_memory_ids": decision.retrieved_memory_ids,
        "memory_ids_used": decision.memory_ids_used,
        "knowledge_snapshot_id": decision.knowledge_snapshot_id,
    }))
}

fn display_number(value: Option<i64>) -> String {
    value.map_or_else(|| "unknown".to_string(), |value| value.to_string())
}
