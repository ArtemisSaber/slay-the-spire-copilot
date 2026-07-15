use super::*;

#[test]
fn has_monsters_detects_active() {
    assert!(has_monsters(&with_monsters("NONE")));
}

#[test]
fn has_monsters_ignores_gone() {
    let state = make_state("NONE", Some(vec![gone_monster()]));
    assert!(!has_monsters(&state));
}

#[test]
fn has_monsters_false_when_no_combat_state() {
    assert!(!has_monsters(&no_combat_state("NONE")));
}

#[test]
fn game_over_screen_ends_run() {
    let state = make_state("GAME_OVER", None);

    assert!(is_game_over_state(&state));
    assert!(should_end_run(&state, true));
    assert_eq!(run_end_reason(&state, true), Some("game_over"));
}

#[test]
fn leaving_game_after_observed_state_ends_run() {
    let state = menu_state();

    assert!(should_end_run(&state, true));
    assert_eq!(run_end_reason(&state, true), Some("left_game"));
}

#[test]
fn menu_before_any_observed_state_does_not_end_run() {
    let state = menu_state();

    assert!(!should_end_run(&state, false));
    assert_eq!(run_end_reason(&state, false), None);
}

#[test]
fn startup_check_enabled_by_default() {
    let opts = runtime_options_from([], None);
    assert!(!opts.skip_startup_check);
    assert!(!opts.force_mock_provider);
    assert!(!opts.setup_only);
}

#[test]
fn startup_check_skipped_by_no_startup_check_flag() {
    let opts = runtime_options_from(["--no-startup-check"], None);
    assert!(opts.skip_startup_check);
    assert!(!opts.force_mock_provider);
}

#[test]
fn startup_check_skipped_by_env_var() {
    let opts = runtime_options_from([], Some("1"));
    assert!(opts.skip_startup_check);
}

#[test]
fn stdin_test_mode_uses_mock_provider_by_default() {
    let opts = runtime_options_from(["--stdin-test"], None);
    assert!(opts.skip_startup_check);
    assert!(opts.force_mock_provider);
    assert!(!opts.setup_only);
}

#[test]
fn setup_mode_runs_without_startup_check() {
    let opts = runtime_options_from(["setup"], None);
    assert!(opts.skip_startup_check);
    assert!(opts.setup_only);
    assert!(!opts.force_mock_provider);
}

#[test]
fn configure_alias_runs_setup_mode() {
    let opts = runtime_options_from(["configure"], None);
    assert!(opts.skip_startup_check);
    assert!(opts.setup_only);
}

#[test]
fn learning_alias_opens_the_user_facing_status_command() {
    let opts = runtime_options_from(["learning", "status"], None);

    assert_eq!(opts.knowledge_args, Some(vec!["status".to_string()]));
    assert!(opts.skip_startup_check);
}

#[test]
fn postmortem_mode_uses_ai_by_default() {
    let opts = runtime_options_from(["postmortem", "runs/test/events.jsonl"], None);
    assert_eq!(
        opts.postmortem_path.as_deref(),
        Some("runs/test/events.jsonl")
    );
    assert!(!opts.postmortem_plain);
}

#[test]
fn postmortem_plain_flag_disables_ai_rewrite() {
    let opts = runtime_options_from(["postmortem", "--plain", "runs/test/events.jsonl"], None);
    assert_eq!(
        opts.postmortem_path.as_deref(),
        Some("runs/test/events.jsonl")
    );
    assert!(opts.postmortem_plain);
}
