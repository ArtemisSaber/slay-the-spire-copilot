mod advice;
mod config;
mod i18n;
mod journal;
mod llm;
mod logging;
mod postmortem;
mod prompt;
mod protocol;
mod startup;
mod state;

use advice::AdviceCache;
use llm::{AdviceScenario, Effort};
use std::collections::HashSet;
use std::io::{self, BufRead};

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
    generate: &["CARD_REWARD", "REST"],
    generate_on_combat: &[],
    // Future screens to add to `generate`:
    // "REST", "SHOP", "BOSS_REWARD", "EVENT", "HAND_SELECT", "GRID",
};

struct AdviceGate {
    advised_combats: HashSet<String>,
}

impl AdviceGate {
    fn new() -> Self {
        AdviceGate {
            advised_combats: HashSet::new(),
        }
    }

    fn should_generate(&mut self, screen_type: &str, raw: &serde_json::Value) -> bool {
        if should_generate_advice(screen_type, raw) {
            return true;
        }

        if screen_type == "NONE"
            && has_monsters(raw)
            && let Some(identity) = combat_identity(raw)
        {
            return self.advised_combats.insert(identity);
        }

        false
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
    if SCREEN_CONFIG.generate.contains(&screen_type) {
        return true;
    }
    if SCREEN_CONFIG.generate_on_combat.contains(&screen_type) && has_monsters(raw) {
        return true;
    }
    false
}

#[tokio::main]
async fn main() {
    let _guard = logging::init();
    let project_root = logging::project_root();
    let _ = dotenvy::from_path(project_root.join(".env"));
    let options = RuntimeOptions::from_env_and_args();

    if let Some(path) = options.postmortem_path.as_deref() {
        let deterministic_report = match std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|content| postmortem::generate_report_from_jsonl(&content))
        {
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
                let prompt = postmortem::build_ai_postmortem_prompt(&deterministic_report);
                match provider.query_postmortem(&prompt).await {
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

    let i18n_data = i18n::I18n::load();
    let mut cache = AdviceCache::new();
    let mut journal = journal::Journal::new();
    journal.log_run_started_with_config(&config);
    let mut advice_gate = AdviceGate::new();
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
            tracing::debug!("skipping non-game state");
            continue;
        }

        let screen_type = raw
            .pointer("/game_state/screen_type")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let room_type = raw
            .pointer("/game_state/room_type")
            .and_then(|v| v.as_str())
            .unwrap_or("?");

        let normalized = state::NormalizedState::from_raw(&raw, &i18n_data);
        let hash = normalized.stable_hash();
        journal.log_state_change(&hash, &normalized);

        if !advice_gate.should_generate(screen_type, &raw) {
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

        let prompt = prompt::build_prompt(&normalized, &i18n_data);
        tracing::debug!(
            "prompt ({} chars): {}",
            prompt.len(),
            &prompt[..prompt.len().min(200)]
        );

        let advice = cache
            .get_or_compute(&hash, &prompt, effort, scenario, &provider)
            .await;

        journal.log_advice(&hash, effort, scenario, &prompt, &advice);
        tracing::info!("wrote advice ({} chars hash={})", advice.len(), &hash[..16]);
        cache.write_advice(&advice);
    }

    journal.log_run_ended("stdin_closed");
    tracing::info!("stdin closed, exiting");
}

#[cfg(test)]
#[path = "tests/main_tests.rs"]
mod tests;
