mod logging;
mod mock;
mod prompts;
mod provider;
mod requests;
mod types;

pub use provider::LlmProvider;
pub use types::{AdviceScenario, Effort};

#[cfg(test)]
use {logging::*, mock::*, prompts::*, provider::OpenAiConfig, requests::*};

#[cfg(test)]
#[path = "tests/llm_tests.rs"]
mod tests;
