use std::collections::HashSet;

use super::context::{ActionContext, ActionType};
use super::formula;
use super::predicates;
use super::rules::{Condition, Rule, RuleSet};

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
    rules.sort_by_key(|r| r.priority);

    let mut results = Vec::new();
    let mut suppressed: HashSet<String> = HashSet::new();

    for rule in &rules {
        let rule_id = &rule.rule_id;

        if !rule_applies(rule, &ctx.action_type) {
            results.push(RuleResult {
                rule_id: rule_id.clone(),
                score: 0,
                matched: false,
            });
            continue;
        }

        if suppressed.contains(rule_id) {
            results.push(RuleResult {
                rule_id: rule_id.clone(),
                score: 0,
                matched: false,
            });
            continue;
        }

        if !eval_conditions(&rule.conditions, ctx, rule, rule_set) {
            results.push(RuleResult {
                rule_id: rule_id.clone(),
                score: 0,
                matched: false,
            });
            continue;
        }

        let score = compute_score(rule, ctx);

        if let Some(ref override_target) = rule.override_rule {
            suppressed.insert(override_target.clone());
        }

        results.push(RuleResult {
            rule_id: rule_id.clone(),
            score,
            matched: true,
        });
    }

    results
}

pub fn rank_contexts(contexts: &[ActionContext], rule_set: &RuleSet) -> Vec<ScoredAction> {
    let mut scored: Vec<ScoredAction> = contexts
        .iter()
        .map(|ctx| {
            let breakdown = evaluate(ctx, rule_set);
            let is_avoid = breakdown.iter().any(|r| r.score == i64::MIN);
            let total = if is_avoid {
                i64::MIN
            } else {
                breakdown.iter().map(|r| r.score).sum()
            };
            ScoredAction {
                action_type: ctx.action_type.clone(),
                target_index: ctx.target_index,
                score: total,
                breakdown,
                is_avoid,
            }
        })
        .collect();

    scored.sort_by(|a, b| b.score.cmp(&a.score));
    scored
}

fn rule_applies(rule: &Rule, action_type: &ActionType) -> bool {
    let tag = match action_type {
        ActionType::PlayCard { .. } => "play_card",
        ActionType::UsePotion { .. } => "use_potion",
        ActionType::EndTurn => "end_turn",
    };
    rule.applies_to.iter().any(|a| a == tag)
}

fn eval_conditions(
    conditions: &[Condition],
    ctx: &ActionContext,
    rule: &Rule,
    rule_set: &RuleSet,
) -> bool {
    conditions
        .iter()
        .all(|cond| eval_condition(cond, ctx, rule, rule_set))
}

fn eval_condition(cond: &Condition, ctx: &ActionContext, rule: &Rule, rule_set: &RuleSet) -> bool {
    match cond {
        Condition::Card(c) => eval_card(c, ctx),
        Condition::Parsed(p) => eval_parsed(p, ctx),
        Condition::Target(t) => eval_target(t, ctx),
        Condition::Monsters(m) => eval_monsters(m, ctx),
        Condition::Player(p) => eval_player(p, ctx),
        Condition::State(s) => eval_state(s, ctx),
        Condition::Compute(c) => eval_compute(c, ctx, rule, rule_set),
    }
}

fn eval_card(cond: &super::rules::CardCondition, ctx: &ActionContext) -> bool {
    let card = match &ctx.card {
        Some(c) => c,
        None => return false,
    };
    let p = &cond.card;

    if let Some(ref t) = p.card_type
        && &card.card_type != t
    {
        return false;
    }
    if let Some(ref types) = p.type_in
        && !types.iter().any(|t| t == &card.card_type)
    {
        return false;
    }
    if let Some(ref types) = p.type_not_in
        && types.iter().any(|t| t == &card.card_type)
    {
        return false;
    }
    if let Some(cost) = p.cost_eq
        && card.cost != cost
    {
        return false;
    }
    if let Some(ethereal) = p.ethereal {
        let is_ethereal =
            card.description.contains("虚无") || card.description.contains("Ethereal");
        if is_ethereal != ethereal {
            return false;
        }
    }
    if let Some(ref id) = p.id
        && &card.id != id
    {
        return false;
    }
    if let Some(ref ids) = p.id_in
        && !ids.iter().any(|i| i == &card.id)
    {
        return false;
    }
    true
}

