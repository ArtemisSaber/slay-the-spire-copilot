use crate::config::Config;
use crate::llm::{AdviceScenario, Effort};
use crate::locales::Locale;
use crate::state::NormalizedState;
use chrono::{DateTime, Local};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA_VERSION: u32 = 1;

enum JournalState {
    Pending { runs_root: PathBuf },
    Confirmed { path: PathBuf, run_id: String },
}

pub struct Journal {
    state: JournalState,
    is_continued: bool,
    last_observation_hash: Option<String>,
    wrote_run_metadata: bool,
}

impl Journal {
    pub fn new(runs_root: impl Into<PathBuf>) -> Self {
        Journal {
            state: JournalState::Pending {
                runs_root: runs_root.into(),
            },
            is_continued: false,
            last_observation_hash: None,
            wrote_run_metadata: false,
        }
    }

    pub fn is_confirmed(&self) -> bool {
        matches!(self.state, JournalState::Confirmed { .. })
    }

    #[allow(dead_code, reason = "public API for potential future use")]
    pub fn is_continued_run(&self) -> bool {
        self.is_continued
    }

    pub fn path(&self) -> Option<&Path> {
        match &self.state {
            JournalState::Pending { .. } => None,
            JournalState::Confirmed { path, .. } => Some(path),
        }
    }

    pub fn confirm(
        &mut self,
        seed: i64,
        character: &str,
        ascension: i64,
        config: &Config,
        locale: &Locale,
    ) {
        if self.is_confirmed() {
            return;
        }

        let runs_root = match &self.state {
            JournalState::Pending { runs_root } => runs_root.clone(),
            JournalState::Confirmed { .. } => {
                tracing::warn!("confirm_run called on already-confirmed journal, skipping");
                return;
            }
        };

        if let Some(existing_dir) = scan_for_existing_run(&runs_root, seed) {
            let run_id = existing_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let path = existing_dir.join("events.jsonl");
            self.is_continued = true;
            self.state = JournalState::Confirmed { path, run_id };
            self.log_run_continued(config, seed, character, ascension);
            return;
        }

        let ts_ms = timestamp_ms();
        let class_display = locale.i18n.character_display_name(character);
        let run_id = format_run_id(ts_ms, class_display, ascension);

        let dir = runs_root.join(&run_id);
        if let Err(e) = fs::create_dir_all(&dir) {
            tracing::error!("failed to create journal directory {}: {e}", dir.display());
        }

        let path = dir.join("events.jsonl");
        self.state = JournalState::Confirmed { path, run_id };
        self.log_run_started_with_config(config);
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
            "run_id": self.run_id(),
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
            "run_id": self.run_id(),
            "event": "run_started",
        });
        self.append_event(&event);
    }

    pub fn log_run_started_with_config(&self, config: &Config) {
        let event = json!({
            "schema_version": SCHEMA_VERSION,
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id(),
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
            "run_id": self.run_id(),
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
            "run_id": self.run_id(),
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

    fn log_run_continued(&self, config: &Config, seed: i64, character: &str, ascension: i64) {
        let event = json!({
            "schema_version": SCHEMA_VERSION,
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id(),
            "event": "run_continued",
            "provider": config.provider,
            "app_version": env!("CARGO_PKG_VERSION"),
            "seed": seed,
            "character": character,
            "ascension_level": ascension,
        });
        self.append_event(&event);
    }

    #[cfg(test)]
    pub fn new_at(root: impl AsRef<Path>, run_id: impl Into<String>) -> Self {
        Self::new_at_inner(root.as_ref().to_path_buf(), run_id.into())
    }

    #[cfg(test)]
    fn new_at_inner(root: PathBuf, run_id: String) -> Self {
        let dir = root.join(&run_id);
        if let Err(e) = fs::create_dir_all(&dir) {
            tracing::error!("failed to create journal directory {}: {e}", dir.display());
        }

        Journal {
            state: JournalState::Confirmed {
                path: dir.join("events.jsonl"),
                run_id,
            },
            is_continued: false,
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
            "run_id": self.run_id(),
            "event": "run_metadata",
            "character": state.character,
            "ascension_level": state.ascension_level,
            "seed": state.seed,
            "floor": state.floor,
        });
        self.append_event(&event);
    }

    fn append_event(&self, event: &serde_json::Value) {
        let Some(path) = self.path() else {
            tracing::error!("attempted to append event to unconfirmed journal");
            return;
        };

        let Ok(line) = serde_json::to_string(event) else {
            tracing::error!("failed to serialize journal event");
            return;
        };

        match fs::OpenOptions::new().create(true).append(true).open(path) {
            Ok(mut file) => {
                if let Err(e) = writeln!(file, "{line}") {
                    tracing::error!("failed to write journal event: {e}");
                }
            }
            Err(e) => tracing::error!("failed to open journal file {}: {e}", path.display()),
        }
    }

    fn run_id(&self) -> &str {
        match &self.state {
            JournalState::Pending { .. } => {
                tracing::error!("log event before journal confirmed");
                "unconfirmed"
            }
            JournalState::Confirmed { run_id, .. } => run_id,
        }
    }
}

fn scan_for_existing_run(runs_root: &Path, seed: i64) -> Option<PathBuf> {
    let entries = fs::read_dir(runs_root)
        .map_err(|e| tracing::warn!("failed to read runs dir {}: {e}", runs_root.display()))
        .ok()?;

    for entry in entries.filter_map(|e| {
        e.map_err(|err| tracing::warn!("dir entry error: {err}"))
            .ok()
    }) {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let journal_path = dir.join("events.jsonl");
        let content = fs::read_to_string(&journal_path)
            .map_err(|e| tracing::warn!("failed to read {}: {e}", journal_path.display()))
            .ok()?;

        let mut found_seed = false;
        let mut has_ended = false;

        for line in content.lines() {
            let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            match event.get("event").and_then(|v| v.as_str()) {
                Some("run_metadata") => {
                    if event.get("seed").and_then(|v| v.as_i64()) == Some(seed) {
                        found_seed = true;
                    }
                }
                Some("run_ended") => {
                    has_ended = true;
                }
                _ => {}
            }
        }

        if found_seed && !has_ended {
            return Some(dir);
        }
    }

    None
}

pub(crate) fn format_datetime(ts_ms: u128) -> String {
    let secs = (ts_ms / 1000) as i64;
    let utc = DateTime::from_timestamp(secs, 0).unwrap_or_default();
    let local: DateTime<Local> = DateTime::from(utc);
    local.format("%Y-%m-%d_%H-%M").to_string()
}

fn format_run_id(ts_ms: u128, character_display: &str, ascension: i64) -> String {
    format!(
        "{}_{}_A{ascension}",
        format_datetime(ts_ms),
        character_display
    )
}

pub(crate) fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "tests/journal_tests.rs"]
mod tests;
