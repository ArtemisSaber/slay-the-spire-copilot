use crate::config::Config;
use crate::learning::bundle::embedded_bundle;
use crate::learning::session::bootstrap_session;
use std::collections::HashMap;

#[test]
fn startup_rebuilds_one_verified_snapshot_with_the_repository_bundle() {
    let temp = tempfile::tempdir().unwrap();
    let config = Config::from_map(&HashMap::from([("MEMORY_MODE", "collect")]));

    let session = bootstrap_session(temp.path(), &config, "en", false).unwrap();
    let bundle = embedded_bundle().unwrap();

    assert!(session.snapshot().verify());
    assert_eq!(session.snapshot().bundle_ids, vec![bundle.bundle_id]);
    assert!(
        temp.path()
            .join("learning/knowledge/manifest.json")
            .exists()
    );
}

#[test]
fn disabled_memory_loads_the_bundle_in_memory_without_writing_runtime_state() {
    let temp = tempfile::tempdir().unwrap();
    let config = Config::from_map(&HashMap::new());

    let session = bootstrap_session(temp.path(), &config, "en", true).unwrap();

    assert!(session.snapshot().verify());
    assert!(!temp.path().join("learning").exists());
}

#[test]
fn corrupt_local_knowledge_fails_open_to_the_verified_repository_bundle() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("learning/knowledge");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("cases.jsonl"), "not-json\n").unwrap();
    let config = Config::from_map(&HashMap::from([("MEMORY_MODE", "on")]));

    let session = bootstrap_session(temp.path(), &config, "en", false).unwrap();

    assert!(session.snapshot().verify());
    assert!(session.snapshot().cases.is_empty());
    assert_eq!(session.snapshot().bundle_ids.len(), 1);
}

#[test]
fn unavailable_pinned_snapshot_disables_memory_for_the_continued_run() {
    let temp = tempfile::tempdir().unwrap();
    let config = Config::from_map(&HashMap::from([("MEMORY_MODE", "on")]));
    let mut session = bootstrap_session(temp.path(), &config, "en", false).unwrap();
    let journal = temp.path().join("events.jsonl");
    std::fs::write(
        &journal,
        format!(
            "{{\"event\":\"autoplay_decision_proposed\",\"knowledge_snapshot_id\":\"sha256:{}\"}}\n",
            "0".repeat(64)
        ),
    )
    .unwrap();

    assert!(session.pin_snapshot_from_journal(&journal).is_err());
    assert!(!session.is_enabled());
}
