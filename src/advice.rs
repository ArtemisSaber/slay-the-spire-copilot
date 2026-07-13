mod overlay;
mod response;

pub(crate) use overlay::{OVERLAY_LOCK, atomic_write_json, timestamp_ms, write_overlay_json_to};
pub use overlay::{OverlayMetadata, OverlayOutput};
pub use response::{AdviceFields, parse_advice_response};

use crate::llm::{AdviceScenario, Effort, LlmProvider};
use crate::locales::Locale;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AdviceCache {
    cache: HashMap<String, String>,
    hide_timer: Option<tokio::task::JoinHandle<()>>,
    overlay_gen: Arc<Mutex<u64>>,
    hide_delay: Duration,
}

impl AdviceCache {
    pub fn new() -> Self {
        AdviceCache {
            cache: HashMap::new(),
            hide_timer: None,
            overlay_gen: Arc::new(Mutex::new(0)),
            hide_delay: Duration::from_secs(30),
        }
    }

    pub async fn get_or_compute(
        &mut self,
        hash: &str,
        prompt: &str,
        effort: Effort,
        scenario: AdviceScenario,
        provider: &LlmProvider,
        locale: &Locale,
    ) -> String {
        if let Some(cached) = self.cache.get(hash) {
            return cached.clone();
        }

        let advice = match provider
            .query_advice(prompt, effort, scenario, locale)
            .await
        {
            Ok(text) => text,
            Err(e) => {
                tracing::error!("LLM call failed: {e}");
                locale.fallback.llm_error.clone()
            }
        };

        self.cache.insert(hash.to_string(), advice.clone());
        advice
    }

    pub fn write_advice(&self, advice: &str) {
        let output_dir = crate::logging::advice_output_dir().join("output");
        if let Err(e) = fs::create_dir_all(&output_dir) {
            tracing::warn!("failed to create output dir {}: {e}", output_dir.display());
        }

        let path = output_dir.join("advice.txt");
        let latest = format!("{advice}\n");

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            && let Err(e) = file.write_all(latest.as_bytes())
        {
            tracing::warn!("failed to write {}: {e}", path.display());
        }
    }

    pub fn write_overlay_loading(&mut self, metadata: &OverlayMetadata) {
        self.cancel_hide_timer();

        let output = OverlayOutput {
            schema_version: 1,
            status: "loading".to_string(),
            overlay_visibility: true,
            advice: AdviceFields::default(),
            screen_type: metadata.screen_type.clone(),
            scenario: metadata.scenario.clone(),
            in_combat: metadata.in_combat,
            state_hash: metadata.state_hash.clone(),
            floor: metadata.floor,
            character: metadata.character.clone(),
            timestamp_ms: timestamp_ms(),
        };

        let overlay_path = crate::logging::advice_output_dir()
            .join("output")
            .join("overlay.json");
        write_overlay_json_to(&overlay_path, &output);
    }

    pub fn write_overlay_ready(
        &mut self,
        status: &str,
        fields: &AdviceFields,
        metadata: &OverlayMetadata,
    ) {
        self.cancel_hide_timer();

        let output = OverlayOutput {
            schema_version: 1,
            status: status.to_string(),
            overlay_visibility: true,
            advice: fields.clone(),
            screen_type: metadata.screen_type.clone(),
            scenario: metadata.scenario.clone(),
            in_combat: metadata.in_combat,
            state_hash: metadata.state_hash.clone(),
            floor: metadata.floor,
            character: metadata.character.clone(),
            timestamp_ms: timestamp_ms(),
        };

        let overlay_path = crate::logging::advice_output_dir()
            .join("output")
            .join("overlay.json");
        write_overlay_json_to(&overlay_path, &output);

        // Start 30s hide timer
        let ticket = {
            let mut g = self.overlay_gen.lock().unwrap_or_else(|e| e.into_inner());
            *g += 1;
            *g
        };

        let path = overlay_path.clone();
        let gen_counter = Arc::clone(&self.overlay_gen);
        let delay = self.hide_delay;

        let handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let current_gen = *gen_counter.lock().unwrap_or_else(|e| e.into_inner());
            if current_gen != ticket {
                return;
            }

            tokio::task::spawn_blocking(move || {
                if let Ok(content) = fs::read_to_string(&path)
                    && let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&content)
                {
                    v["overlay_visibility"] = serde_json::Value::Bool(false);
                    if let Ok(json) = serde_json::to_string_pretty(&v) {
                        atomic_write_json(&path, &json);
                    }
                }
            })
            .await
            .ok();
        });

        self.hide_timer = Some(handle);
    }

    fn cancel_hide_timer(&mut self) {
        if let Some(handle) = self.hide_timer.take() {
            handle.abort();
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
