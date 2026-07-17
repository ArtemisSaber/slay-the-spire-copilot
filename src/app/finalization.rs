#[cfg(test)]
pub(crate) async fn finalize_run_once(
    journal: &crate::journal::Journal,
    provider: &crate::llm::LlmProvider,
    reason: &str,
    finalized: &mut bool,
    locale: &crate::locales::Locale,
) {
    finalize(journal, provider, reason, finalized, locale, None, None).await;
}

pub(crate) async fn finalize_run_once_with_learning(
    journal: &crate::journal::Journal,
    provider: &crate::llm::LlmProvider,
    reason: &str,
    finalized: &mut bool,
    locale: &crate::locales::Locale,
    learning: &mut crate::learning::session::LearningSession,
    overlay_path: &std::path::Path,
) {
    finalize(
        journal,
        provider,
        reason,
        finalized,
        locale,
        Some(learning),
        Some(overlay_path),
    )
    .await;
}

async fn finalize(
    journal: &crate::journal::Journal,
    provider: &crate::llm::LlmProvider,
    reason: &str,
    finalized: &mut bool,
    locale: &crate::locales::Locale,
    learning: Option<&mut crate::learning::session::LearningSession>,
    overlay_path: Option<&std::path::Path>,
) {
    let mut learning = learning;
    if *finalized {
        return;
    }
    *finalized = true;

    let Some(journal_path) = journal.path() else {
        tracing::debug!("journal never confirmed, skipping postmortem");
        return;
    };

    journal.log_run_ended(reason);
    let mut critic_enabled = false;
    if let Some(learning) = learning
        .as_deref_mut()
        .filter(|learning| learning.is_enabled())
    {
        match learning.finalize(reason) {
            Ok(summary) => {
                critic_enabled = summary.eligibility.knowledge_eligible;
                journal.log_learning_event(&serde_json::json!({
                    "schema_version": 1,
                    "event": "learning_run_finalized",
                    "eligibility": summary.eligibility,
                    "appended_cases": summary.appended_cases,
                    "skipped_cases": summary.skipped_cases,
                    "evaluated_lessons": summary.evaluated_lessons,
                    "retired_lessons": summary.retired_lessons,
                    "snapshot_id": summary.snapshot_id,
                }));
                if let Some(path) = overlay_path {
                    crate::advice::write_overlay_learning(path, &learning.status());
                }
            }
            Err(error) => tracing::error!("failed to finalize learning run: {error}"),
        }
    }

    let deterministic_report =
        match crate::postmortem::generate_report_from_journal_file(journal_path, locale) {
            Ok(report) => report,
            Err(error) => {
                tracing::error!("failed to generate postmortem report: {error}");
                return;
            }
        };

    if let Err(error) =
        crate::postmortem::write_report_for_journal(journal_path, &deterministic_report)
    {
        tracing::error!("failed to write deterministic postmortem: {error}");
        return;
    }
    tracing::info!(
        "wrote deterministic postmortem to {}",
        crate::postmortem::postmortem_path_for_journal(journal_path).display(),
    );

    let outcome = if deterministic_report.contains(&locale.postmortem.label_victory) {
        "Victory"
    } else {
        "Defeated"
    };
    let base_prompt =
        crate::postmortem::build_ai_postmortem_prompt(&deterministic_report, locale, outcome);
    if critic_enabled {
        let Some(learning) = learning else {
            tracing::error!("learning critic was enabled without a learning session");
            return;
        };
        match crate::learning::deliberation::deliberate_lesson(
            learning,
            provider,
            locale,
            &base_prompt,
            journal.run_id(),
        )
        .await
        {
            Ok(deliberation) => {
                let ingest = &deliberation.ingest;
                journal.log_learning_event(&serde_json::json!({
                    "schema_version": 1,
                    "event": "learning_critic_ingested",
                    "response_valid": ingest.response_valid,
                    "accepted_lessons": ingest.accepted_lessons,
                    "rejected_lessons": ingest.rejected_lessons,
                    "snapshot_id": ingest.snapshot_id,
                    "deliberation_outcome": deliberation.outcome.as_str(),
                    "api_calls": deliberation.api_calls,
                }));
                if let Some(path) = overlay_path {
                    crate::advice::write_overlay_learning(path, &learning.status());
                }
                if ingest.response_valid {
                    write_ai_report(
                        journal_path,
                        &ingest.report_markdown,
                        &deterministic_report,
                        locale,
                    );
                } else {
                    tracing::warn!(
                        "lesson deliberation ended {} after {} calls; deterministic report saved",
                        deliberation.outcome.as_str(),
                        deliberation.api_calls,
                    );
                }
            }
            Err(error) => tracing::error!("failed to commit reviewed lesson: {error}"),
        }
        return;
    }
    match provider.query_postmortem(&base_prompt, locale).await {
        Ok(response) => write_ai_report(journal_path, &response, &deterministic_report, locale),
        Err(error) => tracing::warn!("AI postmortem failed, deterministic report saved: {error}"),
    }
}

fn write_ai_report(
    journal_path: &std::path::Path,
    ai_report: &str,
    deterministic_report: &str,
    locale: &crate::locales::Locale,
) {
    let combined = crate::postmortem::combine_postmortem_report(
        ai_report,
        deterministic_report,
        &locale.postmortem.section_machine,
    );
    match crate::postmortem::write_report_for_journal(journal_path, &combined) {
        Ok(path) => tracing::info!("wrote AI postmortem report to {}", path.display()),
        Err(error) => tracing::error!("failed to write combined postmortem: {error}"),
    }
}
