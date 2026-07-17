use crate::autoplay::action::AutoPlayAction;
use crate::learning::action::SemanticAction;
use crate::learning::case::{CaseDraft, seed_hash};
use crate::learning::config::MemoryConfig;
use crate::learning::descriptor::SituationDescriptor;
use crate::learning::eligibility::Eligibility;
#[cfg(test)]
use crate::learning::eligibility::RunKind;
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::status::LearningStatus;
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
mod lifecycle;
mod prepare;
mod resume;
pub use bootstrap::bootstrap_session;
use capture::RunCapture;
pub(crate) use critic::{CriticDraft, CriticDraftRejection, CriticIngest};

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
    pub evaluated_lessons: usize,
    pub retired_lessons: usize,
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
    last_completed_benchmark: Option<crate::learning::lesson::LessonBenchmark>,
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
            last_completed_benchmark: None,
        }
    }

    #[cfg(test)]
    pub fn snapshot(&self) -> &KnowledgeSnapshot {
        &self.snapshot
    }

    pub fn is_enabled(&self) -> bool {
        self.config.captures()
    }

    pub fn status(&self) -> LearningStatus {
        LearningStatus {
            mode: self.config.mode,
            case_count: self.snapshot.cases.len(),
            lesson_count: self.snapshot.lessons.len(),
            last_run: self.last_eligibility.as_ref().map(Into::into),
        }
    }

    pub(crate) fn restore_last_run_eligibility(&mut self, eligibility: Option<Eligibility>) {
        self.last_eligibility = eligibility;
    }

    pub fn observe(&mut self, state: &NormalizedState) {
        if self.config.captures() {
            self.capture.observe(state);
        }
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
        let strategic_ids: Vec<_> = used
            .iter()
            .filter(|id| {
                self.snapshot
                    .lessons
                    .iter()
                    .any(|lesson| lesson.lesson_id == id.as_str() && lesson.is_strategic())
            })
            .cloned()
            .collect();
        if let Some(locked) = self.capture.trial_lesson_id().map(str::to_string) {
            used.retain(|id| !strategic_ids.contains(id) || id == &locked);
        } else if let Some(first) = strategic_ids.first() {
            self.capture.lock_trial_lesson(first.clone());
            used.retain(|id| !strategic_ids.contains(id) || id == first);
        }
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
            ascension_level: state.ascension_level,
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
