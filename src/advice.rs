use crate::llm::{AdviceScenario, Effort, LlmProvider};
use crate::locales::Locale;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default, Serialize)]
pub struct AdviceFields {
    pub recommendation: String,
    pub reason: String,
    pub risk: String,
    pub commentary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlayOutput {
    pub schema_version: u32,
    pub status: String,
    pub overlay_visibility: bool,
    pub advice: AdviceFields,
    pub screen_type: Option<String>,
    pub scenario: String,
    pub in_combat: bool,
    pub state_hash: String,
    pub floor: Option<i64>,
    pub character: Option<String>,
    pub timestamp_ms: u128,
}

pub struct OverlayMetadata {
    pub screen_type: Option<String>,
    pub scenario: String,
    pub in_combat: bool,
    pub state_hash: String,
    pub floor: Option<i64>,
    pub character: Option<String>,
}

/// Serializes all overlay.json writes to prevent TOCTOU races between
/// the advice write path (write_overlay_json_to) and the autoplay status
/// path (write_overlay_autoplay, which does read-modify-write).
pub(crate) static OVERLAY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub fn parse_advice_response(raw: &str, locale: &Locale) -> AdviceFields {
    let mut recommendation = String::new();
    let mut reason = String::new();
    let mut risk = String::new();
    let mut commentary = String::new();

    let mut current: Option<&mut String> = None;

    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix(locale.parser.recommendation.as_str()) {
            recommendation.push_str(rest);
            current = Some(&mut recommendation);
        } else if let Some(rest) = line.strip_prefix(locale.parser.reason.as_str()) {
            reason.push_str(rest);
            current = Some(&mut reason);
        } else if let Some(rest) = line.strip_prefix(locale.parser.risk.as_str()) {
            risk.push_str(rest);
            current = Some(&mut risk);
        } else if let Some(rest) = line.strip_prefix(locale.parser.commentary.as_str()) {
            commentary.push_str(rest);
            current = Some(&mut commentary);
        } else if let Some(ref mut field) = current
            && !line.is_empty()
        {
            if !field.is_empty() {
                field.push('\n');
            }
            field.push_str(line);
        }
    }

    AdviceFields {
        recommendation,
        reason,
        risk,
        commentary,
    }
}

pub(crate) fn atomic_write_json(path: &std::path::Path, json: &str) {
    let parent = match path.parent() {
        Some(p) => p,
        None => {
            tracing::warn!("cannot write {}: path has no parent", path.display());
            return;
        }
    };
    if let Err(e) = fs::create_dir_all(parent) {
        tracing::warn!("failed to create dir {}: {e}", parent.display());
    }

    let mut tmp = match tempfile::NamedTempFile::new_in(parent) {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("failed to create temp file in {}: {e}", parent.display());
            return;
        }
    };

    if let Err(e) = tmp.write_all(json.as_bytes()) {
        tracing::warn!("failed to write temp file: {e}");
        return;
    }

    // persist() atomically replaces the target on both Unix (rename(2)) and
    // Windows (MoveFileExW with MOVEFILE_REPLACE_EXISTING). On any error the
    // NamedTempFile is consumed into PersistError and dropped, cleaning up the
    // temp file automatically.
    if let Err(e) = tmp.persist(path) {
        tracing::warn!("failed to persist temp file -> {}: {e}", path.display());
    }
}

fn write_overlay_json_to(path: &std::path::Path, output: &OverlayOutput) {
    if let Ok(json) = serde_json::to_string_pretty(output) {
        let _lock = OVERLAY_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        atomic_write_json(path, &json);
    }
}

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
            let mut g = self.overlay_gen.lock().unwrap();
            *g += 1;
            *g
        };

        let path = overlay_path.clone();
        let gen_counter = Arc::clone(&self.overlay_gen);
        let delay = self.hide_delay;

        let handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let current_gen = *gen_counter.lock().unwrap();
            if current_gen != ticket {
                return;
            }

            {
                if let Ok(content) = fs::read_to_string(&path)
                    && let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&content)
                {
                    v["overlay_visibility"] = serde_json::Value::Bool(false);
                    if let Ok(json) = serde_json::to_string_pretty(&v) {
                        atomic_write_json(&path, &json);
                    }
                }
            }
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
