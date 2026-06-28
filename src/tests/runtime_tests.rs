use crate::runtime::{
    RuntimeOptions, has_monsters, is_error, is_game_over_state, run_end_reason,
    runtime_options_from, should_end_run,
};
use serde_json::json;

// === runtime_options_from ===

#[test]
fn unknown_args_are_ignored() {
    let opts = runtime_options_from(["--unknown-flag", "random-arg"], None);
    assert_eq!(
        opts,
        RuntimeOptions {
            skip_startup_check: false,
            force_mock_provider: false,
            setup_only: false,
            postmortem_path: None,
            postmortem_plain: false,
        }
    );
}

#[test]
fn skip_comm_config_env_value_true() {
    let opts = runtime_options_from([], Some("true"));
    assert!(opts.skip_startup_check);
}

#[test]
fn skip_comm_config_env_value_yes() {
    let opts = runtime_options_from([], Some("yes"));
    assert!(opts.skip_startup_check);
}

#[test]
fn skip_comm_config_env_non_truthy_is_ignored() {
    let opts = runtime_options_from([], Some("0"));
    assert!(!opts.skip_startup_check);
    let opts = runtime_options_from([], Some("no"));
    assert!(!opts.skip_startup_check);
    let opts = runtime_options_from([], Some(""));
    assert!(!opts.skip_startup_check);
}

#[test]
fn skip_comm_config_env_does_not_set_other_fields() {
    let opts = runtime_options_from([], Some("true"));
    assert!(!opts.force_mock_provider);
    assert!(!opts.setup_only);
    assert!(opts.postmortem_path.is_none());
    assert!(!opts.postmortem_plain);
}

#[test]
fn stdin_test_combined_with_env_var() {
    let opts = runtime_options_from(["--stdin-test"], Some("1"));
    assert!(opts.skip_startup_check);
    assert!(opts.force_mock_provider);
}

#[test]
fn postmortem_plain_after_path_is_ignored() {
    let opts = runtime_options_from(["postmortem", "some/path.jsonl", "--plain"], None);
    assert_eq!(opts.postmortem_path.as_deref(), Some("some/path.jsonl"));
    assert!(!opts.postmortem_plain);
}

#[test]
fn postmortem_with_no_following_args() {
    let opts = runtime_options_from(["postmortem"], None);
    assert!(opts.postmortem_path.is_none());
    assert!(!opts.postmortem_plain);
}

#[test]
fn postmortem_only_plain_flag_no_path() {
    let opts = runtime_options_from(["postmortem", "--plain"], None);
    assert!(opts.postmortem_plain);
    assert!(opts.postmortem_path.is_none());
}

// === RuntimeOptions struct ===

#[test]
fn runtime_options_field_access() {
    let opts = RuntimeOptions {
        skip_startup_check: true,
        force_mock_provider: true,
        setup_only: true,
        postmortem_path: Some("test.jsonl".to_string()),
        postmortem_plain: true,
    };
    assert!(opts.skip_startup_check);
    assert!(opts.force_mock_provider);
    assert!(opts.setup_only);
    assert_eq!(opts.postmortem_path.as_deref(), Some("test.jsonl"));
    assert!(opts.postmortem_plain);
}

#[test]
fn runtime_options_clone_is_equal() {
    let opts = RuntimeOptions {
        skip_startup_check: true,
        force_mock_provider: false,
        setup_only: false,
        postmortem_path: None,
        postmortem_plain: false,
    };
    assert_eq!(opts, opts.clone());
}

// === is_error ===

#[test]
fn is_error_detects_error_key_with_message() {
    assert!(is_error(&json!({"error": "something went wrong"})));
}

#[test]
fn is_error_detects_null_error() {
    assert!(is_error(&json!({"error": null})));
}

#[test]
fn is_error_detects_empty_string_error() {
    assert!(is_error(&json!({"error": ""})));
}

#[test]
fn is_error_false_without_error_key() {
    assert!(!is_error(&json!({"in_game": true})));
}

#[test]
fn is_error_false_with_other_keys() {
    assert!(!is_error(&json!({"game_state": {"screen_type": "NONE"}})));
}

