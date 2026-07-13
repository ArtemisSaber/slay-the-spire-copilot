use crate::autoplay::action::AutoPlayAction;
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::AutoPlayControl;
use crate::state::NormalizedState;

use super::deterministic::has_key_reward;

pub(super) fn fallback_action(
    control: &AutoPlayControl,
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    match state.screen_type.as_ref().map(|st| st.as_str()) {
        Some("COMBAT_REWARD") if control.allow_combat_rewards => {
            fallback_combat_reward_action(control, command_state)
        }
        Some("CARD_REWARD") if control.allow_card_rewards => {
            fallback_card_reward_action(command_state, state)
        }
        Some("BOSS_REWARD") if control.allow_boss_rewards => (!state.boss_relic_choices.is_empty()
            && command_state.has_command("choose"))
        .then_some(AutoPlayAction::Choose(0)),
        Some("REST") if control.allow_rest => fallback_rest_action(command_state, state),
        Some("EVENT") if control.allow_events => (!state.event_choices.is_empty()
            && command_state.has_command("choose"))
        .then_some(AutoPlayAction::Choose(0)),
        Some("SHOP_ROOM" | "SHOP_SCREEN") if control.allow_shop => command_state
            .has_command("leave")
            .then_some(AutoPlayAction::Leave),
        Some("MAP") if control.allow_map => (!command_state.choice_list.is_empty()
            && command_state.has_command("choose"))
        .then_some(AutoPlayAction::Choose(0)),
        Some("NONE") if control.allow_combat => fallback_combat_action(command_state, state),
        Some("CHEST") if control.allow_selection_screens => command_state
            .has_command("choose")
            .then_some(AutoPlayAction::Choose(0))
            .or_else(|| {
                command_state
                    .has_command("proceed")
                    .then_some(AutoPlayAction::Proceed)
            }),
        Some("COMPLETE") if control.allow_selection_screens => command_state
            .has_command("proceed")
            .then_some(AutoPlayAction::Proceed),
        Some("GRID") if control.allow_selection_screens => command_state
            .has_command("choose")
            .then_some(AutoPlayAction::Choose(0))
            .or_else(|| {
                command_state
                    .has_command("confirm")
                    .then_some(AutoPlayAction::Proceed)
            }),
        Some("HAND_SELECT") if control.allow_selection_screens => command_state
            .has_command("choose")
            .then_some(AutoPlayAction::Choose(0))
            .or_else(|| {
                command_state
                    .has_command("confirm")
                    .then_some(AutoPlayAction::Proceed)
            }),
        _ => None,
    }
}

fn fallback_combat_reward_action(
    _control: &AutoPlayControl,
    command_state: &CommandState,
) -> Option<AutoPlayAction> {
    if command_state.has_command("choose") {
        if let Some(index) = command_state
            .choice_list
            .iter()
            .position(|choice| matches!(choice.as_str(), "gold" | "stolen_gold"))
        {
            return Some(AutoPlayAction::Choose(index));
        }
        if has_key_reward(&command_state.choice_list) {
            return None;
        }
        if let Some(index) = command_state
            .choice_list
            .iter()
            .position(|choice| choice == "relic")
        {
            return Some(AutoPlayAction::Choose(index));
        }
        if !command_state.choice_list.is_empty() {
            return Some(AutoPlayAction::Choose(0));
        }
    }

    if command_state.choice_list.is_empty() && command_state.has_command("proceed") {
        return Some(AutoPlayAction::Proceed);
    }

    None
}

fn fallback_card_reward_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if state.skip_available && command_state.has_command("skip") {
        return Some(AutoPlayAction::Skip);
    }

    if !state.card_reward_choices.is_empty() && command_state.has_command("choose") {
        return Some(AutoPlayAction::Choose(0));
    }

    None
}

pub(super) fn fallback_rest_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if !command_state.has_command("choose") {
        return command_state
            .has_command("proceed")
            .then_some(AutoPlayAction::Proceed);
    }

    let rest_index = state
        .rest_options
        .iter()
        .position(|option| option == "rest");
    let smith_index = state
        .rest_options
        .iter()
        .position(|option| option == "smith");
    let hp_is_low = match (state.current_hp, state.max_hp) {
        (Some(current), Some(max)) if max > 0 => current * 2 < max,
        _ => false,
    };

    if hp_is_low {
        rest_index.or(smith_index).map(AutoPlayAction::Choose)
    } else {
        smith_index.or(rest_index).map(AutoPlayAction::Choose)
    }
}

fn fallback_combat_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if command_state.has_command("play") {
        let first_target = state.monsters.first().map(|monster| monster.index);
        if let Some((hand_index, target_index)) =
            state
                .hand
                .iter()
                .enumerate()
                .find_map(|(hand_index, card)| {
                    if !card.playable {
                        return None;
                    }

                    let target_index = if card.has_target { first_target } else { None };
                    if card.has_target && target_index.is_none() {
                        return None;
                    }

                    Some((hand_index, target_index))
                })
        {
            return Some(AutoPlayAction::Play {
                hand_index,
                target_index,
            });
        }
    }

    command_state
        .has_command("end")
        .then_some(AutoPlayAction::End)
}
