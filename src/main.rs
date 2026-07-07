#![deny(clippy::allow_attributes_without_reason)]
mod advice;
mod autoplay;
mod combat;
mod config;
mod gate;
mod journal;
mod llm;
mod locales;
mod logging;
mod postmortem;
mod prompt;
mod protocol;
mod ranker;
mod relic_counters;
mod runtime;
mod setup_wizard;
mod startup;
mod state;

use advice::{AdviceCache, OverlayMetadata};
use gate::{CombatTurnGate, MapGate, should_generate_advice};
use llm::{AdviceScenario, Effort};
use runtime::{
    RuntimeOptions, has_monsters, is_error, is_game_over_state, is_in_game, run_end_reason,
    should_end_run,
};
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

pub(crate) const MAX_STDIN_JSON_BYTES: usize = 10 * 1024 * 1024;
/// Truncation length for prompt previews in debug logs.
const PROMPT_LOG_PREVIEW_CHARS: usize = 200;

async fn finalize_run_once(
    journal: &journal::Journal,
    provider: &llm::LlmProvider,
    reason: &str,
    finalized: &mut bool,
    locale: &locales::Locale,
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
        match postmortem::generate_report_from_journal_file(journal_path, locale) {
            Ok(report) => report,
            Err(e) => {
                tracing::error!("failed to generate postmortem report: {e}");
                return;
            }
        };

    if let Err(e) = postmortem::write_report_for_journal(journal_path, &deterministic_report) {
        tracing::error!("failed to write deterministic postmortem: {e}");
        return;
    }
    tracing::info!(
        "wrote deterministic postmortem to {}",
        postmortem::postmortem_path_for_journal(journal_path).display(),
    );

    let outcome = if deterministic_report.contains(&locale.postmortem.label_victory) {
        "Victory"
    } else {
        "Defeated"
    };
    let prompt = postmortem::build_ai_postmortem_prompt(&deterministic_report, locale, outcome);
    match provider.query_postmortem(&prompt, locale).await {
        Ok(ai_report) => {
            let combined = postmortem::combine_postmortem_report(
                &ai_report,
                &deterministic_report,
                &locale.postmortem.section_machine,
            );
            match postmortem::write_report_for_journal(journal_path, &combined) {
                Ok(path) => tracing::info!("wrote AI postmortem report to {}", path.display()),
                Err(e) => tracing::error!("failed to write combined postmortem: {e}"),
            }
        }
        Err(e) => {
            tracing::warn!("AI postmortem failed, deterministic report saved: {e}");
        }
    }
}

fn refresh_autoplay_control(
    control_path: &Path,
    last_revision: &mut Option<u64>,
    current_control: &mut Option<autoplay::control::AutoPlayControl>,
) -> autoplay::control::ControlLoad {
    let load = autoplay::control::load_control(control_path, *last_revision);
    if let Some(control) = autoplay::action::active_control(&load) {
        *last_revision = Some(control.revision);
        *current_control = Some(control.clone());
    } else if matches!(load, autoplay::control::ControlLoad::Malformed(_)) {
        *current_control = None;
    }
    load
}

fn autoplay_load_status(load: &autoplay::control::ControlLoad) -> String {
    match load {
        autoplay::control::ControlLoad::Updated(_) => "updated".to_string(),
        autoplay::control::ControlLoad::MissingDefault(_) => "missing_default".to_string(),
        autoplay::control::ControlLoad::Stale => "stale".to_string(),
        autoplay::control::ControlLoad::Malformed(error) => format!("malformed: {error}"),
    }
}

fn autoplay_mode_name(control: Option<&autoplay::control::AutoPlayControl>) -> String {
    control
        .map(|control| format!("{:?}", control.mode))
        .unwrap_or_else(|| "disabled".to_string())
}

fn autoplay_allows_execution(control: Option<&autoplay::control::AutoPlayControl>) -> bool {
    control.is_some_and(|control| control.mode == autoplay::control::AutoPlayMode::Auto)
}

