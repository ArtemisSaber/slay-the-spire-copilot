use std::io::Cursor;

use super::*;

fn existing_openai() -> &'static str {
    "LLM_PROVIDER=openai-compatible\n\
LLM_BASE_URL=https://api.openai.com/v1\n\
LLM_API_KEY=sk-real\n\
LLM_MODEL=existing-model\n\
AUTO_PLAY=false\n\
MEMORY_MODE=off\n"
}

#[test]
fn explicit_setup_can_change_learning_without_reconfiguring_api() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(&path, existing_openai()).unwrap();
    let mut input = Cursor::new("n\ny\ny\nn\n");
    let mut output = Vec::new();

    assert!(run_setup_with_io(&path, &mut input, &mut output, true).unwrap());

    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("LLM_API_KEY=sk-real"));
    assert!(content.contains("LLM_MODEL=existing-model"));
    assert!(content.contains("AUTO_PLAY=true"));
    assert!(content.contains("MEMORY_MODE=collect"));
    assert!(
        !String::from_utf8(output)
            .unwrap()
            .contains("Choose an API connection")
    );
}

#[test]
fn api_reconfiguration_defaults_to_the_existing_preset() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    fs::write(&path, existing_openai()).unwrap();
    let mut input = Cursor::new("y\n\n\n\n\n");
    let mut output = Vec::new();

    assert!(run_setup_with_io(&path, &mut input, &mut output, true).unwrap());

    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("LLM_PROVIDER=openai-compatible"));
    assert!(content.contains("LLM_BASE_URL=https://api.openai.com/v1"));
    assert!(content.contains("LLM_API_KEY=sk-real"));
    assert!(content.contains("LLM_MODEL=existing-model"));
    assert!(String::from_utf8(output).unwrap().contains("Selection [2]"));
}
