pub(crate) mod finalization;
mod map;
mod modes;
mod session;

use crate::runtime::RuntimeOptions;
use std::io::{BufRead, IsTerminal, Write};

pub(crate) async fn run() {
    let _guard = match crate::logging::init() {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!("failed to initialize logging: {error}");
            std::process::exit(1);
        }
    };
    let project_root = crate::logging::project_root();
    let env_path = project_root.join(".env");
    if let Ok(Err(error)) = tokio::task::spawn_blocking(move || dotenvy::from_path(&env_path)).await
    {
        tracing::debug!(".env not loaded: {error}");
    }
    let options = RuntimeOptions::from_env_and_args();
    let manual_run = std::io::stdin().is_terminal();

    if modes::run_setup_if_needed(&options, &project_root, manual_run) {
        return;
    }
    if modes::run_postmortem_mode(&options).await {
        return;
    }

    if !options.skip_startup_check && !crate::startup::ensure_config() {
        if std::io::stdin().is_terminal() {
            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(stdout, "按 Enter 键退出...");
            let _ = stdout.flush();
            let _ = std::io::stdin().lock().read_line(&mut String::new());
        }
        return;
    }

    let mut config = crate::config::Config::from_env();
    if options.force_mock_provider {
        config.provider = "mock".to_string();
        config.base_url = None;
        config.api_key = None;
    }
    tracing::info!(
        "provider={} fast={} medium={} heavy={} max_tokens={}",
        config.provider,
        config.model_fast,
        config.model_medium,
        config.model_heavy,
        config.max_tokens_heavy,
    );

    let provider = match crate::llm::LlmProvider::from_config(&config) {
        Ok(provider) => provider,
        Err(error) => {
            tracing::error!("failed to create LLM provider: {error}");
            return;
        }
    };

    crate::protocol::send_ready();
    tracing::info!("sent ready");

    let detected = crate::startup::detect_game_language();
    let language = detected
        .as_ref()
        .map(|detected| detected.value.as_str())
        .unwrap_or("en");
    let locale_key = crate::locales::lang_to_locale_key(language);
    tracing::info!("detected language: {language} -> locale: {locale_key}");
    let locale = crate::locales::Locale::load(locale_key);

    session::GameRuntime::new(project_root, config, provider, locale)
        .run()
        .await;
}
