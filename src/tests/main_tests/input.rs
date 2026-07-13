#[test]
fn max_stdin_json_bytes_is_10mb() {
    assert_eq!(crate::MAX_STDIN_JSON_BYTES, 10 * 1024 * 1024);
}

#[test]
fn normal_input_within_size_limit() {
    let normal = r#"{"in_game":true,"game_state":{"screen_type":"NONE"}}"#;
    assert!(normal.len() <= crate::MAX_STDIN_JSON_BYTES);
}

#[test]
fn oversized_input_exceeds_limit() {
    let oversized = "x".repeat(crate::MAX_STDIN_JSON_BYTES + 1);
    assert!(oversized.len() > crate::MAX_STDIN_JSON_BYTES);
}
