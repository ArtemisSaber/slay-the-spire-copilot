#![deny(clippy::allow_attributes_without_reason)]
mod advice;
mod config;
mod journal;
mod llm;
mod locales;
mod logging;
mod postmortem;
mod prompt;
mod protocol;
mod startup;
mod state;

use advice::{AdviceCache, OverlayMetadata};
use llm::{AdviceScenario, Effort};
use std::io::{self, BufRead, IsTerminal, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeOptions {
    skip_startup_check: bool,
    force_mock_provider: bool,
    postmortem_path: Option<String>,
    postmortem_plain: bool,
}

impl RuntimeOptions {
    fn from_env_and_args() -> Self {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let skip_env = std::env::var("SKIP_COMM_CONFIG").ok();
        runtime_options_from(args.iter().map(|s| s.as_str()), skip_env.as_deref())
    }
}

fn runtime_options_from<'a>(
    args: impl IntoIterator<Item = &'a str>,
    skip_comm_config: Option<&str>,
) -> RuntimeOptions {
    let mut skip_startup_check = matches!(skip_comm_config, Some("1" | "true" | "yes"));
    let mut force_mock_provider = false;
    let mut postmortem_path = None;
    let mut postmortem_plain = false;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg {
            "--no-startup-check" => skip_startup_check = true,
            "--stdin-test" => {
                skip_startup_check = true;
                force_mock_provider = true;
            }
            "postmortem" => {
                for next in iter.by_ref() {
                    if next == "--plain" {
                        postmortem_plain = true;
                    } else {
                        postmortem_path = Some(next.to_string());
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    RuntimeOptions {
        skip_startup_check,
        force_mock_provider,
        postmortem_path,
        postmortem_plain,
    }
}

fn is_in_game(raw: &serde_json::Value) -> bool {
    raw.get("in_game")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn is_error(raw: &serde_json::Value) -> bool {
    raw.get("error").is_some()
}

fn is_game_over_state(raw: &serde_json::Value) -> bool {
    raw.pointer("/game_state/screen_type")
        .and_then(|v| v.as_str())
        .is_some_and(|screen| screen == "GAME_OVER")
}

fn should_end_run(raw: &serde_json::Value, has_seen_game_state: bool) -> bool {
    is_game_over_state(raw) || (has_seen_game_state && !is_in_game(raw))
}

fn run_end_reason(raw: &serde_json::Value, has_seen_game_state: bool) -> Option<&'static str> {
    if is_game_over_state(raw) {
        Some("game_over")
    } else if has_seen_game_state && !is_in_game(raw) {
        Some("left_game")
    } else {
        None
    }
}

fn has_monsters(raw: &serde_json::Value) -> bool {
    raw.pointer("/game_state/combat_state/monsters")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .any(|m| !m.get("is_gone").and_then(|g| g.as_bool()).unwrap_or(false))
        })
        .unwrap_or(false)
}

struct ScreenConfig {
    generate: &'static [&'static str],
    generate_on_combat: &'static [&'static str],
}

const SCREEN_CONFIG: ScreenConfig = ScreenConfig {
    generate: &["CARD_REWARD", "BOSS_REWARD", "EVENT", "REST"],
    generate_on_combat: &[],
    // Future screens to add to `generate`:
    // "SHOP", "HAND_SELECT", "GRID",
};

struct CombatTurnGate {
    last_turn: Option<(String, i64)>,
}

impl CombatTurnGate {
    fn new() -> Self {
        CombatTurnGate { last_turn: None }
    }

    fn is_player_turn_start(&mut self, raw: &serde_json::Value) -> bool {
        if raw
            .pointer("/game_state/screen_type")
            .and_then(|v| v.as_str())
            != Some("NONE")
        {
            return false;
        }
        if raw
            .pointer("/game_state/action_phase")
            .and_then(|v| v.as_str())
            != Some("WAITING_ON_USER")
        {
            return false;
        }
        if !has_monsters(raw) {
            return false;
        }
        let identity = match combat_identity(raw) {
            Some(id) => id,
            None => return false,
        };
        let turn = match raw
            .pointer("/game_state/combat_state/turn")
            .and_then(|v| v.as_i64())
        {
            Some(t) => t,
            None => return false,
        };
        let key = (identity, turn);
        if self.last_turn.as_ref() == Some(&key) {
            return false;
        }
        self.last_turn = Some(key);
        true
    }
}

