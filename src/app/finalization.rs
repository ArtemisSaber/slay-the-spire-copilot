#[cfg(test)]
pub(crate) async fn finalize_run_once(
    journal: &crate::journal::Journal,
    provider: &crate::llm::LlmProvider,
    reason: &str,
    finalized: &mut bool,
    locale: &crate::locales::Locale,
) {
    finalize(journal, provider, reason, finalized, locale, None).await;
}

pub(crate) async fn finalize_run_once_with_learning(
    journal: &crate::journal::Journal,
    provider: &crate::llm::LlmProvider,
    reason: &str,
    finalized: &mut bool,
    locale: &crate::locales::Locale,
    learning: &mut crate::learning::session::LearningSession,
) {
    finalize(journal, provider, reason, finalized, locale, Some(learning)).await;
}

async fn finalize(
    journal: &crate::journal::Journal,
    provider: &crate::llm::LlmProvider,
    reason: &str,
    finalized: &mut bool,
    locale: &crate::locales::Locale,
    learning: Option<&mut crate::learning::session::LearningSession>,
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
                    "snapshot_id": summary.snapshot_id,
                }));
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
    let prompt = if critic_enabled {
        learning
            .as_deref()
            .and_then(|learning| learning.build_critic_prompt(&base_prompt, journal.run_id()))
            .unwrap_or(base_prompt)
    } else {
        base_prompt
    };
    match provider.query_postmortem(&prompt, locale).await {
        Ok(response) => {
            let ai_report = if critic_enabled {
                match learning
                    .map(|learning| learning.ingest_critic_response(&response, journal.run_id()))
                {
                    Some(Ok(result)) => {
                        journal.log_learning_event(&serde_json::json!({
                            "schema_version": 1,
                            "event": "learning_critic_ingested",
                            "accepted_lessons": result.accepted_lessons,
                            "rejected_lessons": result.rejected_lessons,
                            "snapshot_id": result.snapshot_id,
                        }));
                        result.report_markdown
                    }
                    Some(Err(error)) => {
                        tracing::error!("failed to ingest learning critic response: {error}");
                        response
                    }
                    None => response,
                }
            } else {
                response
            };
            let combined = crate::postmortem::combine_postmortem_report(
                &ai_report,
                &deterministic_report,
                &locale.postmortem.section_machine,
            );
            match crate::postmortem::write_report_for_journal(journal_path, &combined) {
                Ok(path) => tracing::info!("wrote AI postmortem report to {}", path.display()),
                Err(error) => tracing::error!("failed to write combined postmortem: {error}"),
            }
        }
        Err(error) => {
            tracing::warn!("AI postmortem failed, deterministic report saved: {error}");
        }
    }
}
