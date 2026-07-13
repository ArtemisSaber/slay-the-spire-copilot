use std::collections::HashSet;

use super::context::{ActionContext, ActionType};
use super::rules::{Rule, RuleSet};

mod aoe;
mod conditions;
mod monster_conditions;
mod scoring;

#[derive(Debug, Clone)]
pub struct ScoredAction {
    pub action_type: ActionType,
    pub target_index: Option<usize>,
    pub score: i64,
    pub breakdown: Vec<RuleResult>,
    pub is_avoid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleResult {
    pub rule_id: String,
    pub score: i64,
    pub matched: bool,
}

pub fn evaluate(ctx: &ActionContext, rule_set: &RuleSet) -> Vec<RuleResult> {
    let mut rules: Vec<&Rule> = rule_set.rules.iter().collect();
    rules.sort_by_key(|rule| rule.priority);

    let mut results = Vec::new();
    let mut suppressed = HashSet::new();

    for rule in rules {
        let rule_id = &rule.rule_id;
        let matched = rule_applies(rule, &ctx.action_type)
            && !suppressed.contains(rule_id)
            && conditions::eval_conditions(&rule.conditions, ctx, rule, rule_set);

        if matched && let Some(override_target) = &rule.override_rule {
            suppressed.insert(override_target.clone());
        }

        results.push(RuleResult {
            rule_id: rule_id.clone(),
            score: if matched {
                scoring::compute_score(rule, ctx)
            } else {
                0
            },
            matched,
        });
    }

    results
}

pub fn rank_contexts(contexts: &[ActionContext], rule_set: &RuleSet) -> Vec<ScoredAction> {
    let scored: Vec<ScoredAction> = contexts
        .iter()
        .map(|ctx| {
            let breakdown = evaluate(ctx, rule_set);
            let is_avoid = breakdown.iter().any(|result| result.score == i64::MIN);
            let score = if is_avoid {
                i64::MIN
            } else {
                breakdown.iter().map(|result| result.score).sum()
            };
            ScoredAction {
                action_type: ctx.action_type.clone(),
                target_index: ctx.target_index,
                score,
                breakdown,
                is_avoid,
            }
        })
        .collect();

    let mut scored = aoe::merge_groups(scored, contexts, rule_set);
    scored.sort_by_key(|action| std::cmp::Reverse(action.score));

    tracing::debug!(
        "ranker scored {} actions, {} avoided, best_score={}",
        scored.len(),
        scored.iter().filter(|action| action.is_avoid).count(),
        scored.first().map(|action| action.score).unwrap_or(0),
    );
    scored
}

fn rule_applies(rule: &Rule, action_type: &ActionType) -> bool {
    let tag = match action_type {
        ActionType::PlayCard { .. } => "play_card",
        ActionType::UsePotion { .. } => "use_potion",
        ActionType::EndTurn => "end_turn",
    };
    rule.applies_to.iter().any(|applies_to| applies_to == tag)
}

#[cfg(test)]
#[path = "tests/engine_tests/mod.rs"]
mod tests;
