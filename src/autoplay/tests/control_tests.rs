use super::*;
use std::fs;

#[test]
fn missing_control_defaults_to_auto() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = load_control(&dir.path().join("autoplay-control.json"), None);

    assert_eq!(
        loaded,
        ControlLoad::MissingDefault(AutoPlayControl::default_enabled())
    );
}

#[test]
fn valid_control_loads_when_revision_is_new() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("autoplay-control.json");
    fs::write(
        &path,
        r#"{
                "schema_version": 1,
                "revision": 3,
                "mode": "paused",
                "updated_at_ms": 1000,
                "require_confirmation": true,
                "allow_card_rewards": true,
                "allow_combat_rewards": true,
                "allow_boss_rewards": true,
                "allow_rest": true,
                "allow_events": false,
                "allow_map": false,
                "allow_shop": false,
                "allow_combat": false,
                "allow_selection_screens": false,
                "min_hp_percent": 20
            }"#,
    )
    .unwrap();

    let loaded = load_control(&path, Some(2));
    let ControlLoad::Updated(control) = loaded else {
        panic!("expected updated control");
    };

    assert_eq!(control.revision, 3);
    assert_eq!(control.mode, AutoPlayMode::Paused);
    assert!(control.require_confirmation);
    assert!(!control.allow_events);
    assert_eq!(control.min_hp_percent, Some(20));
}

#[test]
fn unchanged_control_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("autoplay-control.json");
    fs::write(
        &path,
        r#"{
                "schema_version": 1,
                "revision": 3,
                "mode": "auto",
                "updated_at_ms": 1000,
                "require_confirmation": false,
                "allow_card_rewards": true,
                "allow_combat_rewards": true,
                "allow_boss_rewards": true,
                "allow_rest": true,
                "allow_events": true,
                "allow_map": false,
                "allow_shop": false,
                "allow_combat": false,
                "allow_selection_screens": false,
                "min_hp_percent": null
            }"#,
    )
    .unwrap();

    assert_eq!(load_control(&path, Some(3)), ControlLoad::Stale);
}

#[test]
fn lower_revision_still_counts_as_changed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("autoplay-control.json");
    fs::write(
        &path,
        r#"{
                "schema_version": 1,
                "revision": 1,
                "mode": "paused",
                "updated_at_ms": 1000,
                "require_confirmation": false,
                "allow_card_rewards": true,
                "allow_combat_rewards": true,
                "allow_boss_rewards": true,
                "allow_rest": true,
                "allow_events": true,
                "allow_map": false,
                "allow_shop": false,
                "allow_combat": false,
                "allow_selection_screens": false,
                "min_hp_percent": null
            }"#,
    )
    .unwrap();

    let loaded = load_control(&path, Some(10));
    let ControlLoad::Updated(control) = loaded else {
        panic!("expected lower but changed revision to load");
    };

    assert_eq!(control.revision, 1);
    assert_eq!(control.mode, AutoPlayMode::Paused);
}

#[test]
fn malformed_control_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("autoplay-control.json");
    fs::write(&path, "{not-json").unwrap();

    assert!(matches!(
        load_control(&path, None),
        ControlLoad::Malformed(_)
    ));
}

#[test]
fn minimal_json_defaults_allow_flags_to_true() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("autoplay-control.json");
    fs::write(
        &path,
        r#"{"schema_version": 1, "revision": 1, "mode": "auto"}"#,
    )
    .unwrap();

    let loaded = load_control(&path, None);
    let ControlLoad::Updated(control) = loaded else {
        panic!("expected updated control");
    };

    assert_eq!(control.mode, AutoPlayMode::Auto);
    assert!(control.allow_card_rewards);
    assert!(control.allow_combat_rewards);
    assert!(control.allow_boss_rewards);
    assert!(control.allow_rest);
    assert!(control.allow_events);
    assert!(control.allow_map);
    assert!(control.allow_shop);
    assert!(control.allow_combat);
    assert!(control.allow_selection_screens);
    assert!(!control.require_confirmation);
}
