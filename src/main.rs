mod advice;
mod config;
mod i18n;
mod journal;
mod llm;
mod logging;
mod prompt;
mod protocol;
mod startup;
mod state;

use advice::AdviceCache;
use llm::Effort;
use std::io::{self, BufRead};

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
        .map(|arr| arr.iter().any(|m| !m.get("is_gone").and_then(|g| g.as_bool()).unwrap_or(false)))
        .unwrap_or(false)
}

struct ScreenConfig {
    generate: &'static [&'static str],
    generate_on_combat: &'static [&'static str],
}

const SCREEN_CONFIG: ScreenConfig = ScreenConfig {
    generate: &["CARD_REWARD"],
    generate_on_combat: &[],
    // Future screens to add to `generate`:
    // "REST", "SHOP", "BOSS_REWARD", "EVENT", "HAND_SELECT", "GRID",
};

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

    if !startup::ensure_config() {
        return;
    }

    let config = config::Config::from_env();
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
    journal.log_run_started();
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

        if !should_generate_advice(screen_type, &raw) {
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

        let prompt = prompt::build_prompt(&normalized, &i18n_data);
        tracing::debug!(
            "prompt ({} chars): {}",
            prompt.len(),
            &prompt[..prompt.len().min(200)]
        );

        let advice = cache
            .get_or_compute(&hash, &prompt, effort, &provider)
            .await;

        journal.log_advice(&hash, effort, &prompt, &advice);
        tracing::info!("wrote advice ({} chars hash={})", advice.len(), &hash[..16]);
        cache.write_advice(&advice);
    }

    journal.log_run_ended("stdin_closed");
    tracing::info!("stdin closed, exiting");
}

#[cfg(test)]
#[path = "tests/main_tests.rs"]
mod tests;
