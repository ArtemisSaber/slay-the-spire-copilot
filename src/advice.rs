use crate::llm::{Effort, LlmProvider};
use std::collections::HashMap;
use std::fs;
use std::io::Write;

pub struct AdviceCache {
    cache: HashMap<String, String>,
}

impl AdviceCache {
    pub fn new() -> Self {
        AdviceCache {
            cache: HashMap::new(),
        }
    }

    pub async fn get_or_compute(
        &mut self,
        hash: &str,
        prompt: &str,
        effort: Effort,
        provider: &LlmProvider,
    ) -> String {
        if let Some(cached) = self.cache.get(hash) {
            return cached.clone();
        }

        let advice = match provider.query(prompt, effort).await {
            Ok(text) => text,
            Err(e) => {
                tracing::error!("LLM call failed: {e}");
                "LLM 调用失败，请检查配置。".to_string()
            }
        };

        self.cache.insert(hash.to_string(), advice.clone());
        advice
    }

    pub fn write_advice(&self, advice: &str) {
        let output_dir = crate::logging::project_root().join("output");
        let _ = fs::create_dir_all(&output_dir);

        let path = output_dir.join("advice.txt");
        let entry = format!("{advice}\n---\n");

        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

impl Default for AdviceCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
        let cache = AdviceCache::new();
        cache.write_advice("first");
        cache.write_advice("second");

        let path = crate::logging::project_root()
            .join("output")
            .join("advice.txt");
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("first"));
        assert!(content.contains("second"));
    }
}
