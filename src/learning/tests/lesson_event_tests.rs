use super::support::{decision_case, lesson_for};
use crate::learning::lesson::{LessonEvent, LessonEventKind, LessonStatus};

#[test]
fn lesson_events_are_append_only_verifiable_state_transitions() {
    let case = decision_case("run-a", 1);
    let lesson = lesson_for(&case);
    let proposed = LessonEvent::new(LessonEventKind::Proposed, lesson.clone(), None, 10).unwrap();

    assert!(proposed.verify());
    assert!(proposed.event_id.starts_with("sha256:"));

    let mut validated_lesson = lesson;
    validated_lesson
        .set_human_status(LessonStatus::Validated)
        .unwrap();
    let validated = LessonEvent::new(
        LessonEventKind::Validated,
        validated_lesson,
        Some("Reviewed against game mechanics".into()),
        20,
    )
    .unwrap();
    assert!(validated.verify());
    assert_ne!(proposed.event_id, validated.event_id);
}

#[test]
fn event_verification_detects_mutation() {
    let case = decision_case("run-a", 1);
    let mut event =
        LessonEvent::new(LessonEventKind::Proposed, lesson_for(&case), None, 10).unwrap();
    event.lesson.guidance.text.push_str(" changed");

    assert!(!event.verify());
}

#[test]
fn human_transition_requires_matching_status_and_reason() {
    let case = decision_case("run-a", 1);
    let lesson = lesson_for(&case);

    assert!(LessonEvent::new(LessonEventKind::Validated, lesson.clone(), None, 10).is_err());
    assert!(
        LessonEvent::new(LessonEventKind::Retired, lesson, Some("reason".into()), 10,).is_err()
    );
}