fn eval_parsed(cond: &super::rules::ParsedCondition, ctx: &ActionContext) -> bool {
    let parsed = &ctx.parsed;
    let p = &cond.parsed;

    if let Some(v) = p.damage_gt
        && parsed.damage.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.damage_eq
        && parsed.damage.unwrap_or(0) != v
    {
        return false;
    }
    if let Some(v) = p.block_gt
        && parsed.block.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.block_eq
        && parsed.block.unwrap_or(0) != v
    {
        return false;
    }
    if let Some(v) = p.heal_gt
        && parsed.heal.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.draw_gt
        && parsed.draw.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.self_damage_gt
        && parsed.self_damage.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.str_gain_gt
        && parsed.str_gain.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.dex_gain_gt
        && parsed.dex_gain.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.poison_gt
        && parsed.poison.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.vulnerable_gt
        && parsed.vulnerable.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.weak_gt
        && parsed.weak.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.energy_gain_gt
        && parsed.energy_gain.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.focus_gain_gt
        && parsed.focus_gain.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.str_loss_gt
        && parsed.str_loss.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.str_loss_temp
        && parsed.str_loss_temp != v
    {
        return false;
    }
    if let Some(v) = p.mantra_gt
        && parsed.mantra.unwrap_or(0) <= v
    {
        return false;
    }
    if let Some(v) = p.exhausts_cards
        && (parsed.exhaust_count > 0) != v
    {
        return false;
    }
    if let Some(v) = p.exhausts_status_curse {
        let exhausts_bad_card = parsed.exhaust_count > 0
            && ctx
                .card
                .as_ref()
                .is_some_and(|c| c.card_type == "STATUS" || c.card_type == "CURSE");
        if exhausts_bad_card != v {
            return false;
        }
    }
    if let Some(ref orb_type) = p.channel_orb
        && parsed.channel_orb.as_deref() != Some(orb_type.as_str())
    {
        return false;
    }
    if let Some(v) = p.orb_slot_expand_gt
        && parsed.orb_slot_expand <= v
    {
        return false;
    }
    if let Some(v) = p.exits_stance
        && parsed.exits_stance != v
    {
        return false;
    }
    true
}

fn eval_target(cond: &super::rules::TargetCondition, ctx: &ActionContext) -> bool {
    let target = match &ctx.target {
        Some(t) => t,
        None => return false,
    };
    let p = &cond.target;

    if let Some(ref power_id) = p.power
        && !has_monster_power(target, power_id)
    {
        return false;
    }
    if let Some(ref power_id) = p.power_not
        && has_monster_power(target, power_id)
    {
        return false;
    }
    if let Some(ref powers) = p.any_power_in
        && !powers.iter().any(|pid| has_monster_power(target, pid))
    {
        return false;
    }
    if let Some(v) = p.is_scaling
        && target.is_scaling != v
    {
        return false;
    }
    if let Some(v) = p.is_minion {
        let is_minion = has_monster_power(target, "Minion");
        if is_minion != v {
            return false;
        }
    }
    if let Some(v) = p.is_max_hp {
        let at_max = target.current_hp == target.max_hp;
        if at_max != v {
            return false;
        }
    }
    if let Some(v) = p.can_be_killed
        && target.can_be_killed != v
    {
        return false;
    }
    if let Some(ref intent) = p.intent
        && target.intent.as_deref() != Some(intent)
    {
        return false;
    }
    if let Some(ref intent) = p.intent_not
        && target.intent.as_deref() == Some(intent)
    {
        return false;
    }
    true
}

fn eval_monsters(cond: &super::rules::MonstersCondition, ctx: &ActionContext) -> bool {
    let p = &cond.monsters;

    if let Some(ref any) = p.any
        && !any_monster_matches(any, ctx)
    {
        return false;
    }
    if let Some(ref none) = p.none
        && any_monster_matches(none, ctx)
    {
        return false;
    }
    true
}

fn any_monster_matches(sub: &super::rules::MonsterSubPredicates, ctx: &ActionContext) -> bool {
    let monsters = &ctx.monsters;
    let target_idx = ctx.target_index;

    monsters.iter().enumerate().any(|(i, m)| {
        if let Some(true) = sub.exclude_target
            && target_idx == Some(i)
        {
            return false;
        }

        if let Some(ref monster_id) = sub.monster_id {
            let matches = m.monster_id.as_ref().is_some_and(|mid| mid == monster_id);
            if !matches {
                return false;
            }
        }

        if let Some(v) = sub.is_scaling
            && m.is_scaling != v
        {
            return false;
        }
        if let Some(v) = sub.is_not_minion {
            let is_minion = has_monster_power(m, "Minion");
            let is_not_minion = !is_minion;
            if is_not_minion != v {
                return false;
            }
        }
        if let Some(ref power_id) = sub.power
            && !has_monster_power(m, power_id)
        {
            return false;
        }
        if let Some(ref intent) = sub.intent
            && m.intent.as_deref() != Some(intent)
        {
            return false;
        }

        true
    })
}

