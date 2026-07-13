use crate::runtime::RuntimeOptions;
use std::path::Path;

pub(crate) fn run_setup_if_needed(
    options: &RuntimeOptions,
    project_root: &Path,
    manual_run: bool,
) -> bool {
    if options.setup_only {
        match crate::setup_wizard::run_api_setup(project_root) {
            Ok(true) => {
                if let Err(error) = dotenvy::from_path_override(project_root.join(".env")) {
                    tracing::warn!("failed to reload .env after setup: {error}");
                }
            }
            Ok(false) => {}
            Err(error) => eprintln!("setup failed: {error}"),
        }
        return true;
    }

    if manual_run && !options.force_mock_provider && !options.postmortem_plain {
        match crate::setup_wizard::maybe_run_api_setup(project_root) {
            Ok(true) => {
                if let Err(error) = dotenvy::from_path_override(project_root.join(".env")) {
                    tracing::warn!("failed to reload .env after setup: {error}");
                }
            }
            Ok(false) => {}
            Err(error) => eprintln!("setup failed: {error}"),
        }
    }

    false
}

pub(crate) async fn run_postmortem_mode(options: &RuntimeOptions) -> bool {
    let Some(path) = options.postmortem_path.as_deref() else {
        return false;
    };

    let detected = crate::startup::detect_game_language();
    let language = detected
        .as_ref()
        .map(|detected| detected.value.as_str())
        .unwrap_or("en");
    let locale_key = crate::locales::lang_to_locale_key(language);
    let postmortem_locale = crate::locales::Locale::load(locale_key);

    let deterministic_report = match tokio::fs::read_to_string(path)
        .await
        .map_err(|error| error.to_string())
        .and_then(|content| {
            crate::postmortem::generate_report_from_jsonl(&content, &postmortem_locale)
        }) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("failed to generate postmortem: {error}");
            return true;
        }
    };

    if options.postmortem_plain {
        println!("{deterministic_report}");
        return true;
    }

    let config = crate::config::Config::from_env();
    match crate::llm::LlmProvider::from_config(&config) {
        Ok(provider) => {
            let outcome =
                if deterministic_report.contains(&postmortem_locale.postmortem.label_victory) {
                    "Victory"
                } else {
                    "Defeated"
                };
            let prompt = crate::postmortem::build_ai_postmortem_prompt(
                &deterministic_report,
                &postmortem_locale,
                outcome,
            );
            match provider.query_postmortem(&prompt, &postmortem_locale).await {
                Ok(report) => {
                    let combined = crate::postmortem::combine_postmortem_report(
                        &report,
                        &deterministic_report,
                        &postmortem_locale.postmortem.section_machine,
                    );
                    println!("{combined}");
                }
                Err(error) => {
                    eprintln!("AI postmortem failed, falling back to plain report: {error}");
                    println!("{deterministic_report}");
                }
            }
        }
        Err(error) => {
            eprintln!("AI postmortem unavailable, falling back to plain report: {error}");
            println!("{deterministic_report}");
        }
    }
    true
}
