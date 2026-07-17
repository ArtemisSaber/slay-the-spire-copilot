use super::*;
use crate::autoplay::action::ActionCandidate;
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

fn planner_payload(
    state: &NormalizedState,
    candidates: &[ActionCandidate],
    shop_visited: bool,
) -> Value {
    let command_state = CommandState {
        ready_for_command: true,
        available_commands: candidates
            .iter()
            .map(|candidate| candidate.kind.clone())
            .collect(),
        choice_list: vec![],
    };
    let prompt = build_planner_prompt(
        &AutoPlaySession::default(),
        &command_state,
        state,
        &Locale::load("en"),
        shop_visited,
        candidates,
        &[],
    )
    .expect("planner prompt should serialize");
    serde_json::from_str(&prompt).expect("planner prompt should be JSON")
}

mod deterministic_core;
mod deterministic_reward_choices;
mod deterministic_skip_flags;

mod fallback;

mod experience_context;
mod in_combat_routing;

mod card_reward_comparison;
mod card_reward_pipeline;
mod card_reward_retries;
mod card_reward_selection;
mod prompt_action_labels;
mod prompt_ranked_suggestions;
mod structured_context_combat;
mod structured_context_screens;

mod reference_response;
mod response;
