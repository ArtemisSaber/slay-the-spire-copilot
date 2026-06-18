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

fn can_wait(raw: &serde_json::Value) -> bool {
    raw.get("available_commands")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().any(|c| c.as_str() == Some("wait")))
        .unwrap_or(false)
}

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
                protocol::send_response(false);
                continue;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            protocol::send_response(false);
            continue;
        }

        let raw: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("failed to parse JSON: {e}");
                protocol::send_response(false);
                continue;
            }
        };

        if is_error(&raw) {
            tracing::warn!("received error from CommunicationMod: {}", trimmed);
            protocol::send_response(false);
            continue;
        }

        if !is_in_game(&raw) {
            protocol::send_response(can_wait(&raw));
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

        protocol::send_response(can_wait(&raw));
    }

    tracing::info!("stdin closed, exiting");
}
