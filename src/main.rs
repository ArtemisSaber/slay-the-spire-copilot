mod advice;
mod config;
mod i18n;
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

#[tokio::main]
async fn main() {
    let _guard = logging::init();
    dotenvy::dotenv().ok();

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

        let prompt = prompt::build_prompt(&normalized);
        tracing::debug!(
            "prompt ({} chars): {}",
            prompt.len(),
            &prompt[..prompt.len().min(200)]
        );

        let advice = cache
            .get_or_compute(&hash, &prompt, effort, &provider)
            .await;

        tracing::info!("wrote advice ({} chars hash={})", advice.len(), &hash[..16]);
        cache.write_advice(&advice);
    }

    tracing::info!("stdin closed, exiting");
}
