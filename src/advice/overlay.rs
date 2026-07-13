use super::response::AdviceFields;
use crate::state::ScreenType;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
pub struct OverlayOutput {
    pub schema_version: u32,
    pub status: String,
    pub overlay_visibility: bool,
    pub advice: AdviceFields,
    pub screen_type: Option<ScreenType>,
    pub scenario: String,
    pub in_combat: bool,
    pub state_hash: String,
    pub floor: Option<i64>,
    pub character: Option<String>,
    pub timestamp_ms: u128,
}

pub struct OverlayMetadata {
    pub screen_type: Option<ScreenType>,
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

pub(crate) fn atomic_write_json(path: &Path, json: &str) {
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

pub(crate) fn write_overlay_json_to(path: &Path, output: &OverlayOutput) {
    if let Ok(json) = serde_json::to_string_pretty(output) {
        let _lock = OVERLAY_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        atomic_write_json(path, &json);
    }
}
