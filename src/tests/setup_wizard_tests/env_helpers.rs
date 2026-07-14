use super::*;

#[test]
fn env_line_key_simple() {
    assert_eq!(env_line_key("KEY=value"), Some("KEY"));
    assert_eq!(env_line_key("LLM_PROVIDER=mock"), Some("LLM_PROVIDER"));
}

#[test]
fn env_line_key_with_whitespace() {
    assert_eq!(env_line_key("  KEY = value"), Some("KEY"));
    assert_eq!(env_line_key("\tKEY=value"), Some("KEY"));
}

#[test]
fn env_line_key_comment_returns_none() {
    assert_eq!(env_line_key("# comment"), None);
    assert_eq!(env_line_key("  # indented"), None);
}

#[test]
fn env_line_key_no_equals() {
    assert_eq!(env_line_key("JUSTTEXT"), None);
    assert_eq!(env_line_key(""), None);
}

#[test]
fn env_line_key_empty_key() {
    assert_eq!(env_line_key("=value"), None);
    assert_eq!(env_line_key("  =value"), None);
}

#[test]
fn format_env_value_empty() {
    assert_eq!(format_env_value(""), "");
}

#[test]
fn format_env_value_safe_chars_no_quoting() {
    assert_eq!(format_env_value("gpt-4o-mini"), "gpt-4o-mini");
    assert_eq!(
        format_env_value("https://api.openai.com/v1"),
        "https://api.openai.com/v1"
    );
    assert_eq!(format_env_value("openai-fast"), "openai-fast");
    assert_eq!(format_env_value("0.7"), "0.7");
    assert_eq!(format_env_value("true"), "true");
}

#[test]
fn format_env_value_needs_quoting() {
    assert_eq!(format_env_value("hello world"), "\"hello world\"");
    assert_eq!(format_env_value("key=value"), "\"key=value\"");
    assert_eq!(format_env_value("a@b"), "\"a@b\"");
}

#[test]
fn format_env_value_escapes_special_chars() {
    assert_eq!(format_env_value("say \"hi\""), "\"say \\\"hi\\\"\"");
    assert_eq!(format_env_value("a\\b"), "\"a\\\\b\"");
}

#[test]
fn existing_or_default_key_present() {
    let mut map = HashMap::new();
    map.insert("LLM_MODEL".to_string(), "gpt-5".to_string());
    assert_eq!(existing_or_default(&map, "LLM_MODEL", "fallback"), "gpt-5");
}

#[test]
fn existing_or_default_key_missing() {
    let map: HashMap<String, String> = HashMap::new();
    assert_eq!(
        existing_or_default(&map, "LLM_MODEL", "gpt-4o-mini"),
        "gpt-4o-mini"
    );
}

#[test]
fn existing_or_default_empty_value_falls_back() {
    let mut map = HashMap::new();
    map.insert("KEY".to_string(), "".to_string());
    assert_eq!(existing_or_default(&map, "KEY", "default"), "default");
}

#[test]
fn existing_or_default_whitespace_value_falls_back() {
    let mut map = HashMap::new();
    map.insert("KEY".to_string(), "   ".to_string());
    assert_eq!(existing_or_default(&map, "KEY", "default"), "default");
}

#[test]
fn assignment_creates_correct_struct() {
    let a = assignment("LLM_PROVIDER", "openai-compatible");
    assert_eq!(a.key, "LLM_PROVIDER");
    assert_eq!(a.value, "openai-compatible");
}

#[test]
fn assignment_accepts_into_string() {
    let a = assignment("KEY", String::from("value"));
    assert_eq!(a.key, "KEY");
    assert_eq!(a.value, "value");
}

#[test]
fn missing_value_when_key_missing() {
    let values = HashMap::new();
    assert!(missing_value(&values, "LLM_API_KEY"));
}

#[test]
fn missing_value_when_empty_or_whitespace() {
    let mut values = HashMap::new();
    values.insert("LLM_API_KEY".to_string(), "".to_string());
    assert!(missing_value(&values, "LLM_API_KEY"));
    values.insert("LLM_API_KEY".to_string(), "  ".to_string());
    assert!(missing_value(&values, "LLM_API_KEY"));
}

#[test]
fn missing_value_when_present() {
    let mut values = HashMap::new();
    values.insert("LLM_API_KEY".to_string(), "sk-real".to_string());
    assert!(!missing_value(&values, "LLM_API_KEY"));
}

#[test]
fn missing_or_placeholder_empty_or_whitespace() {
    assert!(missing_or_placeholder(""));
    assert!(missing_or_placeholder("   "));
}

#[test]
fn missing_or_placeholder_is_placeholder() {
    assert!(missing_or_placeholder("sk-your-key-here"));
    assert!(missing_or_placeholder("  sk-your-key-here  "));
}

#[test]
fn missing_or_placeholder_real_value() {
    assert!(!missing_or_placeholder("sk-real-key-12345"));
}

#[test]
fn is_placeholder_api_key_matches() {
    assert!(is_placeholder_api_key("sk-your-key-here"));
    assert!(is_placeholder_api_key("  sk-your-key-here  "));
}

#[test]
fn is_placeholder_api_key_does_not_match() {
    assert!(!is_placeholder_api_key("sk-real"));
    assert!(!is_placeholder_api_key(""));
    assert!(!is_placeholder_api_key("sk-your-key-here-real"));
}

#[test]
fn provider_value_present() {
    let mut values = HashMap::new();
    values.insert("LLM_PROVIDER".to_string(), "openai-compatible".to_string());
    assert_eq!(provider_value(&values), Some("openai-compatible"));
}

#[test]
fn provider_value_missing() {
    let values: HashMap<String, String> = HashMap::new();
    assert_eq!(provider_value(&values), None);
}

#[test]
fn read_env_values_file_exists() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(&path, "LLM_PROVIDER=mock\nLLM_MODEL=gpt-5\n").unwrap();
    let values = read_env_values(&path);
    assert_eq!(values.get("LLM_PROVIDER").map(String::as_str), Some("mock"));
    assert_eq!(values.get("LLM_MODEL").map(String::as_str), Some("gpt-5"));
}

#[test]
fn read_env_values_file_missing_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env.nonexistent");
    let values = read_env_values(&path);
    assert!(values.is_empty());
}

#[test]
fn env_file_needs_setup_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(&path, "").unwrap();
    assert!(env_file_needs_setup(&path));
}

#[test]
fn env_file_needs_setup_unknown_provider() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(&path, "LLM_PROVIDER=some-unknown\n").unwrap();
    assert!(env_file_needs_setup(&path));
}

#[test]
fn env_file_needs_setup_anthropic_missing_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(
        &path,
        "LLM_PROVIDER=anthropic\nLLM_BASE_URL=https://api.anthropic.com\nLLM_API_KEY=\n",
    )
    .unwrap();
    assert!(env_file_needs_setup(&path));
}

#[test]
fn env_file_needs_setup_anthropic_placeholder_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(
        &path,
        "LLM_PROVIDER=anthropic\nLLM_BASE_URL=https://api.anthropic.com\nLLM_API_KEY=sk-your-key-here\n",
    )
    .unwrap();
    assert!(env_file_needs_setup(&path));
}

#[test]
fn env_file_needs_setup_pollinations_missing_base_url() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(&path, "LLM_PROVIDER=pollinations-free\n").unwrap();
    assert!(env_file_needs_setup(&path));
}

#[test]
fn env_file_needs_setup_no_provider_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(&path, "LLM_MODEL=gpt-5\n").unwrap();
    assert!(env_file_needs_setup(&path));
}
