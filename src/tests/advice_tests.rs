use super::*;
use crate::llm::{AdviceScenario, Effort, LlmProvider};

#[tokio::test]
async fn cache_returns_cached_value() {
    let provider = LlmProvider::Mock;
    let mut cache = AdviceCache::new();

    let hash = "abc123";
    let prompt = "test prompt";

    let first = cache
        .get_or_compute(
            hash,
            prompt,
            Effort::Fast,
            AdviceScenario::Generic,
            &provider,
        )
        .await;
    let second = cache
        .get_or_compute(
            hash,
            prompt,
            Effort::Fast,
            AdviceScenario::Generic,
            &provider,
        )
        .await;

    assert_eq!(first, second);
}

#[test]
fn advice_output_dir_is_current_working_directory() {
    let dir = crate::logging::advice_output_dir();
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(dir, cwd);
}

#[test]
fn write_advice_output_path_behavior() {
    let cwd_file = crate::logging::advice_output_dir()
        .join("output")
        .join("advice.txt");
    let binary_file = crate::logging::project_root()
        .join("output")
        .join("advice.txt");

    let _ = std::fs::remove_file(&cwd_file);
    let _ = std::fs::remove_file(&binary_file);
    let _ = std::fs::remove_dir_all(crate::logging::advice_output_dir().join("output"));

    let cache = AdviceCache::new();

    cache.write_advice("first");
    assert!(
        cwd_file.exists(),
        "should create output/ dir and write to cwd"
    );

    if cwd_file != binary_file {
        assert!(
            !binary_file.exists(),
            "should not write to binary's output/ when cwd differs"
        );
    }

    cache.write_advice("second");
    let content = std::fs::read_to_string(&cwd_file).unwrap();
    assert!(
        !content.contains("first"),
        "latest write should replace previous"
    );
    assert!(content.contains("second"));

    let _ = std::fs::remove_file(&cwd_file);
}

#[test]
fn journal_remains_source_of_advice_history() {
    let dir = tempfile::tempdir().unwrap();
    let journal = crate::journal::Journal::new_at(dir.path(), "test-run");
    journal.log_advice(
        "hash1",
        Effort::Fast,
        AdviceScenario::Generic,
        "prompt 1",
        "first",
    );
    journal.log_advice(
        "hash2",
        Effort::Heavy,
        AdviceScenario::CardReward,
        "prompt 2",
        "second",
    );

    let content = std::fs::read_to_string(journal.path()).unwrap();
    assert!(content.contains("first"));
    assert!(content.contains("second"));
}

#[tokio::test]
async fn get_or_compute_llm_error_fallback() {
    let provider = LlmProvider::MockError;
    let mut cache = AdviceCache::new();
    let result = cache
        .get_or_compute(
            "hash1",
            "test prompt",
            Effort::Fast,
            AdviceScenario::Generic,
            &provider,
        )
        .await;
    assert_eq!(result, "LLM 调用失败，请检查配置。");
}
