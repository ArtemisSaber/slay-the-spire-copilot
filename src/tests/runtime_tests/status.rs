use super::*;

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
