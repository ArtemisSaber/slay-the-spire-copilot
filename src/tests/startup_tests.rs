use super::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn temp_config(content: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    fs::write(&path, content).unwrap();
    (dir, path)
}

#[path = "startup_tests/command_values.rs"]
mod command_values;
#[path = "startup_tests/config_checks.rs"]
mod config_checks;
#[path = "startup_tests/config_write.rs"]
mod config_write;
#[path = "startup_tests/ensure_config.rs"]
mod ensure_config;
#[path = "startup_tests/environment.rs"]
mod environment;
#[path = "startup_tests/language.rs"]
mod language;
#[path = "startup_tests/language_files.rs"]
mod language_files;
#[path = "startup_tests/messages.rs"]
mod messages;
#[path = "startup_tests/properties.rs"]
mod properties;
