use super::*;

#[test]
fn log_includes_prompt_and_response() {
    let dir = tempfile::tempdir().unwrap();
    let prompt = "test-prompt-🦀🤣🦖";
    let response = "test-response-吃葡萄不吐葡萄皮";
    log_prompt_to(dir.path(), prompt, response);

    let log_path = dir.path().join("logs").join("prompts.log");
    let contents = std::fs::read_to_string(&log_path).unwrap();
    assert!(contents.contains(prompt));
    assert!(contents.contains(response));
    assert!(contents.contains("[system]"));
    assert!(contents.contains("[user]"));
    assert!(contents.contains("[assistant]"));
}

#[test]
fn log_prompt_into_dir_writes_formatted_entry() {
    let dir = tempfile::tempdir().unwrap();
    log_prompt_into_dir(dir.path(), "sys-content", "usr-content", "ast-content");
    let log_path = dir.path().join("logs").join("prompts.log");
    let contents = std::fs::read_to_string(&log_path).unwrap();
    assert!(contents.contains("[system]\nsys-content"));
    assert!(contents.contains("[user]\nusr-content"));
    assert!(contents.contains("[assistant]\nast-content"));
    assert!(contents.contains("\n---\n"));
}

#[test]
fn prompt_logging_disabled_returns_true_for_disabled_values() {
    assert!(prompt_logging_disabled(Some("false".into())));
    assert!(prompt_logging_disabled(Some("0".into())));
    assert!(prompt_logging_disabled(Some("no".into())));
    assert!(prompt_logging_disabled(Some("off".into())));
    assert!(prompt_logging_disabled(Some("FALSE".into())));
}

#[test]
fn prompt_logging_disabled_returns_false_for_enabled_or_unset() {
    assert!(!prompt_logging_disabled(Some("true".into())));
    assert!(!prompt_logging_disabled(Some("1".into())));
    assert!(!prompt_logging_disabled(Some("yes".into())));
    assert!(!prompt_logging_disabled(None));
}
