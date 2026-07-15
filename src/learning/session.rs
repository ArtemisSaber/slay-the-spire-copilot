use crate::autoplay::action::AutoPlayAction;
use crate::learning::action::SemanticAction;
use crate::learning::case::{CaseDraft, seed_hash};
use crate::learning::config::MemoryConfig;
use crate::learning::context::build_experience_context;
use crate::learning::descriptor::SituationDescriptor;
#[cfg(test)]
use crate::learning::eligibility::RunKind;
use crate::learning::eligibility::{Eligibility, RunObjective};
use crate::learning::retrieval::{RetrievalQuery, retrieve};
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::store::KnowledgeStore;
use crate::learning::telemetry::{
    DecisionSource, PlannedDecision, RecordedAction, RecordedRankedAction, proposal_event,
};
use crate::state::NormalizedState;
use serde_json::Value;

mod bootstrap;
mod capture;
mod critic;
mod finalize;
mod resume;
pub use bootstrap::bootstrap_session;
use capture::RunCapture;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionProvenance {
    pub locale: String,
    pub model_profile_sha256: String,
    pub compatibility_sha256: String,
    pub rules_sha256: String,
    pub synthetic_input: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMemory {
    pub situation: SituationDescriptor,
    pub context: Option<Value>,
    pub retrieved_memory_ids: Vec<String>,
    pub exposed_memory_ids: Vec<String>,
    pub knowledge_snapshot_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedPlan {
    pub action: AutoPlayAction,
    pub source: DecisionSource,
    pub selected_action_id: String,
    pub available_semantic_actions: Vec<SemanticAction>,
    pub ranked_suggestions: Vec<RecordedRankedAction>,
    pub memory_ids_used: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalizeSummary {
    pub eligibility: Eligibility,
    pub appended_cases: usize,
    pub skipped_cases: usize,
    pub snapshot_id: String,
}

impl FinalizeSummary {
    #[cfg(test)]
    pub fn run_kind(&self) -> RunKind {
        self.eligibility.run_kind
    }
}

pub struct LearningSession {
    config: MemoryConfig,
    store: KnowledgeStore,
    snapshot: KnowledgeSnapshot,
    provenance: SessionProvenance,
    capture: RunCapture,
    last_eligibility: Option<Eligibility>,
}

impl LearningSession {
    pub fn new(
        config: MemoryConfig,
        store: KnowledgeStore,
        snapshot: KnowledgeSnapshot,
        provenance: SessionProvenance,
    ) -> Self {
        Self {
            config,
            store,
            snapshot,
            provenance,
            capture: RunCapture::default(),
            last_eligibility: None,
        }
    }

    #[cfg(test)]
    pub fn snapshot(&self) -> &KnowledgeSnapshot {
        &self.snapshot
    }

    pub fn is_enabled(&self) -> bool {
        self.config.captures()
    }

    pub fn observe(&mut self, state: &NormalizedState) {
        if self.config.captures() {
            self.capture.observe(state);
        }
    }

    pub fn prepare(
        &self,
        state: &NormalizedState,
        ranker_tags: &[String],
    ) -> Option<PreparedMemory> {
        if !self.config.captures() {
            return None;
        }
        let situation = SituationDescriptor::from_state(
            state,
            self.capture.encounter_ids()?,
            RunObjective::Act3Victory,
            ranker_tags,
        )
        .ok()?;
        let mut retrieved_memory_ids = vec![];
        let mut exposed_memory_ids = vec![];
        let mut context = None;
        if self.config.retrieves()
            && let Some(seed) = state.seed
        {
            let compatibility = &self.provenance.compatibility_sha256;
            let result = retrieve(
                &self.snapshot,
                &RetrievalQuery {
                    situation: situation.clone(),
                    seed_hash: seed_hash(seed, compatibility),
                    compatibility_sha256: compatibility.clone(),
                    language: self.provenance.locale.clone(),
                },
                &self.config,
            );
            retrieved_memory_ids = result.items.iter().map(|item| item.id()).collect();
            if self.config.injects() {
                context = build_experience_context(&result, &self.provenance.locale, &self.config);
                exposed_memory_ids = context
                    .as_ref()
                    .and_then(|value| value.get("items"))
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|item| item.get("memory_id").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect();
            }
        }
        Some(PreparedMemory {
            situation,
            context,
            retrieved_memory_ids,
            exposed_memory_ids,
            knowledge_snapshot_id: self.snapshot.snapshot_id.clone(),
        })
    }

    pub fn record_executed(
        &mut self,
        run_id: &str,
        state: &NormalizedState,
        plan: &ExecutedPlan,
        prepared: &PreparedMemory,
    ) -> Option<Value> {
        let (Some(selected_action), Some(seed)) = (
            SemanticAction::from_execution(&plan.action, state),
            state.seed,
        ) else {
            return None;
        };
        let mut used: Vec<_> = plan
            .memory_ids_used
            .iter()
            .filter(|id| prepared.exposed_memory_ids.contains(id))
            .cloned()
            .collect();
        used.sort();
        used.dedup();
        let decision_id = self
            .capture
            .next_decision_id(run_id, state.floor, state.turn_number);
        let mut available_semantic_actions: Vec<_> = plan
            .available_semantic_actions
            .iter()
            .take(32)
            .cloned()
            .collect();
        if !available_semantic_actions.contains(&selected_action) {
            if available_semantic_actions.len() == 32 {
                available_semantic_actions.pop();
            }
            available_semantic_actions.push(selected_action.clone());
        }
        let ranked_suggestions: Vec<_> = plan.ranked_suggestions.iter().take(64).cloned().collect();
        let draft = CaseDraft {
            run_id: run_id.to_string(),
            decision_id: decision_id.clone(),
            seed_hash: seed_hash(seed, &self.provenance.compatibility_sha256),
            situation: prepared.situation.clone(),
            selected_action: selected_action.clone(),
            decision_source: plan.source,
            available_semantic_actions: available_semantic_actions.clone(),
            ranked_suggestions: ranked_suggestions.clone(),
            retrieved_memory_ids: prepared.exposed_memory_ids.clone(),
            memory_ids_used: used.clone(),
        };
        let decision = PlannedDecision {
            decision_id,
            selected_action_id: plan.selected_action_id.clone(),
            selected_semantic_action: selected_action,
            action: plan.action.clone(),
            source: plan.source,
            available_actions: available_semantic_actions
                .into_iter()
                .enumerate()
                .map(|(index, semantic_action)| RecordedAction {
                    action_id: format!("semantic:{index}"),
                    semantic_action,
                })
                .collect(),
            ranked_suggestions,
            retrieved_memory_ids: prepared.retrieved_memory_ids.clone(),
            memory_ids_used: used,
            knowledge_snapshot_id: Some(prepared.knowledge_snapshot_id.clone()),
        };
        let event = proposal_event(&decision, &state.observation_hash(), &prepared.situation).ok();
        self.capture.record(draft, state);
        event
    }

    pub fn discard_last_execution(&mut self) -> bool {
        self.capture.discard_last_execution()
    }
}
