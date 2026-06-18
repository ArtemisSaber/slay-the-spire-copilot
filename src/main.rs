mod advice;
mod config;
mod llm;
mod logging;
mod prompt;
mod protocol;
mod startup;
mod state;

use advice::AdviceCache;
use std::io::{self, BufRead, Write};

fn is_in_game(raw: &serde_json::Value) -> bool {
    raw.get("in_game")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn ready_for_command(raw: &serde_json::Value) -> bool {
    raw.get("ready_for_command")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn is_error(raw: &serde_json::Value) -> bool {
    raw.get("error").is_some()
}

#[tokio::main]
async fn main() {
    logging::init();
    dotenvy::dotenv().ok();

    if !startup::ensure_config() {
        return;
    }

    let config = config::Config::from_env();
    tracing::info!("provider={} model={}", config.provider, config.model);

    let provider = match llm::LlmProvider::from_config(&config) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("failed to create LLM provider: {e}");
            return;
        }
    };

    protocol::send_ready();
    tracing::info!("sent ready");

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
            continue;
        }

        let normalized = state::NormalizedState::from_raw(&raw);
        let hash = normalized.stable_hash();
        let prompt = prompt::build_prompt(&normalized);

        let advice = cache.get_or_compute(&hash, &prompt, &provider).await;

        if let Err(e) = std::io::stdout().flush() {
            tracing::error!("failed to flush stdout before writing advice: {e}");
        }

        cache.write_advice(&advice);
        tracing::info!("wrote advice (hash={})", &hash[..16]);

        if ready_for_command(&raw) {
            protocol::send_wait();
        }
    }

    tracing::info!("stdin closed, exiting");
}
