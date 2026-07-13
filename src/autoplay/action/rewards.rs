use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
use crate::state::NormalizedState;

use super::{ActionCandidate, ActionRequest, AutoPlayAction, candidate, parse_index};

pub(super) fn combat_reward_candidates(
    control: &AutoPlayControl,
    session: &AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
) -> Vec<ActionCandidate> {
    let mut candidates = vec![];

    let skip_potion = session.skipped_combat_reward_potion && state.empty_potion_slots == 0;
    let skip_card = session.skipped_combat_reward_card;

    if command_state.has_command("choose") {
        for (index, choice) in command_state.choice_list.iter().enumerate() {
            let allowed = matches!(
                choice.as_str(),
                "gold" | "relic" | "stolen_gold" | "potion" | "emerald_key" | "sapphire_key"
            ) || (choice == "card" && control.allow_card_rewards);
            let skip_on_full = choice == "potion" && skip_potion;
            let skip_on_skipped = choice == "card" && skip_card;
            if allowed && !skip_on_full && !skip_on_skipped {
                candidates.push(candidate(
                    "choose",
                    format!("combat_reward:{choice}:{index}"),
                    format!("Collect {choice}"),
                ));
            }
        }
    }

    let visible = !candidates.is_empty();
    if command_state.has_command("proceed") && !visible {
        candidates.push(candidate(
            "proceed",
            "combat_reward:proceed".to_string(),
            "Proceed".to_string(),
        ));
    }

    candidates
}

pub(super) fn resolve_requested_combat_reward(
    command_state: &CommandState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.action_id == "combat_reward:proceed" && request.kind == "proceed" {
        return Some(AutoPlayAction::Proceed);
    }

    if request.kind != "choose" || !command_state.has_command("choose") {
        return None;
    }

    let index = request.action_id.rsplit(':').next()?.parse().ok()?;
    Some(AutoPlayAction::Choose(index))
}

pub(super) fn card_reward_candidates(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Vec<ActionCandidate> {
    let mut candidates = vec![];

    if command_state.has_command("choose") {
        let prefix = if state.is_boss_card_reward() {
            "boss_card_reward"
        } else {
            "card_reward"
        };
        for (index, card) in state.card_reward_choices.iter().enumerate() {
            candidates.push(candidate(
                "choose",
                format!("{prefix}:{index}"),
                card.name.clone(),
            ));
        }
    }

    if state.skip_available && command_state.has_command("skip") {
        let id = if state.is_boss_card_reward() {
            "boss_card_reward:skip"
        } else {
            "card_reward:skip"
        };
        candidates.push(candidate("skip", id.to_string(), "Skip".to_string()));
    }

    candidates
}

pub(super) fn resolve_requested_card_reward(
    command_state: &CommandState,
    state: &NormalizedState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    let prefix = if state.is_boss_card_reward() {
        "boss_card_reward:"
    } else {
        "card_reward:"
    };

    if request.action_id == format!("{prefix}skip") && request.kind == "skip" {
        return Some(AutoPlayAction::Skip);
    }

    if request.kind != "choose" || !command_state.has_command("choose") {
        return None;
    }

    let index = parse_index(&request.action_id, prefix)?;
    (index < state.card_reward_choices.len()).then_some(AutoPlayAction::Choose(index))
}

pub(super) fn boss_reward_candidates(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Vec<ActionCandidate> {
    if !command_state.has_command("choose") {
        return vec![];
    }

    state
        .boss_relic_choices
        .iter()
        .enumerate()
        .map(|(index, relic)| {
            candidate("choose", format!("boss_relic:{index}"), relic.name.clone())
        })
        .collect()
}

pub(super) fn resolve_requested_boss_reward(
    state: &NormalizedState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.kind != "choose" {
        return None;
    }

    let index = parse_index(&request.action_id, "boss_relic:")?;
    (index < state.boss_relic_choices.len()).then_some(AutoPlayAction::Choose(index))
}

pub(super) fn rest_candidates(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Vec<ActionCandidate> {
    let mut candidates: Vec<ActionCandidate> = vec![];

    if command_state.has_command("choose") {
        candidates.extend(
            state
                .rest_options
                .iter()
                .map(|option| candidate("choose", format!("rest:{option}"), option.clone())),
        );
    }

    if command_state.has_command("proceed") {
        candidates.push(candidate(
            "proceed",
            "rest:proceed".to_string(),
            "Proceed".to_string(),
        ));
    }

    candidates
}

pub(super) fn resolve_requested_rest(
    state: &NormalizedState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.action_id == "rest:proceed" && request.kind == "proceed" {
        return Some(AutoPlayAction::Proceed);
    }

    if request.kind != "choose" {
        return None;
    }

    let option = request.action_id.strip_prefix("rest:")?;
    state
        .rest_options
        .iter()
        .position(|candidate| candidate == option)
        .map(AutoPlayAction::Choose)
}
