use crate::config::Config;
use crate::llm::{AdviceScenario, Effort};
use crate::state::NormalizedState;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA_VERSION: u32 = 1;

pub struct Journal {
    run_id: String,
    path: PathBuf,
    last_observation_hash: Option<String>,
    wrote_run_metadata: bool,
}

impl Journal {
    pub fn new() -> Self {
        let run_id = format!("{}-{}", timestamp_ms(), std::process::id());
        Self::new_at(crate::logging::project_root().join("runs"), run_id)
    }

    pub fn log_state_change(&mut self, advice_hash: &str, state: &NormalizedState) {
        self.log_first_observed_metadata(state);

        let observation_hash = state.observation_hash();
        if self.last_observation_hash.as_deref() == Some(&observation_hash) {
            return;
        }

        self.last_observation_hash = Some(observation_hash.clone());

        let event = json!({
            "schema_version": SCHEMA_VERSION,
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id,
            "event": "state_changed",
            "hash": advice_hash,
            "advice_hash": advice_hash,
            "observation_hash": observation_hash,
            "screen_type": state.screen_type,
            "room_type": state.room_type,
            "floor": state.floor,
            "normalized": state,
        });
        self.append_event(&event);
    }

    #[cfg(test)]
    pub fn log_run_started(&self) {
        let event = json!({
            "schema_version": SCHEMA_VERSION,
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id,
            "event": "run_started",
        });
        self.append_event(&event);
    }

    pub fn log_run_started_with_config(&self, config: &Config) {
        let event = json!({
            "schema_version": SCHEMA_VERSION,
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id,
            "event": "run_started",
            "provider": config.provider,
            "model_fast": config.model_fast,
            "model_medium": config.model_medium,
            "model_heavy": config.model_heavy,
            "app_version": env!("CARGO_PKG_VERSION"),
        });
        self.append_event(&event);
    }

    pub fn log_run_ended(&self, reason: &str) {
        let event = json!({
            "schema_version": SCHEMA_VERSION,
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id,
            "event": "run_ended",
            "reason": reason,
        });
        self.append_event(&event);
    }

    pub fn log_advice(
        &self,
        state_hash: &str,
        effort: Effort,
        scenario: AdviceScenario,
        prompt: &str,
        advice: &str,
    ) {
        let event = json!({
            "schema_version": SCHEMA_VERSION,
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id,
            "event": "advice",
            "state_hash": state_hash,
            "advice_hash": state_hash,
            "effort": effort.as_str(),
            "scenario": scenario.as_str(),
            "prompt": prompt,
            "advice": advice,
        });
        self.append_event(&event);
    }

    #[cfg(test)]
    pub fn new_at(root: impl AsRef<Path>, run_id: impl Into<String>) -> Self {
        Self::new_at_inner(root.as_ref().to_path_buf(), run_id.into())
    }

    #[cfg(not(test))]
    fn new_at(root: impl AsRef<Path>, run_id: impl Into<String>) -> Self {
        Self::new_at_inner(root.as_ref().to_path_buf(), run_id.into())
    }

    fn new_at_inner(root: PathBuf, run_id: String) -> Self {
        let dir = root.join(&run_id);
        if let Err(e) = fs::create_dir_all(&dir) {
            tracing::error!("failed to create journal directory {}: {e}", dir.display());
        }

        Journal {
            run_id,
            path: dir.join("events.jsonl"),
            last_observation_hash: None,
            wrote_run_metadata: false,
        }
    }

    fn log_first_observed_metadata(&mut self, state: &NormalizedState) {
        if self.wrote_run_metadata {
            return;
        }
        self.wrote_run_metadata = true;

        let event = json!({
            "schema_version": SCHEMA_VERSION,
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id,
            "event": "run_metadata",
            "character": state.character,
            "ascension_level": state.ascension_level,
            "seed": state.seed,
            "floor": state.floor,
        });
        self.append_event(&event);
    }

    fn append_event(&self, event: &serde_json::Value) {
        let Ok(line) = serde_json::to_string(event) else {
            tracing::error!("failed to serialize journal event");
            return;
        };

        match fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            Ok(mut file) => {
                if let Err(e) = writeln!(file, "{line}") {
                    tracing::error!("failed to write journal event: {e}");
                }
            }
            Err(e) => tracing::error!("failed to open journal file {}: {e}", self.path.display()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Default for Journal {
    fn default() -> Self {
        Self::new()
    }
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "tests/journal_tests.rs"]
mod tests;
