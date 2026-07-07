use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn project_root() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn advice_output_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub(crate) fn rotate_log(log_dir: &std::path::Path, base_name: &str) {
    let log2 = log_dir.join(format!("{base_name}.2"));
    let log1 = log_dir.join(format!("{base_name}.1"));
    let log0 = log_dir.join(base_name);

    if let Err(e) = fs::remove_file(&log2)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!("failed to remove {}: {e}", log2.display());
    }
    if log1.exists()
        && let Err(e) = fs::rename(&log1, &log2)
    {
        tracing::warn!(
            "failed to rename {} -> {}: {e}",
            log1.display(),
            log2.display()
        );
    }
    if log0.exists()
        && let Err(e) = fs::rename(&log0, &log1)
    {
        tracing::warn!(
            "failed to rename {} -> {}: {e}",
            log0.display(),
            log1.display()
        );
    }
}

pub fn init() -> WorkerGuard {
    let root = project_root();
    let log_dir = root.join("logs");
    fs::create_dir_all(&log_dir).expect("failed to create logs directory");

    rotate_log(&log_dir, "sts-ai.log");
    rotate_log(&log_dir, "comm-mod-raw.log");
    rotate_log(&log_dir, "prompts.log");

    let file_appender = tracing_appender::rolling::never(&log_dir, "sts-ai.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
                .with_ansi(false)
                .with_target(false),
        )
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    guard
}

pub fn log_raw_input(line: &str) {
    log_raw_input_to(&project_root().join("logs"), line);
}

pub(crate) fn log_raw_input_to(log_dir: &std::path::Path, line: &str) {
    let path = log_dir.join("comm-mod-raw.log");
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{line}");
    }
}

#[cfg(test)]
#[path = "tests/logging_tests.rs"]
mod tests;
