use anyhow::{Context, bail};
use serde_json::json;
use std::path::{Path, PathBuf};

use crate::learning::bundle::{KnowledgeBundle, embedded_bundle};
use crate::learning::lesson::{LessonEvent, LessonEventKind, LessonStatus};
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::status::{LastRunStatus, LearningStatus, render_human};
use crate::learning::store::KnowledgeStore;

pub fn execute_command(project_root: &Path, args: &[String]) -> anyhow::Result<String> {
    let store = KnowledgeStore::new(project_root.join("learning").join("knowledge"));
    let bundle = embedded_bundle()
        .map_err(|error| anyhow::anyhow!("embedded knowledge bundle invalid: {error:?}"))?;
    match args.first().map(String::as_str).unwrap_or("status") {
        "status" => status(&store, &bundle, args),
        "verify" => verify(&store, &bundle),
        "inspect" => inspect(&store, &bundle, required(args, 1, "memory ID")?),
        "rebuild" => rebuild(&store, &bundle),
        "export-bundle" => {
            let output = export_path(project_root, args.get(1).map(String::as_str))?;
            export(&store, &bundle, &output)
        }
        "validate" => transition(&store, &bundle, args, LessonStatus::Validated),
        "contest" => transition(&store, &bundle, args, LessonStatus::Contested),
        "retire" => transition(&store, &bundle, args, LessonStatus::Retired),
        command => bail!(
            "unknown knowledge command {command}; expected status, verify, inspect, rebuild, export-bundle, validate, contest, or retire"
        ),
    }
}

fn status(
    store: &KnowledgeStore,
    bundle: &KnowledgeBundle,
    args: &[String],
) -> anyhow::Result<String> {
    let snapshot = load_or_rebuild(store, bundle)?;
    if args.get(1).map(String::as_str) == Some("--json") {
        return snapshot_summary(&snapshot);
    }
    let mode = active_memory_mode(store)?;
    let last_run = store
        .read_last_run_eligibility()?
        .as_ref()
        .map(LastRunStatus::from);
    Ok(render_human(&LearningStatus {
        mode,
        case_count: snapshot.cases.len(),
        lesson_count: snapshot.lessons.len(),
        last_run,
    }))
}

fn active_memory_mode(
    store: &KnowledgeStore,
) -> anyhow::Result<crate::learning::config::MemoryMode> {
    if let Ok(value) = std::env::var("MEMORY_MODE") {
        let config = crate::learning::config::MemoryConfig::from_lookup(|key| {
            (key == "MEMORY_MODE").then(|| value.clone())
        });
        return Ok(config.mode);
    }
    Ok(store.read_recorded_mode()?.unwrap_or_default())
}

fn verify(store: &KnowledgeStore, bundle: &KnowledgeBundle) -> anyhow::Result<String> {
    let snapshot = store.rebuild(std::slice::from_ref(bundle))?;
    snapshot_summary(&snapshot)
}

fn snapshot_summary(snapshot: &KnowledgeSnapshot) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(&json!({
        "schema_version": 1,
        "snapshot_id": snapshot.snapshot_id,
        "verified": snapshot.verify(),
        "case_count": snapshot.cases.len(),
        "lesson_count": snapshot.lessons.len(),
        "bundle_ids": snapshot.bundle_ids,
    }))?)
}

fn inspect(store: &KnowledgeStore, bundle: &KnowledgeBundle, id: &str) -> anyhow::Result<String> {
    let snapshot = load_or_rebuild(store, bundle)?;
    if let Some(case) = snapshot.cases.iter().find(|case| case.case_id == id) {
        return Ok(serde_json::to_string_pretty(case)?);
    }
    if let Some(lesson) = snapshot
        .lessons
        .iter()
        .find(|lesson| lesson.lesson_id == id)
    {
        return Ok(serde_json::to_string_pretty(lesson)?);
    }
    bail!("knowledge item not found: {id}")
}

fn rebuild(store: &KnowledgeStore, bundle: &KnowledgeBundle) -> anyhow::Result<String> {
    let snapshot = store.rebuild(std::slice::from_ref(bundle))?;
    Ok(serde_json::to_string_pretty(&json!({
        "snapshot_id": snapshot.snapshot_id,
        "case_count": snapshot.cases.len(),
        "lesson_count": snapshot.lessons.len(),
    }))?)
}

fn export(
    store: &KnowledgeStore,
    bundle: &KnowledgeBundle,
    output: &Path,
) -> anyhow::Result<String> {
    let snapshot = store.rebuild(std::slice::from_ref(bundle))?;
    let exported = store.export_bundle(&snapshot, output)?;
    Ok(serde_json::to_string_pretty(&json!({
        "bundle_id": exported.bundle_id,
        "output": output,
        "case_count": exported.cases.len(),
        "lesson_count": exported.lessons.len(),
    }))?)
}

fn transition(
    store: &KnowledgeStore,
    bundle: &KnowledgeBundle,
    args: &[String],
    status: LessonStatus,
) -> anyhow::Result<String> {
    let lesson_id = required(args, 1, "lesson ID")?;
    let reason_parts = args.get(2..).unwrap_or_default();
    let reason_parts = if reason_parts.first().map(String::as_str) == Some("--reason") {
        &reason_parts[1..]
    } else {
        reason_parts
    };
    let reason = reason_parts.join(" ");
    if reason.trim().is_empty() {
        bail!("a review reason is required");
    }
    let snapshot = load_or_rebuild(store, bundle)?;
    let mut lesson = snapshot
        .lessons
        .iter()
        .find(|lesson| lesson.lesson_id == lesson_id)
        .cloned()
        .with_context(|| format!("lesson not found: {lesson_id}"))?;
    lesson
        .set_human_status(status)
        .map_err(|error| anyhow::anyhow!("invalid lesson transition: {error:?}"))?;
    let kind = match status {
        LessonStatus::Validated => LessonEventKind::Validated,
        LessonStatus::Contested => LessonEventKind::Contested,
        LessonStatus::Retired => LessonEventKind::Retired,
        LessonStatus::Proposed | LessonStatus::Supported => unreachable!("human status checked"),
    };
    let event = LessonEvent::new(kind, lesson, Some(reason), crate::journal::timestamp_ms())
        .map_err(|error| anyhow::anyhow!("invalid lesson event: {error:?}"))?;
    store.append_lesson_events(&[event])?;
    rebuild(store, bundle)
}

fn load_or_rebuild(
    store: &KnowledgeStore,
    bundle: &KnowledgeBundle,
) -> anyhow::Result<KnowledgeSnapshot> {
    store
        .load_snapshot()
        .or_else(|_| store.rebuild(std::slice::from_ref(bundle)))
}

fn required<'a>(args: &'a [String], index: usize, label: &str) -> anyhow::Result<&'a str> {
    args.get(index)
        .map(String::as_str)
        .with_context(|| format!("{label} is required"))
}

fn export_path(project_root: &Path, value: Option<&str>) -> anyhow::Result<PathBuf> {
    let requested = value
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("knowledge/bundled-export.json"));
    if requested
        .components()
        .any(|component| component == std::path::Component::ParentDir)
    {
        bail!("bundle output must remain under the project root");
    }
    let output = if requested.is_absolute() {
        requested
    } else {
        project_root.join(requested)
    };
    if !output.starts_with(project_root) {
        bail!("bundle output must remain under the project root");
    }
    Ok(output)
}
