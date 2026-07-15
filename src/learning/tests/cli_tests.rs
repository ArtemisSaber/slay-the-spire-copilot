use super::support::{decision_case, lesson_for};
use crate::learning::bundle::embedded_bundle;
use crate::learning::cli::execute_command;
use crate::learning::lesson::{LessonEvent, LessonEventKind, LessonStatus};
use crate::learning::store::KnowledgeStore;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn status_and_export_operate_on_verified_repository_plus_local_knowledge() {
    let temp = tempfile::tempdir().unwrap();
    let store = KnowledgeStore::new(temp.path().join("learning/knowledge"));
    store.rebuild(&[embedded_bundle().unwrap()]).unwrap();

    let status = execute_command(temp.path(), &args(&["status"])).unwrap();
    let status: serde_json::Value = serde_json::from_str(&status).unwrap();
    assert_eq!(status["verified"], true);
    assert_eq!(status["case_count"], 0);

    let output = temp.path().join("export.json");
    execute_command(
        temp.path(),
        &args(&["export-bundle", output.to_str().unwrap()]),
    )
    .unwrap();
    assert!(output.is_file());
}

#[test]
fn verify_checks_append_only_sources_even_when_a_snapshot_already_exists() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("learning/knowledge");
    let store = KnowledgeStore::new(&root);
    store.rebuild(&[embedded_bundle().unwrap()]).unwrap();
    std::fs::write(root.join("cases.jsonl"), "not-json\n").unwrap();

    assert!(execute_command(temp.path(), &args(&["status"])).is_ok());
    assert!(execute_command(temp.path(), &args(&["verify"])).is_err());
}

#[test]
fn human_contest_is_an_append_only_lesson_transition() {
    let temp = tempfile::tempdir().unwrap();
    let store = KnowledgeStore::new(temp.path().join("learning/knowledge"));
    let case = decision_case("run-a", 1);
    let lesson = lesson_for(&case);
    let lesson_id = lesson.lesson_id.clone();
    store.append_cases(&[case]).unwrap();
    store
        .append_lesson_events(&[
            LessonEvent::new(LessonEventKind::Proposed, lesson, None, 1).unwrap()
        ])
        .unwrap();
    store.rebuild(&[embedded_bundle().unwrap()]).unwrap();

    execute_command(
        temp.path(),
        &args(&[
            "contest",
            &lesson_id,
            "--reason",
            "Reviewed conflicting evidence",
        ]),
    )
    .unwrap();
    let snapshot = store.load_snapshot().unwrap();

    assert_eq!(snapshot.lessons[0].status, LessonStatus::Contested);
    let lines = std::fs::read_to_string(temp.path().join("learning/knowledge/lessons.jsonl"))
        .unwrap()
        .lines()
        .count();
    assert_eq!(lines, 2);
}

#[test]
fn bundle_export_rejects_paths_outside_the_project_root() {
    let temp = tempfile::tempdir().unwrap();
    let outside = temp.path().parent().unwrap().join("outside-bundle.json");

    let error = execute_command(
        temp.path(),
        &args(&["export-bundle", outside.to_str().unwrap()]),
    )
    .unwrap_err();

    assert!(error.to_string().contains("project root"));
}
