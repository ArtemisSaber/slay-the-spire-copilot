use super::*;
use crate::llm::{AdviceScenario, Effort, LlmProvider};
use crate::state::ScreenType;
use crate::test_utils::test_locale;

#[path = "advice_tests/cache.rs"]
mod cache;
#[path = "advice_tests/io.rs"]
mod io;
#[path = "advice_tests/output.rs"]
mod output;
#[path = "advice_tests/parsing.rs"]
mod parsing;
