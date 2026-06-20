use crate::llm::{AdviceScenario, Effort, LlmProvider};
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
        scenario: AdviceScenario,
        provider: &LlmProvider,
    ) -> String {
        if let Some(cached) = self.cache.get(hash) {
            return cached.clone();
        }

        let advice = match provider.query_advice(prompt, effort, scenario).await {
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
        let output_dir = crate::logging::advice_output_dir().join("output");
        let _ = fs::create_dir_all(&output_dir);

        let path = output_dir.join("advice.txt");
        let latest = format!("{advice}\n");

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
        {
            let _ = file.write_all(latest.as_bytes());
        }
    }
}

impl Default for AdviceCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "tests/advice_tests.rs"]
mod tests;