fn eval_player(cond: &super::rules::PlayerCondition, ctx: &ActionContext) -> bool {
    let p = &cond.player;

    if let Some(ref power_id) = p.power {
        let tag = format!("player_power_{power_id}");
        if ctx.vars.get(&tag).copied() != Some(1.0) {
            return false;
        }
    }
    if let Some(ref power_id) = p.power_not {
        let tag = format!("player_power_{power_id}");
        if ctx.vars.get(&tag).copied() == Some(1.0) {
            return false;
        }
    }
    if let Some(ref relic_id) = p.relic {
        let tag = format!("player_relic_{relic_id}");
        if ctx.vars.get(&tag).copied() != Some(1.0) {
            return false;
        }
    }
    if let Some(ref relic_id) = p.relic_not {
        let tag = format!("player_relic_{relic_id}");
        if ctx.vars.get(&tag).copied() == Some(1.0) {
            return false;
        }
    }
    if let Some(ref stance) = p.stance {
        let tag = format!("player_stance_{stance}");
        if ctx.vars.get(&tag).copied() != Some(1.0) {
            return false;
        }
    }
    if let Some(ref stance) = p.stance_not {
        let tag = format!("player_stance_{stance}");
        if ctx.vars.get(&tag).copied() == Some(1.0) {
            return false;
        }
    }
    true
}

fn eval_state(cond: &super::rules::StateCondition, ctx: &ActionContext) -> bool {
    let p = &cond.state;

    if let Some(v) = p.turn_eq
        && ctx.vars.get("turn").copied().unwrap_or(0.0) as i64 != v
    {
        return false;
    }
    if let Some(v) = p.remaining_energy_gt {
        let energy = ctx.vars.get("remaining_energy").copied().unwrap_or(0.0) as i64;
        if energy <= v {
            return false;
        }
    }
    if let Some(v) = p.incoming_damage_gt {
        let dmg = ctx.vars.get("incoming_damage").copied().unwrap_or(0.0) as i64;
        if dmg <= v {
            return false;
        }
    }
    true
}

fn eval_compute(
    cond: &super::rules::ComputeCondition,
    ctx: &ActionContext,
    rule: &Rule,
    _rule_set: &RuleSet,
) -> bool {
    if let Some(ref formula_str) = cond.compute.formula {
        let mut vars = ctx.vars.clone();
        vars.insert("weight".to_string(), rule.weight.resolve() as f64);
        match formula::evaluate(formula_str, &vars) {
            Ok(v) => v > 0,
            Err(e) => {
                tracing::warn!("compute formula eval failed for rule {}: {e}", rule.rule_id);
                false
            }
        }
    } else {
        true
    }
}

fn compute_score(rule: &Rule, ctx: &ActionContext) -> i64 {
    if rule.per_target {
        let monster_count = ctx.monsters.len();
        if monster_count == 0 {
            return 0;
        }
        let mut total: i64 = 0;
        for _i in 0..monster_count {
            total = total.saturating_add(compute_single_score(rule, ctx));
        }
        return total;
    }

    compute_single_score(rule, ctx)
}

fn compute_single_score(rule: &Rule, ctx: &ActionContext) -> i64 {
    if let Some(ref fn_name) = rule.score_fn {
        let mut vars = ctx.vars.clone();
        vars.insert("weight".to_string(), rule.weight.resolve() as f64);
        return predicates::dispatch(fn_name, &vars).unwrap_or(0);
    }

    if let Some(ref formula_str) = rule.formula {
        let mut vars = ctx.vars.clone();
        vars.insert("weight".to_string(), rule.weight.resolve() as f64);
        return formula::evaluate(formula_str, &vars).unwrap_or(0);
    }

    rule.weight.resolve()
}

fn has_monster_power(monster: &crate::state::MonsterInfo, power_id: &str) -> bool {
    monster.monster_powers.iter().any(|p| p.id == power_id)
}

#[cfg(test)]
#[path = "tests/engine_tests.rs"]
mod tests;
