use std::collections::{HashMap, HashSet};

use crate::ranker::context::{ActionContext, ActionType};
use crate::ranker::rules::{RuleCategory, RuleSet};

use super::ScoredAction;

pub(super) fn merge_groups(
    scored: Vec<ScoredAction>,
    contexts: &[ActionContext],
    rule_set: &RuleSet,
) -> Vec<ScoredAction> {
    let per_card_ids: HashSet<String> = rule_set
        .rules
        .iter()
        .filter(|rule| rule.category == RuleCategory::PerCard)
        .map(|rule| rule.rule_id.clone())
        .collect();
    let (groups, standalone) = partition_contexts(&scored, contexts);
    let mut result = groups
        .values()
        .filter_map(|group| merge_group(group, &scored, &per_card_ids))
        .collect::<Vec<_>>();
    result.extend(standalone.into_iter().map(|index| scored[index].clone()));

    if !groups.is_empty() {
        let entries = groups.values().map(Vec::len).sum::<usize>();
        tracing::debug!(
            "ranker merged {} AoE groups from {} entries",
            groups.len(),
            entries,
        );
    }
    result
}

fn partition_contexts(
    scored: &[ScoredAction],
    contexts: &[ActionContext],
) -> (HashMap<String, Vec<usize>>, Vec<usize>) {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    let mut standalone = Vec::new();

    for (index, ctx) in contexts.iter().enumerate().take(scored.len()) {
        match (&ctx.action_type, ctx.vars.get("aoe").copied()) {
            (ActionType::PlayCard { card_id, .. }, Some(1.0)) => {
                groups.entry(card_id.clone()).or_default().push(index);
            }
            _ => standalone.push(index),
        }
    }
    (groups, standalone)
}

fn merge_group(
    group: &[usize],
    scored: &[ScoredAction],
    per_card_ids: &HashSet<String>,
) -> Option<ScoredAction> {
    let (&first_index, rest) = group.split_first()?;
    let mut merged = scored[first_index].clone();
    merged.target_index = None;

    for &index in rest {
        for (merged_result, other_result) in
            merged.breakdown.iter_mut().zip(&scored[index].breakdown)
        {
            if !per_card_ids.contains(&merged_result.rule_id) {
                merged_result.score = merged_result.score.saturating_add(other_result.score);
            }
            merged_result.matched |= other_result.matched;
        }
    }

    merged.is_avoid = merged
        .breakdown
        .iter()
        .any(|result| result.score == i64::MIN);
    merged.score = if merged.is_avoid {
        i64::MIN
    } else {
        merged.breakdown.iter().map(|result| result.score).sum()
    };
    Some(merged)
}
