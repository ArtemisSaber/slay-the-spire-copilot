use super::*;

#[test]
fn sanitize_err_body_redacts_api_key() {
    let key = "sk-secret-key-12345";
    let body = r#"{"error":"Authorization header: Bearer sk-secret-key-12345"}"#;
    let result = sanitize_err_body(body, Some(key));
    assert!(!result.contains(key), "api key must be redacted");
    assert!(result.contains("<REDACTED>"));
}

#[test]
fn sanitize_err_body_handles_empty_key() {
    let body = r#"{"error":"bad request"}"#;
    let result = sanitize_err_body(body, Some(""));
    assert!(result.contains("bad request"));
    assert!(!result.contains("<REDACTED>"));
}

#[test]
fn sanitize_err_body_handles_none_key() {
    let body = r#"{"error":"rate limited"}"#;
    let result = sanitize_err_body(body, None);
    assert!(result.contains("rate limited"));
}

#[test]
fn sanitize_err_body_truncates_long_body() {
    let long_body = "x".repeat(1000);
    let result = sanitize_err_body(&long_body, None);
    assert!(result.len() < long_body.len());
    assert!(result.ends_with("...(truncated)"));
}

#[test]
fn sanitize_err_body_preserves_short_body() {
    let body = r#"{"error":"not found"}"#;
    let result = sanitize_err_body(body, None);
    assert_eq!(result, body);
}

#[test]
fn validate_base_url_accepts_https() {
    assert!(validate_base_url("https://api.openai.com/v1").is_ok());
}

#[test]
fn validate_base_url_accepts_http_localhost() {
    // Local LLM servers (Ollama, LM Studio) run on http://localhost — must be allowed.
    assert!(validate_base_url("http://localhost:11434").is_ok());
}

#[test]
fn validate_base_url_rejects_file_scheme() {
    let result = validate_base_url("file:///etc/passwd");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("http") || err.contains("scheme"),
        "error: {err}"
    );
}

#[test]
fn validate_base_url_rejects_ftp_scheme() {
    let result = validate_base_url("ftp://example.com");
    assert!(result.is_err());
}

#[test]
fn validate_base_url_rejects_data_scheme() {
    let result = validate_base_url("data:text/plain,hello");
    assert!(result.is_err());
}

#[test]
fn validate_base_url_rejects_schemeless_url() {
    let result = validate_base_url("api.openai.com");
    assert!(result.is_err());
}

#[test]
fn validate_base_url_strips_trailing_slash() {
    let result = validate_base_url("https://api.openai.com/").unwrap();
    assert_eq!(result, "https://api.openai.com");
}

#[test]
fn validate_base_url_is_case_insensitive_for_scheme() {
    assert!(validate_base_url("HTTPS://api.openai.com").is_ok());
    assert!(validate_base_url("Http://localhost:8080").is_ok());
}
