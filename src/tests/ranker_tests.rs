use crate::ranker;
use crate::ranker::context::ActionType;
use crate::state::NormalizedState;
use crate::test_utils;

fn comm_mod_state(filename: &str) -> NormalizedState {
    let raw = test_utils::load_fixture(filename);
    let locale = crate::locales::Locale::load("zh");
    NormalizedState::from_raw(&raw, &locale)
}

fn find_breakdown<'a>(
    breakdown: &'a [crate::ranker::engine::RuleResult],
    rule_id: &str,
) -> &'a crate::ranker::engine::RuleResult {
    breakdown
        .iter()
        .find(|b| b.rule_id == rule_id)
        .unwrap_or_else(|| panic!("rule {rule_id} not in breakdown"))
}

fn find_scored<'a>(
    scored: &'a [crate::ranker::engine::ScoredAction],
    card_name: &str,
) -> &'a crate::ranker::engine::ScoredAction {
    scored
        .iter()
        .find(|s| match &s.action_type {
            ActionType::PlayCard { card_name: cn, .. } => cn == card_name,
            _ => false,
        })
        .unwrap_or_else(|| panic!("card {card_name} not in scored actions"))
}

#[path = "ranker_tests/core.rs"]
mod core;
#[path = "ranker_tests/fixtures.rs"]
mod fixtures;
#[path = "ranker_tests/potions.rs"]
mod potions;