#[tokio::main]
async fn main() {
    let _guard = logging::init();
    let project_root = logging::project_root();
    let _ = dotenvy::from_path(project_root.join(".env"));
    let options = RuntimeOptions::from_env_and_args();
    let manual_run = std::io::stdin().is_terminal();

    if options.setup_only {
        match setup_wizard::run_api_setup(&project_root) {
            Ok(true) => {
                let _ = dotenvy::from_path_override(project_root.join(".env"));
            }
            Ok(false) => {}
            Err(e) => eprintln!("setup failed: {e}"),
        }
        return;
    }

    if manual_run && !options.force_mock_provider && !options.postmortem_plain {
        match setup_wizard::maybe_run_api_setup(&project_root) {
            Ok(true) => {
                let _ = dotenvy::from_path_override(project_root.join(".env"));
            }
            Ok(false) => {}
            Err(e) => eprintln!("setup failed: {e}"),
        }
    }

    if let Some(path) = options.postmortem_path.as_deref() {
        let detected = startup::detect_game_language();
        let lang = detected.as_ref().map(|d| d.value.as_str()).unwrap_or("en");
        let locale_key = locales::lang_to_locale_key(lang);
        let postmortem_locale = locales::Locale::load(locale_key);

        let deterministic_report = match std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|content| {
                postmortem::generate_report_from_jsonl(&content, &postmortem_locale)
            }) {
            Ok(report) => report,
            Err(e) => {
                eprintln!("failed to generate postmortem: {e}");
                return;
            }
        };

        if options.postmortem_plain {
            println!("{deterministic_report}");
            return;
        }

        let config = config::Config::from_env();
        match llm::LlmProvider::from_config(&config) {
            Ok(provider) => {
                let outcome =
                    if deterministic_report.contains(&postmortem_locale.postmortem.label_victory) {
                        "Victory"
                    } else {
                        "Defeated"
                    };
                let prompt = postmortem::build_ai_postmortem_prompt(
                    &deterministic_report,
                    &postmortem_locale,
                    outcome,
                );
                match provider.query_postmortem(&prompt, &postmortem_locale).await {
                    Ok(report) => {
                        let combined = postmortem::combine_postmortem_report(
                            &report,
                            &deterministic_report,
                            &postmortem_locale.postmortem.section_machine,
                        );
                        println!("{combined}");
                    }
                    Err(e) => {
                        eprintln!("AI postmortem failed, falling back to plain report: {e}");
                        println!("{deterministic_report}");
                    }
                }
            }
            Err(e) => {
                eprintln!("AI postmortem unavailable, falling back to plain report: {e}");
                println!("{deterministic_report}");
            }
        }
        return;
    }

    if !options.skip_startup_check && !startup::ensure_config() {
        if std::io::stdin().is_terminal() {
            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(stdout, "按 Enter 键退出...");
            let _ = stdout.flush();
            let _ = std::io::stdin().lock().read_line(&mut String::new());
        }
        return;
    }

    let mut config = config::Config::from_env();
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

    let provider = match llm::LlmProvider::from_config(&config) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("failed to create LLM provider: {e}");
            return;
        }
    };

    protocol::send_ready();
    tracing::info!("sent ready");

    let detected = startup::detect_game_language();
    let lang = detected.as_ref().map(|d| d.value.as_str()).unwrap_or("en");
    let locale_key = locales::lang_to_locale_key(lang);
    tracing::info!("detected language: {lang} -> locale: {locale_key}");
    let locale = locales::Locale::load(locale_key);

    let mut cache = AdviceCache::new();
    let mut journal = journal::Journal::new(project_root.join("runs"));
    let mut combat_turn_gate = CombatTurnGate::new();
    let mut map_gate = MapGate::new();
    let mut autoplay_last_revision: Option<u64> = None;
    let mut current_autoplay_control = if config.auto_play {
        Some(autoplay::control::AutoPlayControl::default_paused())
    } else {
        None
    };
    if config.auto_play {
        let initial_overlay_path = logging::advice_output_dir()
            .join("output")
            .join("overlay.json");
        autoplay::status::write_overlay_autoplay(
            &initial_overlay_path,
            &advice::OverlayMetadata {
                screen_type: None,
                scenario: String::new(),
                in_combat: false,
                state_hash: String::new(),
                floor: None,
                character: None,
            },
            &autoplay::status::AutoPlayState {
                mode: "paused".into(),
                status: "startup".into(),
            },
        );
    }
    let mut autoplay_session = autoplay::control::AutoPlaySession::default();
    let mut autoplay_overlay_state = autoplay::status::AutoPlayState::default();
    let mut last_autoplay_state: Option<(
        serde_json::Value,
        state::NormalizedState,
        autoplay::command_state::CommandState,
    )> = None;
    let mut consecutive_errors: usize = 0;
    let mut saw_game_state = false;
    let mut run_finalized = false;
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("failed to read stdin: {e}");
                continue;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        logging::log_raw_input(trimmed);
        tracing::debug!("received {} bytes", trimmed.len());

        if trimmed.len() > MAX_STDIN_JSON_BYTES {
            tracing::error!(
                "stdin JSON too large: {} bytes (max {}), skipping",
                trimmed.len(),
                MAX_STDIN_JSON_BYTES
            );
            continue;
        }

        let raw: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("failed to parse JSON: {e}");
                continue;
            }
        };

        let overlay_path = logging::advice_output_dir()
            .join("output")
            .join("overlay.json");
        let autoplay_control_path = logging::advice_output_dir()
            .join("output")
            .join("autoplay-control.json");

        if is_error(&raw) {
            tracing::warn!("received error from CommunicationMod: {}", trimmed);
            if let Some((saved_raw, saved_normalized, saved_command_state)) =
                last_autoplay_state.as_ref()
            {
                consecutive_errors += 1;
                if consecutive_errors > 5 {
                    tracing::error!(
                        "autoplay reached {} consecutive errors, blocking",
                        consecutive_errors
                    );
                    current_autoplay_control = None;
                    continue;
                }
                if let Some(control) = current_autoplay_control.as_mut() {
                    tracing::info!("autoplay retrying after error #{}", consecutive_errors);
                    match autoplay::planner::plan_action(
                        &provider,
                        control,
                        &mut autoplay_session,
                        saved_command_state,
                        saved_normalized,
                        &locale,
                        map_gate.shop_visited,
                    )
                    .await
                    {
                        Ok(Some(action)) => {
                            let control_load = refresh_autoplay_control(
                                &autoplay_control_path,
                                &mut autoplay_last_revision,
                                &mut current_autoplay_control,
                            );
                            if autoplay_allows_execution(current_autoplay_control.as_ref()) {
                                tracing::info!(
                                    "autoplay retry executing {:?} screen={}",
                                    action,
                                    saved_normalized.screen_type.as_deref().unwrap_or("?"),
                                );
                                let mut stdout = io::stdout().lock();
                                autoplay::action::execute_action_to(&mut stdout, &action);
                            } else {
                                let retry_scenario = AdviceScenario::from_state(saved_normalized);
                                autoplay::status::write_overlay_autoplay(
                                    &overlay_path,
                                    &advice::OverlayMetadata {
                                        screen_type: saved_normalized.screen_type.clone(),
                                        scenario: retry_scenario.as_str().to_string(),
                                        in_combat: has_monsters(saved_raw),
                                        state_hash: saved_normalized.stable_hash(),
                                        floor: saved_normalized.floor,
                                        character: saved_normalized.character.clone(),
                                    },
                                    &autoplay::status::AutoPlayState {
                                        mode: format!(
                                            "{:?}",
                                            current_autoplay_control
                                                .as_ref()
                                                .map(|control| control.mode)
                                                .unwrap_or(autoplay::control::AutoPlayMode::Off)
                                        )
                                        .to_lowercase(),
                                        status: "idle".into(),
                                    },
                                );
                                tracing::info!(
                                    "autoplay retry blocked before execution: control={} load={}",
                                    autoplay_mode_name(current_autoplay_control.as_ref()),
                                    autoplay_load_status(&control_load),
                                );
                            }
                        }
                        Ok(None) => {
                            tracing::warn!("autoplay retry produced no action");
                        }
                        Err(e) => {
                            tracing::warn!("autoplay retry planner failed: {e}");
                        }
                    }
                }
            }
            continue;
        }

        if !is_in_game(&raw) {
            if should_end_run(&raw, saw_game_state) {
                let reason = run_end_reason(&raw, saw_game_state).unwrap_or("left_game");
                finalize_run_once(&journal, &provider, reason, &mut run_finalized, &locale).await;
                if config.auto_play {
                    autoplay::status::write_overlay_autoplay(
                        &overlay_path,
                        &advice::OverlayMetadata {
                            screen_type: None,
                            scenario: String::new(),
                            in_combat: false,
                            state_hash: String::new(),
                            floor: None,
                            character: None,
                        },
                        &autoplay::status::AutoPlayState {
                            mode: "off".into(),
                            status: "stopped".into(),
                        },
                    );
                }
                break;
            }
            tracing::debug!("skipping non-game state");
            continue;
        }

        saw_game_state = true;
        consecutive_errors = 0;

        let screen_type = raw
            .pointer("/game_state/screen_type")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let room_type = raw
            .pointer("/game_state/room_type")
            .and_then(|v| v.as_str())
            .unwrap_or("?");

        let normalized = state::NormalizedState::from_raw(&raw, &locale);
        let hash = normalized.stable_hash();
        let command_state = autoplay::command_state::CommandState::from_raw(&raw);
        let scenario = AdviceScenario::from_state(&normalized);
        let metadata = OverlayMetadata {
            screen_type: Some(screen_type.to_string()),
            scenario: scenario.as_str().to_string(),
            in_combat: has_monsters(&raw),
            state_hash: hash.clone(),
            floor: normalized.floor,
            character: normalized.character.clone(),
        };

        if config.auto_play {
            let autoplay_control_load = refresh_autoplay_control(
                &autoplay_control_path,
                &mut autoplay_last_revision,
                &mut current_autoplay_control,
            );
            let autoplay_load_status = autoplay_load_status(&autoplay_control_load);
            let autoplay_mode = autoplay_mode_name(current_autoplay_control.as_ref());
            tracing::debug!(
                "autoplay control={autoplay_mode} load={autoplay_load_status} ready={} commands={} choose_available={}",
                command_state.ready_for_command,
                command_state.available_commands.len(),
                command_state.has_command("choose"),
            );
        }

        if !journal.is_confirmed()
            && let (Some(seed), Some(character)) = (normalized.seed, normalized.character.as_ref())
        {
            journal.confirm(
                seed,
                character,
                normalized.ascension_level.unwrap_or(0),
                &config,
                &locale,
            );
        }
        if !journal.is_confirmed() {
            continue;
        }

        journal.log_state_change(&hash, &normalized);

        if let Some(control) = current_autoplay_control.as_mut() {
            autoplay_overlay_state.mode = format!("{:?}", control.mode).to_lowercase();
            autoplay_overlay_state.status = "planning".into();
            autoplay::status::write_overlay_autoplay(
                &overlay_path,
                &metadata,
                &autoplay_overlay_state,
            );

            match autoplay::planner::plan_action(
                &provider,
                control,
                &mut autoplay_session,
                &command_state,
                &normalized,
                &locale,
                map_gate.shop_visited,
            )
            .await
            {
                Ok(Some(action)) => {
                    let control_load = refresh_autoplay_control(
                        &autoplay_control_path,
                        &mut autoplay_last_revision,
                        &mut current_autoplay_control,
                    );
                    if autoplay_allows_execution(current_autoplay_control.as_ref()) {
                        autoplay_overlay_state.status = "executing".into();
                        autoplay::status::write_overlay_autoplay(
                            &overlay_path,
                            &metadata,
                            &autoplay_overlay_state,
                        );
                        tracing::info!(
                            "autoplay executing {:?} screen={} hash={}",
                            action,
                            screen_type,
                            &hash[..16],
                        );
                        last_autoplay_state =
                            Some((raw.clone(), normalized.clone(), command_state.clone()));
                        let mut stdout = io::stdout().lock();
                        autoplay::action::execute_action_to(&mut stdout, &action);
                        continue;
                    }

                    autoplay_overlay_state.mode = format!(
                        "{:?}",
                        current_autoplay_control
                            .as_ref()
                            .map(|control| control.mode)
                            .unwrap_or(autoplay::control::AutoPlayMode::Off)
                    )
                    .to_lowercase();
                    autoplay_overlay_state.status = "idle".into();
                    autoplay::status::write_overlay_autoplay(
                        &overlay_path,
                        &metadata,
                        &autoplay_overlay_state,
                    );
                    tracing::info!(
                        "autoplay blocked before execution: control={} load={}",
                        autoplay_mode_name(current_autoplay_control.as_ref()),
                        autoplay_load_status(&control_load),
                    );
                }
                Ok(None) => {
                    autoplay_overlay_state.status = "idle".into();
                    autoplay::status::write_overlay_autoplay(
                        &overlay_path,
                        &metadata,
                        &autoplay_overlay_state,
                    );
                }
                Err(e) => {
                    autoplay_overlay_state.status = "error".into();
                    autoplay::status::write_overlay_autoplay(
                        &overlay_path,
                        &metadata,
                        &autoplay_overlay_state,
                    );
                    tracing::warn!("autoplay planner did not produce an executable action: {e}");
                }
            }
        }

        if screen_type == "MAP" {
            let rp = raw
                .pointer("/game_state/room_phase")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            if rp == "COMPLETE" {
                let first_chosen = raw
                    .pointer("/game_state/screen_state/first_node_chosen")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                if first_chosen {
                    let paths = raw
                        .pointer("/game_state/screen_state/current_node")
                        .and_then(|v| Some((v.get("x")?.as_i64()?, v.get("y")?.as_i64()?)))
                        .map(|(x, y)| prompt::enumerate_paths(x, y, &normalized.map_nodes))
                        .unwrap_or_default();
                    tracing::info!(
                        "MAP: {} paths from current_node, room_phase=COMPLETE",
                        paths.len(),
                    );
                    for (i, path) in paths.iter().enumerate() {
                        let route: Vec<String> = path
                            .iter()
                            .map(|n| format!("{}({},{})", n.symbol, n.x, n.y))
                            .collect();
                        let summary = prompt::summarize_path(path);
                        tracing::info!(
                            "  Path {}: {}  [{}]",
                            (b'A' + i as u8) as char,
                            route.join(" → "),
                            summary,
                        );
                    }
                } else {
                    let roots = prompt::enumerate_paths_from_roots(&normalized.map_nodes);
                    let total: usize = roots.iter().map(|r| r.paths.len()).sum();
                    tracing::info!(
                        "MAP: {} roots, {} paths total, room_phase=COMPLETE",
                        roots.len(),
                        total,
                    );
                    for (ri, root_group) in roots.iter().enumerate() {
                        let root_label = format!(
                            "{}({},{})",
                            root_group.root.symbol, root_group.root.x, root_group.root.y
                        );
                        tracing::info!(
                            "  Root {} {}: {} paths",
                            (b'A' + ri as u8) as char,
                            root_label,
                            root_group.paths.len(),
                        );
                        for (pi, path) in root_group.paths.iter().enumerate() {
                            let route: Vec<String> = path
                                .iter()
                                .map(|n| format!("{}({},{})", n.symbol, n.x, n.y))
                                .collect();
                            let summary = prompt::summarize_path(path);
                            tracing::info!(
                                "    Path {}.{}: {}  [{}]",
                                (b'A' + ri as u8) as char,
                                pi + 1,
                                route.join(" → "),
                                summary,
                            );
                        }
                    }
                }
            } else {
                tracing::debug!("MAP screen but room_phase={rp} or no current_node");
            }
        }

        if is_game_over_state(&raw) {
            finalize_run_once(
                &journal,
                &provider,
                "game_over",
                &mut run_finalized,
                &locale,
            )
            .await;
            journal = journal::Journal::new(project_root.join("runs"));
            cache = AdviceCache::new();
            combat_turn_gate = CombatTurnGate::new();
            map_gate = MapGate::new();
            saw_game_state = false;
            run_finalized = false;
            last_autoplay_state = None;
            autoplay_overlay_state = autoplay::status::AutoPlayState::default();
            consecutive_errors = 0;
            tracing::info!("run ended, waiting for next run...");
            continue;
        }

        if screen_type == "SHOP_ROOM" || screen_type == "SHOP_SCREEN" {
            map_gate.on_shop();
        }
        if screen_type == "MAP" && normalized.map_first_node_chosen == Some(false) {
            map_gate.on_act_entry();
        }

        let map_should_generate =
            screen_type == "MAP" && map_gate.should_generate(&raw, &normalized.map_nodes);

        if !should_generate_advice(screen_type, &raw)
            && !combat_turn_gate.is_player_turn_start(&raw)
            && !map_should_generate
        {
            tracing::debug!("skipping screen type: {screen_type}");
            continue;
        }

        tracing::info!(
            "state screen={screen_type} room={room_type} floor={} hp={}/{} block={} energy={} hand={} mons={} deck={} incoming={} danger={:?}",
            normalized.floor.map_or("?".to_string(), |v| v.to_string()),
            normalized
                .current_hp
                .map_or("?".to_string(), |v| v.to_string()),
            normalized.max_hp.map_or("?".to_string(), |v| v.to_string()),
            normalized.block.map_or("?".to_string(), |v| v.to_string()),
            normalized.energy.map_or("?".to_string(), |v| v.to_string()),
            normalized.hand.len(),
            normalized.monsters.len(),
            normalized.deck_names.len(),
            normalized.incoming_damage,
            normalized.danger.level,
        );

        let effort = Effort::from_screen_type(screen_type, has_monsters(&raw));

        let prompt = prompt::build_prompt(&normalized, &locale, map_gate.shop_visited);
        tracing::debug!(
            "prompt ({} chars): {}",
            prompt.len(),
            &prompt[..prompt.len().min(PROMPT_LOG_PREVIEW_CHARS)]
        );

        cache.write_overlay_loading(&metadata);

        let advice = cache
            .get_or_compute(&hash, &prompt, effort, scenario, &provider, &locale)
            .await;

        let fields = advice::parse_advice_response(&advice, &locale);
        let status = if advice.contains(&locale.fallback.llm_error) {
            "error"
        } else {
            "ok"
        };
        cache.write_overlay_ready(status, &fields, &metadata);

        journal.log_advice(&hash, effort, scenario, &prompt, &advice);
        tracing::info!("wrote advice ({} chars hash={})", advice.len(), &hash[..16]);
        cache.write_advice(&advice);

        if screen_type == "MAP"
            && let (Some(cx), Some(cy)) = (normalized.map_current_x, normalized.map_current_y)
        {
            map_gate.record(&normalized.map_nodes, cx, cy);
        }
    }

    finalize_run_once(
        &journal,
        &provider,
        "stdin_closed",
        &mut run_finalized,
        &locale,
    )
    .await;
    tracing::info!("stdin closed, exiting");
}

#[cfg(test)]
mod test_utils;

#[cfg(test)]
#[path = "tests/main_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/ranker_tests.rs"]
mod ranker_integration_tests;
