use super::*;

#[test]
fn provider_assignments_has_all_13_keys() {
    let setup = ProviderSetup {
        provider: "test-provider",
        base_url: "https://test.example.com".to_string(),
        api_key: "sk-test".to_string(),
        fast: "fast-model".to_string(),
        medium: "medium-model".to_string(),
        heavy: "heavy-model".to_string(),
        disable_fast_thinking_default: Some("true"),
    };
    let existing = HashMap::new();
    let assignments = provider_assignments(setup, &existing);
    assert_eq!(assignments.len(), 13);

    let keys: Vec<&str> = assignments.iter().map(|a| a.key).collect();
    for expected_key in LLM_ENV_KEYS {
        assert!(keys.contains(expected_key), "missing key: {expected_key}");
    }

    assert_eq!(find_value(&assignments, "LLM_PROVIDER"), "test-provider");
    assert_eq!(
        find_value(&assignments, "LLM_BASE_URL"),
        "https://test.example.com"
    );
    assert_eq!(find_value(&assignments, "LLM_API_KEY"), "sk-test");
    assert_eq!(find_value(&assignments, "LLM_MODEL"), "heavy-model");
    assert_eq!(find_value(&assignments, "LLM_MODEL_FAST"), "fast-model");
    assert_eq!(find_value(&assignments, "LLM_MODEL_MEDIUM"), "medium-model");
    assert_eq!(find_value(&assignments, "LLM_MODEL_HEAVY"), "heavy-model");
    assert_eq!(
        find_value(&assignments, "LLM_MAX_TOKENS"),
        DEFAULT_MAX_TOKENS
    );
    assert_eq!(
        find_value(&assignments, "LLM_MAX_TOKENS_FAST"),
        DEFAULT_MAX_TOKENS_FAST
    );
    assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS_MEDIUM"), "");
    assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS_HEAVY"), "");
    assert_eq!(
        find_value(&assignments, "LLM_TEMPERATURE"),
        DEFAULT_TEMPERATURE
    );
    assert_eq!(
        find_value(&assignments, "LLM_DISABLE_FAST_THINKING"),
        "true"
    );
}

#[test]
fn provider_assignments_respects_existing_values() {
    let setup = ProviderSetup {
        provider: "test-provider",
        base_url: "https://test.example.com".to_string(),
        api_key: "sk-test".to_string(),
        fast: "fast".to_string(),
        medium: "medium".to_string(),
        heavy: "heavy".to_string(),
        disable_fast_thinking_default: None,
    };
    let mut existing = HashMap::new();
    existing.insert("LLM_MAX_TOKENS".to_string(), "999".to_string());
    existing.insert("LLM_MAX_TOKENS_HEAVY".to_string(), "888".to_string());
    existing.insert("LLM_TEMPERATURE".to_string(), "0.3".to_string());

    let assignments = provider_assignments(setup, &existing);
    assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS"), "999");
    assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS_HEAVY"), "888");
    assert_eq!(find_value(&assignments, "LLM_TEMPERATURE"), "0.3");
}

#[test]
fn provider_assignments_disable_fast_thinking_none() {
    let setup = ProviderSetup {
        provider: "test-provider",
        base_url: "https://test.example.com".to_string(),
        api_key: "sk-test".to_string(),
        fast: "fast".to_string(),
        medium: "medium".to_string(),
        heavy: "heavy".to_string(),
        disable_fast_thinking_default: None,
    };
    let existing = HashMap::new();
    let assignments = provider_assignments(setup, &existing);
    assert_eq!(find_value(&assignments, "LLM_DISABLE_FAST_THINKING"), "");
}

#[test]
fn mock_assignments_has_all_13_keys() {
    let existing = HashMap::new();
    let assignments = mock_assignments(&existing);
    assert_eq!(assignments.len(), 13);

    let keys: Vec<&str> = assignments.iter().map(|a| a.key).collect();
    for expected_key in LLM_ENV_KEYS {
        assert!(keys.contains(expected_key), "missing key: {expected_key}");
    }

    assert_eq!(find_value(&assignments, "LLM_PROVIDER"), "mock");
    assert_eq!(
        find_value(&assignments, "LLM_BASE_URL"),
        DEFAULT_OPENAI_BASE_URL
    );
    assert_eq!(find_value(&assignments, "LLM_API_KEY"), PLACEHOLDER_API_KEY);
    assert_eq!(find_value(&assignments, "LLM_MODEL"), DEFAULT_MODEL);
    assert_eq!(find_value(&assignments, "LLM_MODEL_FAST"), DEFAULT_MODEL);
    assert_eq!(find_value(&assignments, "LLM_MODEL_MEDIUM"), "");
    assert_eq!(find_value(&assignments, "LLM_MODEL_HEAVY"), "");
    assert_eq!(
        find_value(&assignments, "LLM_MAX_TOKENS"),
        DEFAULT_MAX_TOKENS
    );
    assert_eq!(
        find_value(&assignments, "LLM_MAX_TOKENS_FAST"),
        DEFAULT_MAX_TOKENS_FAST
    );
    assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS_MEDIUM"), "");
    assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS_HEAVY"), "");
    assert_eq!(
        find_value(&assignments, "LLM_TEMPERATURE"),
        DEFAULT_TEMPERATURE
    );
    assert_eq!(find_value(&assignments, "LLM_DISABLE_FAST_THINKING"), "");
}

#[test]
fn mock_assignments_respects_existing_values() {
    let mut existing = HashMap::new();
    existing.insert("LLM_MODEL".to_string(), "custom-model".to_string());
    existing.insert(
        "LLM_BASE_URL".to_string(),
        "https://custom.example.com".to_string(),
    );
    existing.insert("LLM_API_KEY".to_string(), "sk-custom".to_string());

    let assignments = mock_assignments(&existing);
    assert_eq!(find_value(&assignments, "LLM_MODEL"), "custom-model");
    assert_eq!(
        find_value(&assignments, "LLM_BASE_URL"),
        "https://custom.example.com"
    );
    assert_eq!(find_value(&assignments, "LLM_API_KEY"), "sk-custom");
}
