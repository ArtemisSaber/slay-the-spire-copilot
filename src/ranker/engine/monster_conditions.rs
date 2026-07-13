use crate::ranker::context::ActionContext;
use crate::ranker::rules::{MonsterSubPredicates, MonstersCondition, TargetCondition};
use crate::state::MonsterInfo;

pub(super) fn eval_target(condition: &TargetCondition, ctx: &ActionContext) -> bool {
    let Some(target) = &ctx.target else {
        return false;
    };
    let predicate = &condition.target;

    if let Some(power_id) = &predicate.power
        && !has_monster_power(target, power_id)
    {
        return false;
    }
    if let Some(power_id) = &predicate.power_not
        && has_monster_power(target, power_id)
    {
        return false;
    }
    if let Some(power_ids) = &predicate.any_power_in
        && !power_ids
            .iter()
            .any(|power_id| has_monster_power(target, power_id))
    {
        return false;
    }
    if predicate
        .is_scaling
        .is_some_and(|value| target.is_scaling != value)
        || predicate
            .can_be_killed
            .is_some_and(|value| target.can_be_killed != value)
        || predicate
            .intent
            .as_ref()
            .is_some_and(|intent| target.intent.as_deref() != Some(intent))
        || predicate
            .intent_not
            .as_ref()
            .is_some_and(|intent| target.intent.as_deref() == Some(intent))
    {
        return false;
    }
    if let Some(is_minion) = predicate.is_minion
        && has_monster_power(target, "Minion") != is_minion
    {
        return false;
    }
    predicate
        .is_max_hp
        .is_none_or(|value| (target.current_hp == target.max_hp) == value)
}

pub(super) fn eval_monsters(condition: &MonstersCondition, ctx: &ActionContext) -> bool {
    let predicate = &condition.monsters;
    predicate
        .any
        .as_ref()
        .is_none_or(|sub| any_monster_matches(sub, ctx))
        && predicate
            .none
            .as_ref()
            .is_none_or(|sub| !any_monster_matches(sub, ctx))
}

fn any_monster_matches(predicate: &MonsterSubPredicates, ctx: &ActionContext) -> bool {
    ctx.monsters.iter().enumerate().any(|(index, monster)| {
        if predicate.exclude_target == Some(true) && ctx.target_index == Some(index) {
            return false;
        }
        if let Some(monster_id) = &predicate.monster_id
            && monster.monster_id.as_ref() != Some(monster_id)
        {
            return false;
        }
        if predicate
            .is_scaling
            .is_some_and(|value| monster.is_scaling != value)
            || predicate
                .intent
                .as_ref()
                .is_some_and(|intent| monster.intent.as_deref() != Some(intent))
        {
            return false;
        }
        if let Some(is_not_minion) = predicate.is_not_minion
            && has_monster_power(monster, "Minion") == is_not_minion
        {
            return false;
        }
        predicate
            .power
            .as_ref()
            .is_none_or(|power_id| has_monster_power(monster, power_id))
    })
}

fn has_monster_power(monster: &MonsterInfo, power_id: &str) -> bool {
    monster
        .monster_powers
        .iter()
        .any(|power| power.id == power_id)
}
