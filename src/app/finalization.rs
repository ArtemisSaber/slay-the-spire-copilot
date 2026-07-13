pub(crate) async fn finalize_run_once(
    journal: &crate::journal::Journal,
    provider: &crate::llm::LlmProvider,
    reason: &str,
    finalized: &mut bool,
    locale: &crate::locales::Locale,
) {
    if *finalized {
        return;
    }
    *finalized = true;

    let Some(journal_path) = journal.path() else {
        tracing::debug!("journal never confirmed, skipping postmortem");
        return;
    };

    journal.log_run_ended(reason);

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
    let prompt =
        crate::postmortem::build_ai_postmortem_prompt(&deterministic_report, locale, outcome);
    match provider.query_postmortem(&prompt, locale).await {
        Ok(ai_report) => {
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
