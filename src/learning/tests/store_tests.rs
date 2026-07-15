use super::support::{decision_case, lesson_for};
use crate::learning::bundle::{KnowledgeBundle, embedded_bundle};
use crate::learning::lesson::{LessonEvent, LessonEventKind};
use crate::learning::store::KnowledgeStore;
use std::fs;

#[test]
fn append_is_idempotent_and_rebuild_publishes_loadable_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let store = KnowledgeStore::new(temp.path());
    let case = decision_case("run-a", 1);

    let first = store.append_cases(std::slice::from_ref(&case)).unwrap();
    let second = store.append_cases(std::slice::from_ref(&case)).unwrap();
    assert_eq!((first.appended, first.skipped), (1, 0));
    assert_eq!((second.appended, second.skipped), (0, 1));

    let snapshot = store.rebuild(&[embedded_bundle().unwrap()]).unwrap();
    assert_eq!(snapshot.cases, vec![case]);
    assert_eq!(store.load_snapshot().unwrap(), snapshot);
    assert!(temp.path().join("manifest.json").is_file());
    assert!(
        temp.path()
            .join("snapshots")
            .join(snapshot.file_name())
            .is_file()
    );
}

#[test]
fn corrupt_source_fails_without_replacing_last_good_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let store = KnowledgeStore::new(temp.path());
    store.append_cases(&[decision_case("run-a", 1)]).unwrap();
    let good = store.rebuild(&[]).unwrap();

    fs::write(temp.path().join("cases.jsonl"), "not-json\n").unwrap();
    assert!(store.rebuild(&[]).is_err());
    assert_eq!(store.load_snapshot().unwrap(), good);
}

#[test]
fn export_bundle_creates_a_repository_ready_verified_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let store = KnowledgeStore::new(temp.path().join("knowledge"));
    store.append_cases(&[decision_case("run-a", 1)]).unwrap();
    let snapshot = store.rebuild(&[]).unwrap();
    let output = temp.path().join("bundled-v1.json");

    store.export_bundle(&snapshot, &output).unwrap();
    let content = fs::read_to_string(output).unwrap();
    let bundle = KnowledgeBundle::from_json(&content).unwrap();
    assert!(bundle.verify());
    assert_eq!(bundle.cases, snapshot.cases);
    assert!(!content.contains("profile-salt"));
}

#[test]
fn rebuild_folds_lesson_events_to_latest_state() {
    let temp = tempfile::tempdir().unwrap();
    let store = KnowledgeStore::new(temp.path());
    let case = decision_case("run-a", 1);
    let lesson = lesson_for(&case);
    store.append_cases(std::slice::from_ref(&case)).unwrap();
    let proposed = LessonEvent::new(LessonEventKind::Proposed, lesson.clone(), None, 1).unwrap();
    store
        .append_lesson_events(std::slice::from_ref(&proposed))
        .unwrap();
    store
        .append_lesson_events(std::slice::from_ref(&proposed))
        .unwrap();

    let snapshot = store.rebuild(&[]).unwrap();
    assert_eq!(snapshot.lessons, vec![lesson]);
    let lines = fs::read_to_string(temp.path().join("lessons.jsonl")).unwrap();
    assert_eq!(lines.lines().count(), 1);
}