fn combat_identity(raw: &serde_json::Value) -> Option<String> {
    let floor = raw.pointer("/game_state/floor")?.as_i64()?;
    let room_type = raw
        .pointer("/game_state/room_type")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    Some(format!("{floor}:{room_type}"))
}

fn should_generate_advice(screen_type: &str, raw: &serde_json::Value) -> bool {
    if screen_type == "EVENT" {
        return available_event_choice_count(raw) > 1;
    }
    if SCREEN_CONFIG.generate.contains(&screen_type) {
        return true;
    }
    if SCREEN_CONFIG.generate_on_combat.contains(&screen_type) && has_monsters(raw) {
        return true;
    }
    false
}

fn available_event_choice_count(raw: &serde_json::Value) -> usize {
    let choices = raw
        .pointer("/game_state/screen_state/options")
        .or_else(|| raw.pointer("/game_state/screen_state/choices"))
        .or_else(|| raw.pointer("/game_state/screen_state/buttons"))
        .or_else(|| raw.pointer("/game_state/choice_list"));

    choices
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|choice| {
                    !choice
                        .get("disabled")
                        .and_then(|disabled| disabled.as_bool())
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

async fn postmortem_report_text(
    deterministic_report: &str,
    provider: &llm::LlmProvider,
    locale: &locales::Locale,
) -> String {
    let prompt = postmortem::build_ai_postmortem_prompt(deterministic_report, locale);
    match provider.query_postmortem(&prompt, locale).await {
        Ok(report) => report,
        Err(e) => {
            tracing::warn!("AI postmortem failed, saving deterministic report: {e}");
            deterministic_report.to_string()
        }
    }
}

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

    journal.log_run_ended(reason);

    let deterministic_report =
        match postmortem::generate_report_from_journal_file(journal.path(), locale) {
            Ok(report) => report,
            Err(e) => {
                tracing::error!("failed to generate postmortem report: {e}");
                return;
            }
        };
    let report = postmortem_report_text(&deterministic_report, provider, locale).await;

    match postmortem::write_report_for_journal(journal.path(), &report) {
        Ok(path) => tracing::info!("wrote postmortem report to {}", path.display()),
        Err(e) => tracing::error!("failed to write postmortem report: {e}"),
    }
}

#[tokio::main]
async fn main() {
    let _guard = logging::init();
    let project_root = logging::project_root();
    let _ = dotenvy::from_path(project_root.join(".env"));
    let options = RuntimeOptions::from_env_and_args();

    if let Some(path) = options.postmortem_path.as_deref() {
        let postmortem_locale = locales::Locale::load("en");

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
                let prompt = postmortem::build_ai_postmortem_prompt(
                    &deterministic_report,
                    &postmortem_locale,
                );
                match provider.query_postmortem(&prompt, &postmortem_locale).await {
                    Ok(report) => println!("{report}"),
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
    let mut journal = journal::Journal::new();
    journal.log_run_started_with_config(&config);
    let mut combat_turn_gate = CombatTurnGate::new();
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

        tracing::debug!("received {} bytes", trimmed.len());

        let raw: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("failed to parse JSON: {e}");
                continue;
            }
        };

        if is_error(&raw) {
            tracing::warn!("received error from CommunicationMod: {}", trimmed);
            continue;
        }

        if !is_in_game(&raw) {
            if should_end_run(&raw, saw_game_state) {
                let reason = run_end_reason(&raw, saw_game_state).unwrap_or("left_game");
                finalize_run_once(&journal, &provider, reason, &mut run_finalized, &locale).await;
                break;
            }
            tracing::debug!("skipping non-game state");
            continue;
        }

        saw_game_state = true;

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
        journal.log_state_change(&hash, &normalized);

        if is_game_over_state(&raw) {
            finalize_run_once(
                &journal,
                &provider,
                "game_over",
                &mut run_finalized,
                &locale,
            )
            .await;
            break;
        }

        if !should_generate_advice(screen_type, &raw)
            && !combat_turn_gate.is_player_turn_start(&raw)
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

        let effort = Effort::from_screen_type(screen_type);
        let scenario = AdviceScenario::from_state(&normalized);

        let prompt = prompt::build_prompt(&normalized, &locale);
        tracing::debug!(
            "prompt ({} chars): {}",
            prompt.len(),
            &prompt[..prompt.len().min(200)]
        );

        let metadata = OverlayMetadata {
            screen_type: Some(screen_type.to_string()),
            scenario: scenario.as_str().to_string(),
            in_combat: has_monsters(&raw),
            state_hash: hash.clone(),
            floor: normalized.floor,
            character: normalized.character.clone(),
        };
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
