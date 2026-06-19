use super::*;
use crate::llm::{Effort, LlmProvider};

#[tokio::test]
async fn cache_returns_cached_value() {
    let provider = LlmProvider::Mock;
    let mut cache = AdviceCache::new();

    let hash = "abc123";
    let prompt = "test prompt";

    let first = cache
        .get_or_compute(hash, prompt, Effort::Fast, &provider)
        .await;
    let second = cache
        .get_or_compute(hash, prompt, Effort::Fast, &provider)
        .await;

    assert_eq!(first, second);
}

#[test]
fn write_advice_appends() {
    let path = crate::logging::project_root()
        .join("output")
        .join("advice.txt");
    let _ = std::fs::remove_file(&path);

    let cache = AdviceCache::new();
    cache.write_advice("first");
    cache.write_advice("second");

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("first"));
    assert!(content.contains("second"));

    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn get_or_compute_llm_error_fallback() {
    let provider = LlmProvider::MockError;
    let mut cache = AdviceCache::new();
    let result = cache
        .get_or_compute("hash1", "test prompt", Effort::Fast, &provider)
        .await;
    assert_eq!(result, "LLM 调用失败，请检查配置。");
}
