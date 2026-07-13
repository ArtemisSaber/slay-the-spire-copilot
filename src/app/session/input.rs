pub(super) fn parse_input_line(line: &str) -> Option<serde_json::Value> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    crate::logging::log_raw_input(trimmed);
    tracing::debug!("received {} bytes", trimmed.len());
    if trimmed.len() > crate::MAX_STDIN_JSON_BYTES {
        tracing::error!(
            "stdin JSON too large: {} bytes (max {}), skipping",
            trimmed.len(),
            crate::MAX_STDIN_JSON_BYTES,
        );
        return None;
    }

    match serde_json::from_str(trimmed) {
        Ok(raw) => Some(raw),
        Err(error) => {
            tracing::error!("failed to parse JSON: {error}");
            None
        }
    }
}
