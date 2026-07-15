use super::*;
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
use crate::locales::Locale;
use serde_json::{Value, json};

fn state(raw: Value) -> NormalizedState {
    NormalizedState::from_raw(&raw, &Locale::load("en"))
}

fn command_state(raw: &Value) -> CommandState {
    CommandState::from_raw(raw)
}

mod deterministic_core;
mod deterministic_reward_choices;
mod deterministic_skip_flags;

mod fallback;

mod experience_context;

mod prompt_action_labels;
mod prompt_ranked_suggestions;

mod response;
