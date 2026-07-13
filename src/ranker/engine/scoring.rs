use crate::ranker::context::ActionContext;
use crate::ranker::formula;
use crate::ranker::predicates;
use crate::ranker::rules::{ComputeCondition, PlayerCondition, Rule, RuleSet, StateCondition};

pub(super) fn eval_player(condition: &PlayerCondition, ctx: &ActionContext) -> bool {
    let predicate = &condition.player;
    has_player_tag(ctx, "player_power_", predicate.power.as_deref(), true)
        && has_player_tag(ctx, "player_power_", predicate.power_not.as_deref(), false)
        && has_player_tag(ctx, "player_relic_", predicate.relic.as_deref(), true)
        && has_player_tag(ctx, "player_relic_", predicate.relic_not.as_deref(), false)
        && has_player_tag(ctx, "player_stance_", predicate.stance.as_deref(), true)
        && has_player_tag(
            ctx,
            "player_stance_",
            predicate.stance_not.as_deref(),
            false,
        )
}

fn has_player_tag(ctx: &ActionContext, prefix: &str, id: Option<&str>, expected: bool) -> bool {
    id.is_none_or(|id| {
        let tag = format!("{prefix}{id}");
        (ctx.vars.get(&tag).copied() == Some(1.0)) == expected
    })
}

pub(super) fn eval_state(condition: &StateCondition, ctx: &ActionContext) -> bool {
    let predicate = &condition.state;
    if predicate
        .turn_eq
        .is_some_and(|value| variable_as_i64(ctx, "turn") != value)
    {
        return false;
    }
    if predicate
        .remaining_energy_gt
        .is_some_and(|value| variable_as_i64(ctx, "remaining_energy") <= value)
    {
        return false;
    }
    predicate
        .incoming_damage_gt
        .is_none_or(|value| variable_as_i64(ctx, "incoming_damage") > value)
}

fn variable_as_i64(ctx: &ActionContext, key: &str) -> i64 {
    ctx.vars.get(key).copied().unwrap_or(0.0) as i64
}

pub(super) fn eval_compute(
    condition: &ComputeCondition,
    ctx: &ActionContext,
    rule: &Rule,
    _rule_set: &RuleSet,
) -> bool {
    let Some(formula_str) = &condition.compute.formula else {
        return true;
    };
    match formula::evaluate(formula_str, &weighted_vars(ctx, rule)) {
        Ok(value) => value > 0,
        Err(error) => {
            tracing::warn!(
                "compute formula eval failed for rule {}: {error}",
                rule.rule_id
            );
            false
        }
    }
}

pub(super) fn compute_score(rule: &Rule, ctx: &ActionContext) -> i64 {
    if let Some(function_name) = &rule.score_fn {
        return predicates::dispatch(function_name, &weighted_vars(ctx, rule)).unwrap_or(0);
    }
    if let Some(formula_str) = &rule.formula {
        return formula::evaluate(formula_str, &weighted_vars(ctx, rule)).unwrap_or(0);
    }
    rule.weight.resolve()
}

fn weighted_vars(ctx: &ActionContext, rule: &Rule) -> std::collections::HashMap<String, f64> {
    let mut vars = ctx.vars.clone();
    vars.insert("weight".to_string(), rule.weight.resolve() as f64);
    vars
}
