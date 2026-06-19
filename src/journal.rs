use crate::llm::Effort;
use crate::state::NormalizedState;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Journal {
    run_id: String,
    path: PathBuf,
    last_state_hash: Option<String>,
}

impl Journal {
    pub fn new() -> Self {
        let run_id = format!("{}-{}", timestamp_ms(), std::process::id());
        Self::new_at(crate::logging::project_root().join("runs"), run_id)
    }

    pub fn log_state_change(&mut self, hash: &str, state: &NormalizedState) {
        if self.last_state_hash.as_deref() == Some(hash) {
            return;
        }

        self.last_state_hash = Some(hash.to_string());

        let event = json!({
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id,
            "event": "state_changed",
            "hash": hash,
            "screen_type": state.screen_type,
            "room_type": state.room_type,
            "floor": state.floor,
            "normalized": state,
        });
        self.append_event(&event);
    }

    pub fn log_run_started(&self) {
        let event = json!({
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id,
            "event": "run_started",
        });
        self.append_event(&event);
    }

    pub fn log_run_ended(&self, reason: &str) {
        let event = json!({
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id,
            "event": "run_ended",
            "reason": reason,
        });
        self.append_event(&event);
    }

    pub fn log_advice(&self, state_hash: &str, effort: Effort, prompt: &str, advice: &str) {
        let event = json!({
            "ts_ms": timestamp_ms(),
            "run_id": self.run_id,
            "event": "advice",
            "state_hash": state_hash,
            "effort": effort.as_str(),
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
            last_state_hash: None,
        }
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

    #[cfg(test)]
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
