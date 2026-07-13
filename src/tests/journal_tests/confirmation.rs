use super::*;

#[test]
fn not_confirmed_until_confirm_called() {
    let dir = tempfile::tempdir().unwrap();
    let journal = Journal::new(dir.path().to_path_buf());
    assert!(!journal.is_confirmed());
    assert!(journal.path().is_none());
}

#[test]
fn confirm_creates_readable_folder_name() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new(dir.path().to_path_buf());
    let config = crate::config::Config::from_env();
    let locale = test_locale();

    journal.confirm(42, "IRONCLAD", 20, &config, locale);
    assert!(journal.is_confirmed());

    let path = journal.path().unwrap();
    let folder_name = path
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let expected_class = locale.i18n.character_display_name("IRONCLAD");
    assert!(
        folder_name.contains(expected_class),
        "folder should contain class name '{expected_class}': {folder_name}"
    );
    assert!(
        folder_name.contains("_A20"),
        "folder should contain ascension: {folder_name}"
    );
}

#[test]
fn confirm_finds_existing_unfinished_run() {
    let dir = tempfile::tempdir().unwrap();
    let locale = test_locale();
    let seed = -3047511808784702860_i64;

    let mut existing = Journal::new_at(dir.path(), "existing-run");
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, locale);
    existing.log_state_change(&state.stable_hash(), &state);

    let config = crate::config::Config::from_env();
    let mut journal = Journal::new(dir.path().to_path_buf());
    journal.confirm(seed, "IRONCLAD", 20, &config, locale);

    assert!(journal.is_confirmed());
    assert!(journal.is_continued_run());
    assert_eq!(
        journal
            .path()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap(),
        "existing-run"
    );

    let events = read_events(journal.path().unwrap());
    let continued: Vec<_> = events
        .iter()
        .filter(|e| e["event"] == "run_continued")
        .collect();
    assert_eq!(continued.len(), 1);
    assert_eq!(continued[0]["seed"], seed);
    assert_eq!(continued[0]["character"], "IRONCLAD");
}

#[test]
fn confirm_skips_ended_run() {
    let dir = tempfile::tempdir().unwrap();
    let locale = test_locale();
    let seed = -3047511808784702860_i64;

    let mut existing = Journal::new_at(dir.path(), "ended-run");
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, locale);
    existing.log_state_change(&state.stable_hash(), &state);
    existing.log_run_ended("game_over");

    let config = crate::config::Config::from_env();
    let mut journal = Journal::new(dir.path().to_path_buf());
    journal.confirm(seed, "IRONCLAD", 20, &config, locale);

    assert!(!journal.is_continued_run());
    assert_ne!(
        journal
            .path()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap(),
        "ended-run"
    );
}

#[test]
fn confirm_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = Journal::new(dir.path().to_path_buf());
    let config = crate::config::Config::from_env();
    let locale = test_locale();

    journal.confirm(99, "DEFECT", 5, &config, locale);
    let path1 = journal.path().unwrap().to_path_buf();
    let count1 = read_events(&path1).len();

    journal.confirm(99, "DEFECT", 5, &config, locale);
    let path2 = journal.path().unwrap().to_path_buf();
    let count2 = read_events(&path2).len();

    assert_eq!(path1, path2);
    assert_eq!(count1, count2);
}

#[test]
fn run_continued_includes_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let locale = test_locale();
    let seed = -3047511808784702860_i64;

    let mut existing = Journal::new_at(dir.path(), "existing-run");
    let raw = load_fixture("combat-state.json");
    let state = crate::state::NormalizedState::from_raw(&raw, locale);
    existing.log_state_change(&state.stable_hash(), &state);

    let config = crate::config::Config::from_env();
    let mut journal = Journal::new(dir.path().to_path_buf());
    journal.confirm(seed, "IRONCLAD", 20, &config, locale);

    let events = read_events(journal.path().unwrap());
    let continued = events
        .iter()
        .find(|e| e["event"] == "run_continued")
        .unwrap();
    assert_eq!(continued["seed"], seed);
    assert_eq!(continued["character"], "IRONCLAD");
    assert_eq!(continued["ascension_level"], 20);
    assert!(!continued["app_version"].as_str().unwrap().is_empty());
}
