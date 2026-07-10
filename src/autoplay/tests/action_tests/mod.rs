use super::*;
use crate::autoplay::control::{AutoPlayControl, ControlLoad};
use crate::locales::Locale;
use crate::state::ScreenType;
use serde_json::{Value, json};

fn state(raw: Value) -> NormalizedState {
    NormalizedState::from_raw(&raw, &Locale::load("en"))
}

fn command_state(raw: &Value) -> CommandState {
    CommandState::from_raw(raw)
}

fn request(kind: &str, action_id: &str) -> ActionRequest {
    ActionRequest {
        kind: kind.to_string(),
        action_id: action_id.to_string(),
        target_index: None,
    }
}

fn targeted_request(kind: &str, action_id: &str, target_index: usize) -> ActionRequest {
    ActionRequest {
        kind: kind.to_string(),
        action_id: action_id.to_string(),
        target_index: Some(target_index),
    }
}

fn resolve(
    raw: &Value,
    control: &AutoPlayControl,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    resolve_requested_action(
        control,
        &AutoPlaySession::default(),
        &command_state(raw),
        &state(raw.clone()),
        request,
    )
}

mod combat_candidates;
mod combat_misc;
mod combat_potion_candidates;
mod combat_potion_resolve;
mod combat_resolve;

mod core;

mod execute;

mod gating_control;
mod gating_misc;
mod gating_screens;

mod navigation_chest_complete;
mod navigation_event;
mod navigation_grid;
mod navigation_hand_select;
mod navigation_indexed;
mod navigation_map;
mod navigation_rest;
mod navigation_shop;

mod rewards_boss;
mod rewards_cards;
mod rewards_combat_core;
mod rewards_combat_filters;
mod rewards_combat_items;
mod rewards_misc;
