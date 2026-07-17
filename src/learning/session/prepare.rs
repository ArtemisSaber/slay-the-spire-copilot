use super::{LearningSession, PreparedMemory};
use crate::learning::case::seed_hash;
use crate::learning::context::build_experience_context;
use crate::learning::descriptor::SituationDescriptor;
use crate::learning::eligibility::RunObjective;
use crate::learning::retrieval::{RetrievalItem, RetrievalQuery, retrieve};
use crate::state::NormalizedState;
use serde_json::Value;

impl LearningSession {
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
            let mut result = retrieve(
                &self.snapshot,
                &RetrievalQuery {
                    situation: situation.clone(),
                    ascension_level: state.ascension_level,
                    seed_hash: seed_hash(seed, compatibility),
                    compatibility_sha256: compatibility.clone(),
                },
                &self.config,
            );
            if let Some(locked) = self.capture.trial_lesson_id() {
                result.items.retain(|item| {
                    !matches!(item, RetrievalItem::Lesson { .. }) || item.id() == locked
                });
            }
            retrieved_memory_ids = result.items.iter().map(RetrievalItem::id).collect();
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
}
