use super::*;

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
            test_locale(),
        )
        .await;
    let second = cache
        .get_or_compute(
            hash,
            prompt,
            Effort::Fast,
            AdviceScenario::Generic,
            &provider,
            test_locale(),
        )
        .await;

    assert_eq!(first, second);
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

#[tokio::test]
async fn get_or_compute_llm_error_fallback() {
    let provider = LlmProvider::Mock;
    let mut cache = AdviceCache::new();
    let result = cache
        .get_or_compute(
            "hash1",
            "TRIGGER_LLM_ERROR",
            Effort::Fast,
            AdviceScenario::Generic,
            &provider,
            test_locale(),
        )
        .await;
    assert_eq!(result, "LLM 调用失败，请检查配置。");
}

#[test]
fn cancel_hide_timer_when_none_is_noop() {
    let mut cache = AdviceCache::new();
    cache.cancel_hide_timer();
    assert!(cache.hide_timer.is_none());
}

#[test]
fn overlay_gen_lock_survives_mutex_poison() {
    use std::sync::{Arc, Mutex};

    let m = Arc::new(Mutex::new(0u64));
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = m.lock().unwrap();
        panic!("intentional poison");
    }));
    let val = *m.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(val, 0);
}
