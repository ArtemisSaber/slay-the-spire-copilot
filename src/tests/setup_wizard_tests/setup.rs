use std::io::Cursor;

use super::*;

#[test]
fn missing_env_file_needs_setup() {
    let dir = tempfile::tempdir().unwrap();
    assert!(env_file_needs_setup(&dir.path().join(".env")));
}

#[test]
fn mock_provider_is_considered_configured() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(&path, "LLM_PROVIDER=mock\n").unwrap();

    assert!(!env_file_needs_setup(&path));
}

#[test]
fn placeholder_openai_key_needs_setup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(
        &path,
        "LLM_PROVIDER=openai-compatible\nLLM_BASE_URL=https://api.openai.com/v1\nLLM_API_KEY=sk-your-key-here\n",
    )
    .unwrap();

    assert!(env_file_needs_setup(&path));
}

#[test]
fn complete_openai_config_does_not_need_setup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(
        &path,
        "LLM_PROVIDER=openai-compatible\nLLM_BASE_URL=https://api.openai.com/v1\nLLM_API_KEY=sk-real\n",
    )
    .unwrap();

    assert!(!env_file_needs_setup(&path));
}

#[test]
fn complete_pollinations_free_config_does_not_need_setup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(
        &path,
        "LLM_PROVIDER=pollinations-free\nLLM_BASE_URL=https://text.pollinations.ai/openai\nLLM_API_KEY=\n",
    )
    .unwrap();

    assert!(!env_file_needs_setup(&path));
}

#[test]
fn setup_wizard_writes_pollinations_free_config_without_api_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    let mut input = Cursor::new("y\n1\n\n");
    let mut output = Vec::new();

    let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

    assert!(saved);
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("LLM_PROVIDER=pollinations-free"));
    assert!(content.contains("LLM_BASE_URL=https://text.pollinations.ai/openai"));
    assert!(content.contains("LLM_API_KEY=\n"));
    assert!(content.contains("LLM_MODEL=openai-fast"));
}

#[test]
fn complete_anthropic_config_does_not_need_setup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(
        &path,
        "LLM_PROVIDER=anthropic\nLLM_BASE_URL=https://api.anthropic.com\nLLM_API_KEY=sk-ant-real\n",
    )
    .unwrap();

    assert!(!env_file_needs_setup(&path));
}

#[test]
fn setup_wizard_writes_openai_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    let mut input = Cursor::new("y\n2\nsk-real\ncustom-model\n");
    let mut output = Vec::new();

    let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

    assert!(saved);
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("LLM_PROVIDER=openai-compatible"));
    assert!(content.contains("LLM_BASE_URL=https://api.openai.com/v1"));
    assert!(content.contains("LLM_API_KEY=sk-real"));
    assert!(content.contains("LLM_MODEL=custom-model"));
    assert!(content.contains("LLM_MODEL_FAST=custom-model"));
    assert!(content.contains("LLM_MODEL_MEDIUM=custom-model"));
    assert!(content.contains("LLM_MODEL_HEAVY=custom-model"));
}

#[test]
fn setup_wizard_writes_anthropic_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    let mut input = Cursor::new("y\n3\nsk-ant-real\n\n");
    let mut output = Vec::new();

    let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

    assert!(saved);
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("LLM_PROVIDER=anthropic"));
    assert!(content.contains("LLM_BASE_URL=https://api.anthropic.com"));
    assert!(content.contains("LLM_API_KEY=sk-ant-real"));
    assert!(content.contains("LLM_MODEL=claude-sonnet-4-6"));
}

#[test]
fn setup_wizard_writes_deepseek_fast_thinking_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    let mut input = Cursor::new("y\n5\nsk-deepseek\n\n");
    let mut output = Vec::new();

    let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

    assert!(saved);
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("LLM_BASE_URL=https://api.deepseek.com"));
    assert!(content.contains("LLM_MODEL=deepseek-v4-flash"));
    assert!(content.contains("LLM_DISABLE_FAST_THINKING=true"));
}

#[test]
fn setup_wizard_writes_openrouter_free_router_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    let mut input = Cursor::new("y\n7\nsk-or\n\n");
    let mut output = Vec::new();

    let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

    assert!(saved);
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("LLM_BASE_URL=https://openrouter.ai/api/v1"));
    assert!(content.contains("LLM_MODEL=openrouter/free"));
}

#[test]
fn setup_wizard_can_write_compatible_tiered_models() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    let mut input = Cursor::new(
        "y\n10\nhttps://api.example.com/v1\nsk-real\nfallback\ny\nfast\nmedium\nheavy\n",
    );
    let mut output = Vec::new();

    let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

    assert!(saved);
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("LLM_BASE_URL=https://api.example.com/v1"));
    assert!(content.contains("LLM_MODEL_FAST=fast"));
    assert!(content.contains("LLM_MODEL_MEDIUM=medium"));
    assert!(content.contains("LLM_MODEL_HEAVY=heavy"));
}

#[test]
fn env_writer_preserves_unrelated_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(&path, "# keep me\nOTHER=value\nLLM_PROVIDER=mock\n").unwrap();

    write_env_assignments(&path, &[assignment("LLM_PROVIDER", "openai-compatible")]).unwrap();

    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("# keep me"));
    assert!(content.contains("OTHER=value"));
    assert!(content.contains("LLM_PROVIDER=openai-compatible"));
}
