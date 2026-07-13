#![deny(clippy::allow_attributes_without_reason)]
mod advice;
mod app;
mod autoplay;
mod combat;
mod config;
mod gate;
mod journal;
mod llm;
mod locales;
mod logging;
mod parsing;
mod postmortem;
mod prompt;
mod protocol;
mod ranker;
mod relic_counters;
mod runtime;
mod setup_wizard;
mod startup;
mod state;

pub(crate) const MAX_STDIN_JSON_BYTES: usize = 10 * 1024 * 1024;

#[tokio::main]
async fn main() {
    app::run().await;
}

#[cfg(test)]
pub(crate) use app::finalization::finalize_run_once;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
#[path = "tests/main_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/ranker_tests.rs"]
mod ranker_integration_tests;
