use anyhow::{Context, bail};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

const MAX_LISTED_RUNS: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewableRun {
    pub run_id: String,
    pub case_count: usize,
    pub lesson_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewResult {
    pub response_valid: bool,
    pub accepted_lessons: usize,
    pub rejected_lessons: usize,
    pub api_calls: usize,
    pub outcome: crate::learning::deliberation::DeliberationOutcome,
    pub report_path: PathBuf,
}

pub(crate) fn list_reviewable_runs(project_root: &Path) -> anyhow::Result<Vec<ReviewableRun>> {
    let bundle = crate::learning::bundle::embedded_bundle()
        .map_err(|error| anyhow::anyhow!("embedded knowledge bundle invalid: {error:?}"))?;
    let store = crate::learning::store::KnowledgeStore::new(
        project_root.join("learning").join("knowledge"),
    );
    let snapshot = store
        .load_snapshot()
        .or_else(|_| store.rebuild(std::slice::from_ref(&bundle)))?;
    let mut cases_by_run = BTreeMap::<String, Vec<String>>::new();
    let mut case_run = HashMap::new();
    for case in &snapshot.cases {
        cases_by_run
            .entry(case.run_id.clone())
            .or_default()
            .push(case.case_id.clone());
        case_run.insert(case.case_id.as_str(), case.run_id.as_str());
    }
    let mut lessons_by_run = HashMap::<&str, HashSet<&str>>::new();
    for lesson in &snapshot.lessons {
        for run_id in lesson
            .source_case_ids
            .iter()
            .filter_map(|case_id| case_run.get(case_id.as_str()).copied())
        {
            lessons_by_run
                .entry(run_id)
                .or_default()
                .insert(lesson.lesson_id.as_str());
        }
    }
    let mut runs: Vec<_> = cases_by_run
        .into_iter()
        .filter(|(run_id, _)| journal_path(project_root, run_id).is_some_and(|path| path.is_file()))
        .map(|(run_id, case_ids)| ReviewableRun {
            lesson_count: lessons_by_run.get(run_id.as_str()).map_or(0, HashSet::len),
            run_id,
            case_count: case_ids.len(),
        })
        .collect();
    runs.sort_by(|left, right| right.run_id.cmp(&left.run_id));
    runs.truncate(MAX_LISTED_RUNS);
    Ok(runs)
}

pub(crate) async fn review_run_with_provider(
    project_root: &Path,
    config: &crate::config::Config,
    provider: &crate::llm::LlmProvider,
    locale: &crate::locales::Locale,
    locale_key: &str,
    run_id: &str,
) -> anyhow::Result<ReviewResult> {
    if !config.memory.captures() {
        bail!("local learning is off; enable it in Settings first");
    }
    let journal_path = journal_path(project_root, run_id)
        .filter(|path| path.is_file())
        .with_context(|| format!("completed run was not found: {run_id}"))?;
    let deterministic_report =
        crate::postmortem::generate_report_from_journal_file(&journal_path, locale)
            .map_err(anyhow::Error::msg)?;
    let outcome = if deterministic_report.contains(&locale.postmortem.label_victory) {
        "Victory"
    } else {
        "Defeated"
    };
    let base_prompt =
        crate::postmortem::build_ai_postmortem_prompt(&deterministic_report, locale, outcome);
    let mut learning =
        crate::learning::session::bootstrap_session(project_root, config, locale_key, false)?;
    let deliberation = crate::learning::deliberation::deliberate_lesson(
        &mut learning,
        provider,
        locale,
        &base_prompt,
        run_id,
    )
    .await?;
    let ingest = &deliberation.ingest;
    let report = if ingest.response_valid {
        crate::postmortem::combine_postmortem_report(
            &ingest.report_markdown,
            &deterministic_report,
            &locale.postmortem.section_machine,
        )
    } else {
        deterministic_report
    };
    let report_path = crate::postmortem::write_report_for_journal(&journal_path, &report)
        .map_err(anyhow::Error::msg)?;
    append_review_event(&journal_path, run_id, &deliberation);
    Ok(ReviewResult {
        response_valid: ingest.response_valid,
        accepted_lessons: ingest.accepted_lessons,
        rejected_lessons: ingest.rejected_lessons,
        api_calls: deliberation.api_calls,
        outcome: deliberation.outcome,
        report_path,
    })
}

pub(crate) async fn review_run(project_root: &Path, run_id: &str) -> anyhow::Result<ReviewResult> {
    let config = crate::config::Config::from_env();
    let provider = crate::llm::LlmProvider::from_config(&config)?;
    let detected = crate::startup::detect_game_language();
    let language = detected
        .as_ref()
        .map(|detected| detected.value.as_str())
        .unwrap_or("en");
    let locale_key = crate::locales::lang_to_locale_key(language);
    let locale = crate::locales::Locale::load(locale_key);
    review_run_with_provider(
        project_root,
        &config,
        &provider,
        &locale,
        locale_key,
        run_id,
    )
    .await
}

fn journal_path(project_root: &Path, run_id: &str) -> Option<PathBuf> {
    let mut components = Path::new(run_id).components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        return None;
    }
    Some(project_root.join("runs").join(run_id).join("events.jsonl"))
}

fn append_review_event(
    journal_path: &Path,
    run_id: &str,
    deliberation: &crate::learning::deliberation::DeliberationResult,
) {
    let ingest = &deliberation.ingest;
    let event = serde_json::json!({
        "schema_version": 1,
        "ts_ms": crate::journal::timestamp_ms(),
        "run_id": run_id,
        "event": "learning_critic_reviewed",
        "response_valid": ingest.response_valid,
        "accepted_lessons": ingest.accepted_lessons,
        "rejected_lessons": ingest.rejected_lessons,
        "snapshot_id": ingest.snapshot_id,
        "deliberation_outcome": deliberation.outcome.as_str(),
        "api_calls": deliberation.api_calls,
    });
    let Ok(mut line) = serde_json::to_vec(&event) else {
        return;
    };
    line.push(b'\n');
    if let Err(error) = crate::logging::append_locked(journal_path, &line) {
        tracing::warn!("failed to append terminal review event: {error}");
    }
}
