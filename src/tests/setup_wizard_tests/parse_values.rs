use super::*;

#[test]
fn parse_env_values_key_value() {
    let result = parse_env_values("KEY=value\nOTHER=other\n");
    assert_eq!(result.get("KEY").map(String::as_str), Some("value"));
    assert_eq!(result.get("OTHER").map(String::as_str), Some("other"));
    assert_eq!(result.len(), 2);
}

#[test]
fn parse_env_values_with_spaces() {
    let result = parse_env_values("  KEY = value with spaces  \n");
    assert_eq!(
        result.get("KEY").map(String::as_str),
        Some("value with spaces")
    );
}

#[test]
fn parse_env_values_comments_and_blanks() {
    let result = parse_env_values("# comment\n\n  # indented\n\nKEY=val\n");
    assert_eq!(result.get("KEY").map(String::as_str), Some("val"));
    assert_eq!(result.len(), 1);
}

#[test]
fn parse_env_values_double_quoted() {
    let result = parse_env_values("KEY=\"hello world\"\n");
    assert_eq!(result.get("KEY").map(String::as_str), Some("hello world"));
}

#[test]
fn parse_env_values_single_quoted() {
    let result = parse_env_values("KEY='single quoted'\n");
    assert_eq!(result.get("KEY").map(String::as_str), Some("single quoted"));
}

#[test]
fn parse_env_values_escaped_quote() {
    let content = "KEY=\"escaped \\\"quote\\\"\"\n";
    let result = parse_env_values(content);
    assert_eq!(
        result.get("KEY").map(String::as_str),
        Some("escaped \"quote\"")
    );
}

#[test]
fn parse_env_values_backslash_escapes() {
    let content = "KEY=\"path\\\\to\\\\file\"\n";
    let result = parse_env_values(content);
    assert_eq!(
        result.get("KEY").map(String::as_str),
        Some("path\\to\\file")
    );
}

#[test]
fn parse_env_values_no_equals_skipped() {
    let result = parse_env_values("JUSTTEXT\nKEY=val\n");
    assert_eq!(result.get("KEY").map(String::as_str), Some("val"));
    assert!(!result.contains_key("JUSTTEXT"));
}

#[test]
fn parse_env_values_empty_key_skipped() {
    let result = parse_env_values("=value\nKEY=real\n");
    assert_eq!(result.get("KEY").map(String::as_str), Some("real"));
    assert_eq!(result.len(), 1);
}

#[test]
fn parse_env_values_empty_value() {
    let result = parse_env_values("KEY=\n");
    assert_eq!(result.get("KEY").map(String::as_str), Some(""));
}

#[test]
fn parse_env_value_unquoted() {
    assert_eq!(parse_env_value("hello"), "hello");
    assert_eq!(parse_env_value("hello world"), "hello world");
    assert_eq!(parse_env_value(""), "");
}

#[test]
fn parse_env_value_double_quoted_basic() {
    assert_eq!(parse_env_value("\"hello\""), "hello");
    assert_eq!(parse_env_value("\"hello world\""), "hello world");
}

#[test]
fn parse_env_value_double_quoted_escapes() {
    assert_eq!(
        parse_env_value("\"hello \\\"world\\\"\""),
        "hello \"world\""
    );
    assert_eq!(parse_env_value("\"path\\\\to\""), "path\\to");
    assert_eq!(parse_env_value("\"\\\\\\\"\""), "\\\"");
}

#[test]
fn parse_env_value_single_quoted_basic() {
    assert_eq!(parse_env_value("'hello'"), "hello");
    assert_eq!(parse_env_value("'hello world'"), "hello world");
}

#[test]
fn parse_env_value_single_quoted_no_escapes() {
    assert_eq!(parse_env_value("'back\\slash'"), "back\\slash");
    assert_eq!(parse_env_value("'quote\"inside'"), "quote\"inside");
}
