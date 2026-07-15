use crate::learning::action::SemanticAction;
use crate::learning::descriptor::SituationDescriptor;
use crate::learning::telemetry::{DecisionSource, RecordedRankedAction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseOutcome {
    pub command_succeeded: bool,
    pub turn_hp_lost: Option<i64>,
    pub combat_completed: bool,
    pub combat_won: Option<bool>,
    pub combat_hp_lost: Option<i64>,
    pub combat_turns: Option<i64>,
    pub potions_used: Vec<String>,
    pub run_completed: bool,
    pub run_victory: Option<bool>,
    pub final_floor: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseProvenance {
    pub app_version: String,
    pub prompt_schema_version: u32,
    pub rules_sha256: String,
    pub model_profile_sha256: String,
    pub mod_profile_sha256: String,
    pub knowledge_snapshot_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionCase {
    pub schema_version: u32,
    pub case_id: String,
    pub run_id: String,
    pub decision_id: String,
    pub seed_hash: String,
    pub situation_hash: String,
    pub situation: SituationDescriptor,
    pub selected_action: SemanticAction,
    pub decision_source: DecisionSource,
    pub available_semantic_actions: Vec<SemanticAction>,
    pub ranked_suggestions: Vec<RecordedRankedAction>,
    pub retrieved_memory_ids: Vec<String>,
    pub memory_ids_used: Vec<String>,
    pub outcome: CaseOutcome,
    pub provenance: CaseProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseDraft {
    pub run_id: String,
    pub decision_id: String,
    pub seed_hash: String,
    pub situation: SituationDescriptor,
    pub selected_action: SemanticAction,
    pub decision_source: DecisionSource,
    pub available_semantic_actions: Vec<SemanticAction>,
    pub ranked_suggestions: Vec<RecordedRankedAction>,
    pub retrieved_memory_ids: Vec<String>,
    pub memory_ids_used: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseError {
    TooManyAvailableActions,
    TooManyRankedSuggestions,
    SelectedActionUnavailable,
    IncompleteOutcome,
    InvalidIdentifier,
    Serialization,
}

impl CaseDraft {
    pub fn finalize(
        self,
        outcome: CaseOutcome,
        provenance: CaseProvenance,
    ) -> Result<DecisionCase, CaseError> {
        self.validate(&outcome, &provenance)?;
        let situation_hash = self
            .situation
            .situation_hash()
            .map_err(|_| CaseError::Serialization)?;
        let mut case = DecisionCase {
            schema_version: 1,
            case_id: String::new(),
            run_id: self.run_id,
            decision_id: self.decision_id,
            seed_hash: self.seed_hash,
            situation_hash,
            situation: self.situation,
            selected_action: self.selected_action,
            decision_source: self.decision_source,
            available_semantic_actions: self.available_semantic_actions,
            ranked_suggestions: self.ranked_suggestions,
            retrieved_memory_ids: self.retrieved_memory_ids,
            memory_ids_used: self.memory_ids_used,
            outcome,
            provenance,
        };
        case.case_id = case.compute_id()?;
        Ok(case)
    }

    fn validate(
        &self,
        outcome: &CaseOutcome,
        provenance: &CaseProvenance,
    ) -> Result<(), CaseError> {
        if self.available_semantic_actions.len() > 32 {
            return Err(CaseError::TooManyAvailableActions);
        }
        if self.ranked_suggestions.len() > 64 {
            return Err(CaseError::TooManyRankedSuggestions);
        }
        if !self
            .available_semantic_actions
            .contains(&self.selected_action)
        {
            return Err(CaseError::SelectedActionUnavailable);
        }
        if !complete_outcome(outcome) {
            return Err(CaseError::IncompleteOutcome);
        }
        let ids = [
            self.run_id.as_str(),
            self.decision_id.as_str(),
            self.seed_hash.as_str(),
            provenance.app_version.as_str(),
            provenance.rules_sha256.as_str(),
            provenance.model_profile_sha256.as_str(),
            provenance.mod_profile_sha256.as_str(),
        ]
        .into_iter()
        .chain(self.retrieved_memory_ids.iter().map(String::as_str))
        .chain(self.memory_ids_used.iter().map(String::as_str))
        .chain(outcome.potions_used.iter().map(String::as_str));
        if ids.into_iter().any(|id| !valid_identifier(id)) {
            return Err(CaseError::InvalidIdentifier);
        }
        Ok(())
    }
}

impl DecisionCase {
    pub fn verify_id(&self) -> bool {
        self.schema_version == 1
            && self.situation.descriptor_version == 1
            && self.situation.ranker_tag_schema_version == 1
            && self.available_semantic_actions.len() <= 32
            && self.ranked_suggestions.len() <= 64
            && self
                .available_semantic_actions
                .contains(&self.selected_action)
            && complete_outcome(&self.outcome)
            && self
                .situation
                .situation_hash()
                .is_ok_and(|hash| hash == self.situation_hash)
            && self.compute_id().is_ok_and(|id| id == self.case_id)
    }

    fn compute_id(&self) -> Result<String, CaseError> {
        let identity = CaseIdentity {
            schema_version: self.schema_version,
            run_id: &self.run_id,
            decision_id: &self.decision_id,
            seed_hash: &self.seed_hash,
            situation_hash: &self.situation_hash,
            situation: &self.situation,
            selected_action: &self.selected_action,
            decision_source: self.decision_source,
            available_semantic_actions: &self.available_semantic_actions,
            ranked_suggestions: &self.ranked_suggestions,
            retrieved_memory_ids: &self.retrieved_memory_ids,
            memory_ids_used: &self.memory_ids_used,
            outcome: &self.outcome,
            provenance: &self.provenance,
        };
        let bytes = serde_json::to_vec(&identity).map_err(|_| CaseError::Serialization)?;
        Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
    }
}

#[derive(Serialize)]
struct CaseIdentity<'a> {
    schema_version: u32,
    run_id: &'a str,
    decision_id: &'a str,
    seed_hash: &'a str,
    situation_hash: &'a str,
    situation: &'a SituationDescriptor,
    selected_action: &'a SemanticAction,
    decision_source: DecisionSource,
    available_semantic_actions: &'a [SemanticAction],
    ranked_suggestions: &'a [RecordedRankedAction],
    retrieved_memory_ids: &'a [String],
    memory_ids_used: &'a [String],
    outcome: &'a CaseOutcome,
    provenance: &'a CaseProvenance,
}

pub fn seed_hash(seed: i64, profile_salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"sts-copilot-memory-seed-v1\0");
    hasher.update(profile_salt.as_bytes());
    hasher.update(b"\0");
    hasher.update(seed.to_le_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

fn valid_identifier(id: &str) -> bool {
    !id.is_empty() && id.trim() == id && id.len() <= 256 && !id.contains(['\n', '\r', '\0'])
}

fn complete_outcome(outcome: &CaseOutcome) -> bool {
    outcome.command_succeeded
        && outcome.combat_completed
        && outcome.combat_won.is_some()
        && outcome.run_completed
        && outcome.run_victory.is_some()
}
