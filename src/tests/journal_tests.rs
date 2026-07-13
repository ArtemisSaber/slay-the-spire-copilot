use super::support::format_datetime;
use super::*;
use crate::llm::AdviceScenario;
use crate::test_utils::{load_fixture, test_locale};
use chrono::{TimeZone, Utc};
use serde_json::Value;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

const JOURNAL_LOCK_CHILD_ROOT: &str = "STS_COPILOT_JOURNAL_LOCK_CHILD_ROOT";
const JOURNAL_LOCK_CHILD_READY: &str = "STS_COPILOT_JOURNAL_LOCK_CHILD_READY";

fn wait_for_ready(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("child did not signal readiness: {}", path.display());
}

fn assert_child_waits_while_locked(child: &mut Child) {
    thread::sleep(Duration::from_millis(150));
    assert!(
        child.try_wait().unwrap().is_none(),
        "child append completed while parent lock was held"
    );
}

fn read_events(path: &Path) -> Vec<Value> {
    let content = std::fs::read_to_string(path).unwrap();
    content
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn journal_append_waits_for_cross_process_lock() {
    if let Ok(root) = std::env::var(JOURNAL_LOCK_CHILD_ROOT) {
        let ready = std::env::var(JOURNAL_LOCK_CHILD_READY).unwrap();
        let journal = Journal::new_at(root, "lock-test");
        std::fs::write(ready, "ready").unwrap();
        journal.log_run_started();
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let run_dir = dir.path().join("lock-test");
    std::fs::create_dir_all(&run_dir).unwrap();
    let journal_path = run_dir.join("events.jsonl");
    let ready_path = dir.path().join("child-ready");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal_path)
        .unwrap();
    fs4::FileExt::lock(&file).unwrap();

    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("journal::tests::journal_append_waits_for_cross_process_lock")
        .arg("--exact")
        .env(JOURNAL_LOCK_CHILD_ROOT, dir.path())
        .env(JOURNAL_LOCK_CHILD_READY, &ready_path)
        .spawn()
        .unwrap();

    wait_for_ready(&ready_path);
    assert_child_waits_while_locked(&mut child);
    fs4::FileExt::unlock(&file).unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "child test failed: {status}");

    let events = read_events(&journal_path);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "run_started");
}

#[path = "journal_tests/confirmation.rs"]
mod confirmation;
#[path = "journal_tests/events.rs"]
mod events;
#[path = "journal_tests/metadata.rs"]
mod metadata;
