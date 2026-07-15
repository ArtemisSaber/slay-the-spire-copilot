use crate::learning::bundle::KnowledgeBundle;
use crate::learning::case::DecisionCase;
use crate::learning::config::MemoryMode;
use crate::learning::eligibility::Eligibility;
use crate::learning::lesson::{Lesson, LessonEvent};
use crate::learning::snapshot::KnowledgeSnapshot;
use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

mod support;
use support::{atomic_write, read_cases, read_lesson_events};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppendSummary {
    pub appended: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone)]
pub struct KnowledgeStore {
    root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    snapshot_id: String,
    snapshot_file: String,
    case_count: usize,
    lesson_count: usize,
    bundle_ids: Vec<String>,
}

impl KnowledgeStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn append_cases(&self, cases: &[DecisionCase]) -> anyhow::Result<AppendSummary> {
        let existing = read_cases(&self.cases_path())?;
        let mut ids: HashSet<_> = existing.into_iter().map(|case| case.case_id).collect();
        let mut batch = String::new();
        let mut appended = 0;
        let mut skipped = 0;
        for case in cases {
            if !case.verify_id() {
                bail!("refusing to append case with invalid identity");
            }
            if !ids.insert(case.case_id.clone()) {
                skipped += 1;
                continue;
            }
            batch.push_str(&serde_json::to_string(case)?);
            batch.push('\n');
            appended += 1;
        }
        if !batch.is_empty() {
            fs::create_dir_all(&self.root)?;
            crate::logging::append_locked(&self.cases_path(), batch.as_bytes())?;
        }
        Ok(AppendSummary { appended, skipped })
    }

    pub fn append_lesson_events(&self, events: &[LessonEvent]) -> anyhow::Result<AppendSummary> {
        let existing = read_lesson_events(&self.lessons_path())?;
        let mut ids: HashSet<_> = existing.into_iter().map(|event| event.event_id).collect();
        let mut batch = String::new();
        let mut appended = 0;
        let mut skipped = 0;
        for event in events {
            if !event.verify() {
                bail!("refusing to append lesson event with invalid identity");
            }
            if !ids.insert(event.event_id.clone()) {
                skipped += 1;
                continue;
            }
            batch.push_str(&serde_json::to_string(event)?);
            batch.push('\n');
            appended += 1;
        }
        if !batch.is_empty() {
            fs::create_dir_all(&self.root)?;
            crate::logging::append_locked(&self.lessons_path(), batch.as_bytes())?;
        }
        Ok(AppendSummary { appended, skipped })
    }

    pub fn rebuild(&self, bundles: &[KnowledgeBundle]) -> anyhow::Result<KnowledgeSnapshot> {
        let cases = read_cases(&self.cases_path())?;
        let mut lessons = BTreeMap::<String, Lesson>::new();
        for event in read_lesson_events(&self.lessons_path())? {
            lessons.insert(event.lesson.lesson_id.clone(), event.lesson);
        }
        let snapshot =
            KnowledgeSnapshot::build_with_lessons(cases, lessons.into_values().collect(), bundles)
                .map_err(|error| anyhow::anyhow!("snapshot build failed: {error:?}"))?;
        fs::create_dir_all(self.root.join("snapshots"))?;
        let encoded = serde_json::to_vec_pretty(&snapshot)?;
        atomic_write(
            &self.root.join("snapshots").join(snapshot.file_name()),
            &encoded,
        )?;
        atomic_write(&self.root.join("index-v1.json"), &encoded)?;
        let manifest = Manifest {
            schema_version: 1,
            snapshot_id: snapshot.snapshot_id.clone(),
            snapshot_file: snapshot.file_name(),
            case_count: snapshot.cases.len(),
            lesson_count: snapshot.lessons.len(),
            bundle_ids: snapshot.bundle_ids.clone(),
        };
        atomic_write(
            &self.root.join("manifest.json"),
            &serde_json::to_vec_pretty(&manifest)?,
        )?;
        Ok(snapshot)
    }

    pub fn load_snapshot(&self) -> anyhow::Result<KnowledgeSnapshot> {
        let manifest: Manifest = serde_json::from_slice(
            &fs::read(self.root.join("manifest.json")).context("knowledge manifest missing")?,
        )?;
        if manifest.schema_version != 1 {
            bail!("unsupported knowledge manifest schema");
        }
        let snapshot: KnowledgeSnapshot = serde_json::from_slice(&fs::read(
            self.root.join("snapshots").join(&manifest.snapshot_file),
        )?)?;
        if !snapshot.verify() || snapshot.snapshot_id != manifest.snapshot_id {
            bail!("knowledge snapshot failed identity verification");
        }
        Ok(snapshot)
    }

    pub fn load_snapshot_by_id(&self, snapshot_id: &str) -> anyhow::Result<KnowledgeSnapshot> {
        let digest = snapshot_id
            .strip_prefix("sha256:")
            .filter(|digest| {
                digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
            .context("invalid snapshot ID")?;
        let path = self
            .root
            .join("snapshots")
            .join(format!("snapshot-{digest}.json"));
        let snapshot: KnowledgeSnapshot = serde_json::from_slice(&fs::read(path)?)?;
        if !snapshot.verify() || snapshot.snapshot_id != snapshot_id {
            bail!("requested knowledge snapshot failed identity verification");
        }
        Ok(snapshot)
    }

    pub fn export_bundle(
        &self,
        snapshot: &KnowledgeSnapshot,
        output: &Path,
    ) -> anyhow::Result<KnowledgeBundle> {
        if !snapshot.verify() {
            bail!("cannot export an invalid snapshot");
        }
        let bundle =
            KnowledgeBundle::from_knowledge(snapshot.cases.clone(), snapshot.lessons.clone())
                .map_err(|error| anyhow::anyhow!("bundle build failed: {error:?}"))?;
        let mut bytes = serde_json::to_vec_pretty(&bundle)?;
        bytes.push(b'\n');
        atomic_write(output, &bytes)?;
        Ok(bundle)
    }

    pub fn write_status(
        &self,
        snapshot: &KnowledgeSnapshot,
        mode: MemoryMode,
        last_run_eligibility: Option<&Eligibility>,
    ) -> anyhow::Result<()> {
        if !snapshot.verify() {
            bail!("cannot write status for invalid snapshot");
        }
        let status = serde_json::json!({
            "schema_version": 1,
            "updated_at_ms": crate::journal::timestamp_ms(),
            "mode": mode,
            "snapshot_id": snapshot.snapshot_id,
            "case_count": snapshot.cases.len(),
            "lesson_count": snapshot.lessons.len(),
            "bundle_ids": snapshot.bundle_ids,
            "last_run_eligibility": last_run_eligibility,
        });
        atomic_write(
            &self.root.join("status.json"),
            &serde_json::to_vec_pretty(&status)?,
        )
    }

    fn cases_path(&self) -> PathBuf {
        self.root.join("cases.jsonl")
    }

    fn lessons_path(&self) -> PathBuf {
        self.root.join("lessons.jsonl")
    }
}
