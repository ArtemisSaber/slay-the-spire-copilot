use super::support::{decision_case, lesson_for};
use crate::learning::bundle::KnowledgeBundle;
use crate::learning::lesson::LessonStatus;
use crate::learning::snapshot::{KnowledgeSnapshot, SnapshotError};

#[test]
fn snapshot_merges_local_and_bundled_cases_deterministically() {
    let a = decision_case("run-a", 1);
    let b = decision_case("run-b", 2);
    let bundle = KnowledgeBundle::from_cases(vec![b.clone()]).unwrap();

    let first = KnowledgeSnapshot::build(vec![a.clone(), b.clone()], std::slice::from_ref(&bundle))
        .unwrap();
    let second = KnowledgeSnapshot::build(vec![b, a], &[bundle]).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.cases.len(), 2);
    assert!(first.verify());
    assert!(first.snapshot_id.starts_with("sha256:"));
}

#[test]
fn snapshot_rejects_a_case_with_a_forged_identity() {
    let mut forged = decision_case("run-a", 1);
    forged.case_id = "sha256:forged".into();

    assert!(KnowledgeSnapshot::build(vec![forged], &[]).is_err());
}

#[test]
fn snapshot_rejects_a_lesson_without_its_source_case() {
    let case = decision_case("run-a", 1);
    let lesson = lesson_for(&case);

    assert_eq!(
        KnowledgeSnapshot::build_with_lessons(vec![], vec![lesson], &[]),
        Err(SnapshotError::MissingLessonSource)
    );
}

#[test]
fn snapshot_rejects_conflicting_states_for_one_lesson_across_bundles() {
    let case = decision_case("run-a", 1);
    let proposed = lesson_for(&case);
    let mut validated = proposed.clone();
    validated.set_human_status(LessonStatus::Validated).unwrap();
    let first = KnowledgeBundle::from_knowledge(vec![case.clone()], vec![proposed]).unwrap();
    let second = KnowledgeBundle::from_knowledge(vec![case], vec![validated]).unwrap();

    assert_eq!(
        KnowledgeSnapshot::build(vec![], &[first, second]),
        Err(SnapshotError::ConflictingLesson)
    );
}

#[test]
fn snapshot_id_changes_when_knowledge_changes() {
    let a = KnowledgeSnapshot::build(vec![decision_case("run-a", 1)], &[]).unwrap();
    let b = KnowledgeSnapshot::build(
        vec![decision_case("run-a", 1), decision_case("run-b", 2)],
        &[],
    )
    .unwrap();

    assert_ne!(a.snapshot_id, b.snapshot_id);
}

#[test]
fn snapshot_contains_the_latest_verified_lesson_state() {
    let case = decision_case("run-a", 1);
    let lesson = lesson_for(&case);
    let snapshot =
        KnowledgeSnapshot::build_with_lessons(vec![case], vec![lesson.clone()], &[]).unwrap();

    assert_eq!(snapshot.lessons, vec![lesson]);
    assert!(snapshot.verify());
}

#[test]
fn snapshot_recalculates_lesson_support_from_all_merged_cases() {
    let cases: Vec<_> = (1..=5)
        .map(|seed| decision_case(&format!("run-{seed}"), seed))
        .collect();
    let lesson = lesson_for(&cases[0]);

    let snapshot = KnowledgeSnapshot::build_with_lessons(cases, vec![lesson], &[]).unwrap();

    assert_eq!(snapshot.lessons[0].status, LessonStatus::Supported);
    assert_eq!(snapshot.lessons[0].support.distinct_independent_seeds, 5);
}
