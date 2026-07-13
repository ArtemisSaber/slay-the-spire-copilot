use crate::ranker::context::ActionContext;
use crate::ranker::rules::{CardCondition, Condition, ParsedCondition, Rule, RuleSet};

pub(super) fn eval_conditions(
    conditions: &[Condition],
    ctx: &ActionContext,
    rule: &Rule,
    rule_set: &RuleSet,
) -> bool {
    conditions
        .iter()
        .all(|condition| eval_condition(condition, ctx, rule, rule_set))
}

fn eval_condition(
    condition: &Condition,
    ctx: &ActionContext,
    rule: &Rule,
    rule_set: &RuleSet,
) -> bool {
    match condition {
        Condition::Card(condition) => eval_card(condition, ctx),
        Condition::Parsed(condition) => eval_parsed(condition, ctx),
        Condition::Target(condition) => super::monster_conditions::eval_target(condition, ctx),
        Condition::Monsters(condition) => super::monster_conditions::eval_monsters(condition, ctx),
        Condition::Player(condition) => super::scoring::eval_player(condition, ctx),
        Condition::State(condition) => super::scoring::eval_state(condition, ctx),
        Condition::Compute(condition) => {
            super::scoring::eval_compute(condition, ctx, rule, rule_set)
        }
    }
}

fn eval_card(condition: &CardCondition, ctx: &ActionContext) -> bool {
    let Some(card) = &ctx.card else {
        return false;
    };
    let predicate = &condition.card;

    if let Some(card_type) = &predicate.card_type
        && &card.card_type != card_type
    {
        return false;
    }
    if let Some(types) = &predicate.type_in
        && !types.iter().any(|card_type| card_type == &card.card_type)
    {
        return false;
    }
    if let Some(types) = &predicate.type_not_in
        && types.iter().any(|card_type| card_type == &card.card_type)
    {
        return false;
    }
    if let Some(cost) = predicate.cost_eq
        && card.cost != cost
    {
        return false;
    }
    if let Some(ethereal) = predicate.ethereal {
        let is_ethereal =
            card.description.contains("虚无") || card.description.contains("Ethereal");
        if is_ethereal != ethereal {
            return false;
        }
    }
    if let Some(id) = &predicate.id
        && &card.id != id
    {
        return false;
    }
    predicate
        .id_in
        .as_ref()
        .is_none_or(|ids| ids.iter().any(|id| id == &card.id))
}

fn eval_parsed(condition: &ParsedCondition, ctx: &ActionContext) -> bool {
    let parsed = &ctx.parsed;
    let predicate = &condition.parsed;
    let greater_than = [
        (&predicate.damage_gt, parsed.damage.unwrap_or(0)),
        (&predicate.block_gt, parsed.block.unwrap_or(0)),
        (&predicate.heal_gt, parsed.heal.unwrap_or(0)),
        (&predicate.draw_gt, parsed.draw.unwrap_or(0)),
        (&predicate.self_damage_gt, parsed.self_damage.unwrap_or(0)),
        (&predicate.str_gain_gt, parsed.str_gain.unwrap_or(0)),
        (&predicate.dex_gain_gt, parsed.dex_gain.unwrap_or(0)),
        (&predicate.poison_gt, parsed.poison.unwrap_or(0)),
        (&predicate.vulnerable_gt, parsed.vulnerable.unwrap_or(0)),
        (&predicate.weak_gt, parsed.weak.unwrap_or(0)),
        (&predicate.energy_gain_gt, parsed.energy_gain.unwrap_or(0)),
        (&predicate.focus_gain_gt, parsed.focus_gain.unwrap_or(0)),
        (&predicate.str_loss_gt, parsed.str_loss.unwrap_or(0)),
        (&predicate.mantra_gt, parsed.mantra.unwrap_or(0)),
        (&predicate.orb_slot_expand_gt, parsed.orb_slot_expand),
    ];
    if greater_than
        .iter()
        .any(|(threshold, actual)| threshold.is_some_and(|value| *actual <= value))
    {
        return false;
    }

    let equals = [
        (&predicate.damage_eq, parsed.damage.unwrap_or(0)),
        (&predicate.block_eq, parsed.block.unwrap_or(0)),
    ];
    if equals
        .iter()
        .any(|(threshold, actual)| threshold.is_some_and(|value| *actual != value))
    {
        return false;
    }

    if predicate
        .str_loss_temp
        .is_some_and(|value| parsed.str_loss_temp != value)
        || predicate
            .exhausts_cards
            .is_some_and(|value| (parsed.exhaust_count > 0) != value)
        || predicate
            .exits_stance
            .is_some_and(|value| parsed.exits_stance != value)
    {
        return false;
    }

    if let Some(expected) = predicate.exhausts_status_curse {
        let exhausts_bad_card = parsed.exhaust_count > 0
            && ctx
                .card
                .as_ref()
                .is_some_and(|card| card.card_type == "STATUS" || card.card_type == "CURSE");
        if exhausts_bad_card != expected {
            return false;
        }
    }
    predicate
        .channel_orb
        .as_ref()
        .is_none_or(|orb_type| parsed.channel_orb.as_deref() == Some(orb_type))
}
