use crate::autoplay::action::{ActionCandidate, AutoPlayAction, action_reference};
use crate::ranker::context::ActionType;
use crate::state::NormalizedState;

mod memory;
pub use memory::{ranker_tags, recorded_ranked_actions};

pub fn try_kill_scan_action(state: &NormalizedState) -> Option<AutoPlayAction> {
    if state.screen_type.as_ref().map(|st| st.as_str()) != Some("NONE") {
        return None;
    }
    let sequence = crate::combat::find_kill_sequence(state)?;
    let first = sequence.first()?;
    let hand_index = state
        .hand
        .iter()
        .position(|c| c.uuid.as_deref() == Some(&first.card))?;
    Some(AutoPlayAction::Play {
        hand_index,
        target_index: first.target,
    })
}

fn resolve_target(
    state: &NormalizedState,
    target_index: Option<usize>,
) -> Option<serde_json::Value> {
    let idx = target_index?;
    let monster = state.monsters.iter().find(|m| m.index == idx)?;
    Some(serde_json::json!({
        "index": monster.index,
        "name": monster.name,
        "hp": monster.current_hp,
        "block": monster.block,
    }))
}

fn derive_tags(breakdown: &[crate::ranker::engine::RuleResult]) -> Vec<String> {
    let mut tags = Vec::new();
    for r in breakdown {
        if !r.matched {
            continue;
        }
        let tag = match r.rule_id.as_str() {
            "core_damage" | "core_damage_vulnerable" | "core_damage_intangible" => "damage",
            "core_block_non_excessive" | "core_block_retain" | "core_block_retain_relic" => "block",
            "core_block_excessive" => "excessive",
            "target_killable" => "killable",
            "combat_ends_fight" => "lethal",
            "combat_priority_kill" | "combat_minion_kill" => "priority_kill",
            "core_heal" => "heal",
            "setup_power_card" => "power",
            "core_draw_has_energy" => "draw",
            "potion_base" => "potion",
            "danger_retaliatory_damage" => "retaliation",
            _ => continue,
        };
        if !tags.contains(&tag.to_string()) {
            tags.push(tag.to_string());
        }
    }
    tags
}

#[cfg(test)]
fn action_entry(
    s: &crate::ranker::engine::ScoredAction,
    state: &NormalizedState,
) -> serde_json::Value {
    let target = resolve_target(state, s.target_index);
    let tags = derive_tags(&s.breakdown);
    serde_json::json!({
        "action": s.action_type,
        "target": target,
        "score": s.score,
        "tags": tags,
    })
}

fn referenced_action_entry(
    scored: &crate::ranker::engine::ScoredAction,
    state: &NormalizedState,
    candidates: &[ActionCandidate],
) -> Option<serde_json::Value> {
    let candidate_index = candidate_index(scored, state, candidates)?;
    let candidate = &candidates[candidate_index];
    Some(serde_json::json!({
        "ref": action_reference(candidate_index),
        "label": candidate.label,
        "target": resolve_target(state, scored.target_index),
        "score": scored.score,
        "tags": derive_tags(&scored.breakdown),
    }))
}

fn candidate_index(
    scored: &crate::ranker::engine::ScoredAction,
    state: &NormalizedState,
    candidates: &[ActionCandidate],
) -> Option<usize> {
    match &scored.action_type {
        ActionType::PlayCard { card_id, .. } => candidates
            .iter()
            .position(|candidate| candidate.action_id == format!("combat:play:{card_id}")),
        ActionType::EndTurn => candidates
            .iter()
            .position(|candidate| candidate.action_id == "combat:end"),
        ActionType::UsePotion { potion_name } => unique_potion_candidate(
            state
                .potions
                .iter()
                .filter(|potion| potion.name == *potion_name)
                .filter_map(|potion| {
                    let action_id = format!("combat:potion:{}", potion.slot);
                    candidates
                        .iter()
                        .position(|candidate| candidate.action_id == action_id)
                }),
        ),
    }
}

fn unique_potion_candidate(indices: impl Iterator<Item = usize>) -> Option<usize> {
    let mut unique = indices.collect::<Vec<_>>();
    unique.sort_unstable();
    unique.dedup();
    let [index] = unique.as_slice() else {
        return None;
    };
    Some(*index)
}

#[cfg(test)]
pub fn top_ranked_context(state: &NormalizedState) -> Option<serde_json::Value> {
    if state.screen_type.as_ref().map(|st| st.as_str()) != Some("NONE") {
        return None;
    }
    let scored = crate::ranker::rank(state);
    if scored.is_empty() {
        return None;
    }

    let actions: Vec<serde_json::Value> = scored
        .iter()
        .filter(|s| !s.is_avoid)
        .map(|s| action_entry(s, state))
        .collect();

    Some(serde_json::json!({
        "ranked_suggestions": actions
    }))
}

pub fn top_ranked_context_with_refs(
    state: &NormalizedState,
    candidates: &[ActionCandidate],
) -> Option<serde_json::Value> {
    if state.screen_type.as_ref().map(|st| st.as_str()) != Some("NONE") {
        return None;
    }
    let actions: Vec<_> = crate::ranker::rank(state)
        .iter()
        .filter(|scored| !scored.is_avoid)
        .filter_map(|scored| referenced_action_entry(scored, state, candidates))
        .collect();
    (!actions.is_empty()).then(|| serde_json::json!(actions))
}

#[cfg(test)]
#[path = "../tests/autoplay_combat_adviser_tests.rs"]
mod tests;
