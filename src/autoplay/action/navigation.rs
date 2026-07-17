use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::AutoPlaySession;
use crate::state::NormalizedState;

use super::{ActionCandidate, ActionRequest, AutoPlayAction, candidate, parse_index};

pub(super) fn event_candidates(
    command_state: &CommandState,
    state: &crate::state::NormalizedState,
) -> Vec<ActionCandidate> {
    if !command_state.has_command("choose") {
        tracing::info!(
            "event_candidates empty: no 'choose' command commands={:?}",
            command_state.available_commands,
        );
        return vec![];
    }

    let detailed_choices = (state.event_choices.len() == command_state.choice_list.len())
        .then_some(state.event_choices.as_slice());
    let candidates: Vec<_> = command_state
        .choice_list
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let label = detailed_choices
                .and_then(|choices| choices.get(index))
                .unwrap_or(choice)
                .clone();
            candidate("choose", format!("event:{index}"), label)
        })
        .collect();
    if candidates.is_empty() {
        tracing::info!("event_candidates empty: executable choice_list is empty");
    }
    candidates
}

pub(super) fn resolve_requested_indexed(
    prefix: &str,
    len: usize,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.kind != "choose" {
        return None;
    }

    let index = parse_index(&request.action_id, prefix)?;
    (index < len).then_some(AutoPlayAction::Choose(index))
}

pub(super) fn shop_candidates(
    session: &AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
) -> Vec<ActionCandidate> {
    let mut candidates = vec![];

    let completed = match (session.completed_shop_floor, state.floor) {
        (Some(last), Some(current)) => last == current,
        _ => false,
    };
    let inside_shop =
        state.screen_type.as_ref().map(|screen| screen.as_str()) == Some("SHOP_SCREEN");

    if completed && !inside_shop {
        if command_state.has_command("proceed") {
            candidates.push(candidate(
                "proceed",
                "shop:proceed".to_string(),
                "Proceed".to_string(),
            ));
        }
    } else if command_state.has_command("choose") {
        for (index, choice) in command_state.choice_list.iter().enumerate() {
            candidates.push(candidate(
                "choose",
                format!("shop:choice:{index}"),
                choice.clone(),
            ));
        }
    }

    if inside_shop
        && command_state.has_command("proceed")
        && !candidates
            .iter()
            .any(|candidate| candidate.action_id == "shop:proceed")
    {
        candidates.push(candidate(
            "proceed",
            "shop:proceed".to_string(),
            "Proceed".to_string(),
        ));
    }

    if command_state.has_command("leave") {
        candidates.push(candidate(
            "leave",
            "shop:leave".to_string(),
            "Leave shop".to_string(),
        ));
    }

    candidates
}

pub(super) fn resolve_requested_shop(
    command_state: &CommandState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.action_id == "shop:leave" && request.kind == "leave" {
        return Some(AutoPlayAction::Leave);
    }

    if request.action_id == "shop:proceed" && request.kind == "proceed" {
        return Some(AutoPlayAction::Proceed);
    }

    resolve_requested_indexed("shop:choice:", command_state.choice_list.len(), request)
}

pub(super) fn map_candidates(command_state: &CommandState) -> Vec<ActionCandidate> {
    if !command_state.has_command("choose") {
        return vec![];
    }

    command_state
        .choice_list
        .iter()
        .enumerate()
        .map(|(index, choice)| candidate("choose", format!("map:choice:{index}"), choice.clone()))
        .collect()
}

pub(super) fn chest_candidates(command_state: &CommandState) -> Vec<ActionCandidate> {
    if command_state.has_command("choose") {
        return command_state
            .choice_list
            .iter()
            .enumerate()
            .map(|(index, choice)| candidate("choose", format!("chest:{index}"), choice.clone()))
            .collect();
    }

    if command_state.has_command("proceed") {
        return vec![ActionCandidate {
            kind: "proceed".to_string(),
            action_id: "chest:proceed".to_string(),
            label: "Proceed".to_string(),
            target_required: None,
        }];
    }

    vec![]
}

pub(super) fn grid_candidates(command_state: &CommandState) -> Vec<ActionCandidate> {
    let mut candidates: Vec<ActionCandidate> = vec![];

    if command_state.has_command("choose") {
        candidates.extend(
            command_state
                .choice_list
                .iter()
                .enumerate()
                .map(|(index, choice)| {
                    candidate("choose", format!("grid:{index}"), choice.clone())
                }),
        );
    }

    if command_state.has_command("confirm") {
        candidates.push(candidate(
            "proceed",
            "grid:confirm".to_string(),
            "Confirm".to_string(),
        ));
    }

    candidates
}

pub(super) fn hand_select_candidates(command_state: &CommandState) -> Vec<ActionCandidate> {
    let mut candidates: Vec<ActionCandidate> = vec![];

    if command_state.has_command("choose") {
        candidates.extend(
            command_state
                .choice_list
                .iter()
                .enumerate()
                .map(|(index, choice)| {
                    candidate("choose", format!("hand_select:{index}"), choice.clone())
                }),
        );
    }

    if command_state.has_command("confirm") {
        candidates.push(candidate(
            "proceed",
            "hand_select:confirm".to_string(),
            "Confirm".to_string(),
        ));
    }

    candidates
}
