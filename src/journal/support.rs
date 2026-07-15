use chrono::{DateTime, Local};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn scan_for_existing_run(runs_root: &Path, seed: i64) -> Option<PathBuf> {
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

pub(super) fn format_run_id(ts_ms: u128, character_display: &str, ascension: i64) -> String {
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

pub(super) fn learning_event(event: &serde_json::Value, run_id: &str) -> Option<serde_json::Value> {
    let mut object = event.as_object()?.clone();
    object.insert("ts_ms".into(), serde_json::json!(timestamp_ms()));
    object.insert("run_id".into(), serde_json::json!(run_id));
    Some(serde_json::Value::Object(object))
}
