use crate::learning::case::DecisionCase;
use crate::learning::lesson::LessonEvent;
use anyhow::{Context, bail};
use std::fs;
use std::io::Write;
use std::path::Path;

pub(super) fn read_cases(path: &Path) -> anyhow::Result<Vec<DecisionCase>> {
    read_jsonl(path, "case", |case: &DecisionCase| case.verify_id())
}

pub(super) fn read_lesson_events(path: &Path) -> anyhow::Result<Vec<LessonEvent>> {
    read_jsonl(path, "lesson event", LessonEvent::verify)
}

fn read_jsonl<T: serde::de::DeserializeOwned>(
    path: &Path,
    label: &str,
    verify: impl Fn(&T) -> bool,
) -> anyhow::Result<Vec<T>> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => return Err(error.into()),
    };
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            let value: T = serde_json::from_str(line)
                .with_context(|| format!("invalid {label} JSONL line {}", index + 1))?;
            if !verify(&value) {
                bail!("{label} identity mismatch at line {}", index + 1);
            }
            Ok(value)
        })
        .collect()
}

pub(super) fn atomic_write(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let parent = path.parent().context("output path has no parent")?;
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}
