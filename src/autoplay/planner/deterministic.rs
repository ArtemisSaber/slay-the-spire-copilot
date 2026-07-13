use crate::autoplay::action::{
    ActionCandidate, ActionRequest, AutoPlayAction, resolve_requested_action,
};
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
use crate::state::NormalizedState;

pub(super) fn potion_in_full_slots_was_rejected(
    candidates: &[ActionCandidate],
    command_state: &CommandState,
    state: &NormalizedState,
    action: &Option<AutoPlayAction>,
) -> Option<(usize, AutoPlayAction)> {
    if state.screen_type.as_ref().map(|st| st.as_str()) != Some("COMBAT_REWARD") {
        return None;
    }
    if state.empty_potion_slots > 0 {
        return None;
    }
    let potion_index = command_state
        .choice_list
        .iter()
        .position(|c| c == "potion")?;
    let potion_candidate_selected = candidates.iter().any(|c| {
        c.action_id == format!("combat_reward:potion:{potion_index}") && c.kind == "choose"
    });
    if !potion_candidate_selected {
        return None;
    }
    action.as_ref().map(|a| (potion_index, a.clone()))
}

pub(super) fn try_deterministic_action(
    control: &mut AutoPlayControl,
    session: &mut AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
    candidates: &[ActionCandidate],
) -> Option<AutoPlayAction> {
    if candidates.len() == 1 {
        let sole = &candidates[0];
        let request = ActionRequest {
            kind: sole.kind.clone(),
            action_id: sole.action_id.clone(),
            target_index: sole.target_required.and(Some(0)),
        };
        let action = resolve_requested_action(control, session, command_state, state, &request)?;

        return Some(action);
    }

    // COMBAT_REWARD: collect unambiguous rewards deterministically. A key makes
    // the remaining reward choice strategic, so defer it to the LLM.
    if state.screen_type.as_ref().map(|st| st.as_str()) == Some("COMBAT_REWARD") {
        let has_potion = command_state.choice_list.iter().any(|c| c == "potion");
        let has_card = command_state.choice_list.iter().any(|c| c == "card");

        // 1. Gold is never a trade-off. Collect it before deciding among the
        // remaining rewards.
        if let Some(index) = command_state
            .choice_list
            .iter()
            .position(|choice| matches!(choice.as_str(), "gold" | "stolen_gold"))
            && command_state.has_command("choose")
        {
            return Some(AutoPlayAction::Choose(index));
        }

        // Taking a key instead of its linked relic is a run-level decision.
        // Do not let the first entry in choice_list decide it.
        if has_key_reward(&command_state.choice_list) {
            return None;
        }

        // Without a key alternative, collecting a relic is unambiguous.
        if let Some(index) = command_state
            .choice_list
            .iter()
            .position(|choice| choice == "relic")
            && command_state.has_command("choose")
        {
            return Some(AutoPlayAction::Choose(index));
        }

        // 2. Potion: deterministic if empty slots, LLM if full and not skipped.
        if has_potion && !session.skipped_combat_reward_potion {
            if state.empty_potion_slots > 0
                && let Some(index) = command_state.choice_list.iter().position(|c| c == "potion")
            {
                return Some(AutoPlayAction::Choose(index));
            }
            return None;
        }

        // 3. Card: always pick (enters CARD_REWARD where LLM decides pick/skip).
        if has_card
            && !session.skipped_combat_reward_card
            && let Some(index) = command_state.choice_list.iter().position(|c| c == "card")
        {
            return Some(AutoPlayAction::Choose(index));
        }

        // 4. Safety net: any unknown leftover choice.
        if !command_state.choice_list.is_empty() && command_state.has_command("choose") {
            return Some(AutoPlayAction::Choose(0));
        }

        // 5. Nothing left — proceed.
        if command_state.has_command("proceed") {
            return Some(AutoPlayAction::Proceed);
        }
        return None;
    }

    None
}

pub(crate) fn has_key_reward(choice_list: &[String]) -> bool {
    choice_list
        .iter()
        .any(|choice| matches!(choice.as_str(), "emerald_key" | "sapphire_key"))
}