#[test]
fn is_error_false_on_empty_object() {
    assert!(!is_error(&json!({})));
}

// === is_game_over_state ===

#[test]
fn game_over_state_different_screen_type() {
    let state = json!({"game_state": {"screen_type": "NONE"}});
    assert!(!is_game_over_state(&state));
}

#[test]
fn game_over_state_missing_screen_type() {
    let state = json!({"game_state": {"floor": 1}});
    assert!(!is_game_over_state(&state));
}

#[test]
fn game_over_state_no_game_state_key() {
    let state = json!({"in_game": true});
    assert!(!is_game_over_state(&state));
}

#[test]
fn game_over_state_null_screen_type() {
    let state = json!({"game_state": {"screen_type": null}});
    assert!(!is_game_over_state(&state));
}

#[test]
fn game_over_state_empty_object() {
    assert!(!is_game_over_state(&json!({})));
}

// === should_end_run ===

#[test]
fn should_end_run_false_when_in_game_no_game_state() {
    let state = json!({"in_game": true});
    assert!(!should_end_run(&state, true));
    assert!(!should_end_run(&state, false));
}

#[test]
fn should_end_run_false_playing_with_seen_state() {
    let state = json!({"in_game": true, "game_state": {"screen_type": "NONE"}});
    assert!(!should_end_run(&state, true));
}

#[test]
fn should_end_run_false_not_in_game_never_seen_state() {
    let state = json!({"in_game": false});
    assert!(!should_end_run(&state, false));
}

#[test]
fn should_end_run_true_not_in_game_has_seen_state() {
    let state = json!({"in_game": false});
    assert!(should_end_run(&state, true));
}

// === run_end_reason ===

#[test]
fn run_end_reason_none_while_playing() {
    let state = json!({"in_game": true, "game_state": {"screen_type": "NONE"}});
    assert_eq!(run_end_reason(&state, true), None);
    assert_eq!(run_end_reason(&state, false), None);
}

#[test]
fn run_end_reason_none_not_in_game_never_seen() {
    let state = json!({"in_game": false});
    assert_eq!(run_end_reason(&state, false), None);
}

#[test]
fn run_end_reason_left_game_when_seen() {
    let state = json!({"in_game": false});
    assert_eq!(run_end_reason(&state, true), Some("left_game"));
}

// === has_monsters ===

#[test]
fn has_monsters_mixed_active_and_gone() {
    let state = json!({
        "game_state": {
            "combat_state": {
                "monsters": [
                    {"id": "a", "is_gone": true},
                    {"id": "b", "is_gone": false},
                    {"id": "c", "is_gone": true}
                ]
            }
        }
    });
    assert!(has_monsters(&state));
}

#[test]
fn has_monsters_missing_is_gone_field_treated_as_active() {
    let state = json!({
        "game_state": {
            "combat_state": {
                "monsters": [
                    {"id": "a"},
                    {"id": "b", "is_gone": true}
                ]
            }
        }
    });
    assert!(has_monsters(&state));
}

#[test]
fn has_monsters_all_missing_is_gone_all_active() {
    let state = json!({
        "game_state": {
            "combat_state": {
                "monsters": [
                    {"id": "a"},
                    {"id": "b"}
                ]
            }
        }
    });
    assert!(has_monsters(&state));
}

#[test]
fn has_monsters_combat_state_without_monsters_key() {
    let state = json!({
        "game_state": {
            "combat_state": {"turn": 1}
        }
    });
    assert!(!has_monsters(&state));
}

#[test]
fn has_monsters_monsters_not_an_array() {
    let state = json!({
        "game_state": {
            "combat_state": {
                "monsters": "not-an-array"
            }
        }
    });
    assert!(!has_monsters(&state));
}

#[test]
fn has_monsters_null_monsters() {
    let state = json!({
        "game_state": {
            "combat_state": {
                "monsters": null
            }
        }
    });
    assert!(!has_monsters(&state));
}

#[test]
fn has_monsters_no_combat_state_key() {
    let state = json!({"game_state": {"screen_type": "NONE"}});
    assert!(!has_monsters(&state));
}

#[test]
fn has_monsters_empty_object() {
    assert!(!has_monsters(&json!({})));
}
