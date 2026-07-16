use super::{
    ActionPattern, Lesson, LessonBenchmark, LessonError, LessonScope, LessonTrigger, OutcomeCode,
    StrategicHypothesis,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub(super) fn family_key(lesson: &Lesson) -> Result<String, LessonError> {
    if let Some(strategy) = lesson.strategy.as_ref() {
        return hash(&StrategicFamilyIdentity {
            scope: &lesson.scope,
            strategy,
        });
    }
    hash(&FamilyIdentity {
        scope: &lesson.scope,
        trigger: &lesson.trigger,
        action_pattern: &lesson.action_pattern,
        outcome_code: lesson.outcome_code,
        outcome_predicate_version: lesson.outcome_predicate_version,
    })
}

pub(super) fn lesson_id(lesson: &Lesson) -> Result<String, LessonError> {
    if let (Some(strategy), Some(lifecycle)) = (lesson.strategy.as_ref(), lesson.lifecycle.as_ref())
    {
        return hash(&StrategicLessonIdentity {
            family_key: &lesson.family_key,
            language: &lesson.language,
            strategy,
            source_case_ids: &lesson.source_case_ids,
            critic_model_profile_sha256: &lesson.critic.model_profile_sha256,
            confidence_millis: lesson.critic.confidence_millis,
            benchmark: &lifecycle.benchmark,
            parent_lesson_id: lifecycle.parent_lesson_id.as_deref(),
            generation: lifecycle.generation,
        });
    }
    hash(&LessonIdentity {
        family_key: &lesson.family_key,
        language: &lesson.language,
        guidance_kind: lesson.guidance.kind,
        guidance_text: &lesson.guidance.text,
        rationale: &lesson.rationale,
        source_case_ids: &lesson.source_case_ids,
        critic_model_profile_sha256: &lesson.critic.model_profile_sha256,
        confidence_millis: lesson.critic.confidence_millis,
    })
}

fn hash(value: &impl Serialize) -> Result<String, LessonError> {
    let bytes = serde_json::to_vec(value).map_err(|_| LessonError::Serialization)?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

#[derive(Serialize)]
struct FamilyIdentity<'a> {
    scope: &'a LessonScope,
    trigger: &'a LessonTrigger,
    action_pattern: &'a ActionPattern,
    outcome_code: OutcomeCode,
    outcome_predicate_version: u32,
}

#[derive(Serialize)]
struct LessonIdentity<'a> {
    family_key: &'a str,
    language: &'a str,
    guidance_kind: super::GuidanceKind,
    guidance_text: &'a str,
    rationale: &'a str,
    source_case_ids: &'a [String],
    critic_model_profile_sha256: &'a str,
    confidence_millis: u16,
}

#[derive(Serialize)]
struct StrategicFamilyIdentity<'a> {
    scope: &'a LessonScope,
    strategy: &'a StrategicHypothesis,
}

#[derive(Serialize)]
struct StrategicLessonIdentity<'a> {
    family_key: &'a str,
    language: &'a str,
    strategy: &'a StrategicHypothesis,
    source_case_ids: &'a [String],
    critic_model_profile_sha256: &'a str,
    confidence_millis: u16,
    benchmark: &'a LessonBenchmark,
    parent_lesson_id: Option<&'a str>,
    generation: u32,
}
