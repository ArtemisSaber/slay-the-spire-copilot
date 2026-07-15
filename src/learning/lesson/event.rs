use super::{Lesson, LessonError, LessonStatus};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonEventKind {
    Proposed,
    Recalculated,
    Validated,
    Contested,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LessonEvent {
    pub schema_version: u32,
    pub event_id: String,
    pub ts_ms: u128,
    pub event: LessonEventKind,
    pub lesson: Lesson,
    pub reason: Option<String>,
}

impl LessonEvent {
    pub fn new(
        event: LessonEventKind,
        lesson: Lesson,
        reason: Option<String>,
        ts_ms: u128,
    ) -> Result<Self, LessonError> {
        validate_transition(event, &lesson, reason.as_deref())?;
        let mut value = Self {
            schema_version: 1,
            event_id: String::new(),
            ts_ms,
            event,
            lesson,
            reason,
        };
        value.event_id = value.compute_id()?;
        Ok(value)
    }

    pub fn verify(&self) -> bool {
        self.schema_version == 1
            && self.lesson.verify_identity()
            && validate_transition(self.event, &self.lesson, self.reason.as_deref()).is_ok()
            && self.compute_id().is_ok_and(|id| id == self.event_id)
    }

    fn compute_id(&self) -> Result<String, LessonError> {
        let identity = EventIdentity {
            schema_version: self.schema_version,
            ts_ms: self.ts_ms,
            event: self.event,
            lesson: &self.lesson,
            reason: self.reason.as_deref(),
        };
        let bytes = serde_json::to_vec(&identity).map_err(|_| LessonError::Serialization)?;
        Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
    }
}

fn validate_transition(
    event: LessonEventKind,
    lesson: &Lesson,
    reason: Option<&str>,
) -> Result<(), LessonError> {
    let expected = match event {
        LessonEventKind::Validated => Some(LessonStatus::Validated),
        LessonEventKind::Contested => Some(LessonStatus::Contested),
        LessonEventKind::Retired => Some(LessonStatus::Retired),
        LessonEventKind::Proposed => Some(LessonStatus::Proposed),
        LessonEventKind::Recalculated => None,
    };
    if expected.is_some_and(|status| lesson.status != status) {
        return Err(LessonError::InvalidHumanStatus);
    }
    let human = matches!(
        event,
        LessonEventKind::Validated | LessonEventKind::Contested | LessonEventKind::Retired
    );
    if human
        && reason.is_none_or(|reason| {
            reason.trim().is_empty() || reason.len() > 512 || reason.chars().any(char::is_control)
        })
    {
        return Err(LessonError::UnsafeText);
    }
    if !human && reason.is_some() {
        return Err(LessonError::UnsafeText);
    }
    Ok(())
}

#[derive(Serialize)]
struct EventIdentity<'a> {
    schema_version: u32,
    ts_ms: u128,
    event: LessonEventKind,
    lesson: &'a Lesson,
    reason: Option<&'a str>,
}
