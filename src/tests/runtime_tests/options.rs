use super::*;

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
            knowledge_args: None,
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

#[test]
fn runtime_options_field_access() {
    let opts = RuntimeOptions {
        skip_startup_check: true,
        force_mock_provider: true,
        setup_only: true,
        postmortem_path: Some("test.jsonl".to_string()),
        postmortem_plain: true,
        knowledge_args: None,
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
        knowledge_args: None,
    };
    assert_eq!(opts, opts.clone());
}

#[test]
fn knowledge_mode_preserves_subcommand_arguments() {
    let opts = runtime_options_from(
        ["knowledge", "contest", "lesson:1", "reviewed evidence"],
        None,
    );

    assert_eq!(
        opts.knowledge_args,
        Some(vec![
            "contest".to_string(),
            "lesson:1".to_string(),
            "reviewed evidence".to_string(),
        ])
    );
    assert!(opts.skip_startup_check);
}
