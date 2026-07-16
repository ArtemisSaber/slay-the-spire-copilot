use crate::combat::effects::CardEffect;
use crate::combat::{CombatScanContext, MonsterSnapshot};

#[cfg(test)]
use crate::combat::effects::TargetType;

mod application;
mod powers;

#[cfg(test)]
pub(crate) use application::apply_damage;
pub(crate) use application::apply_damage_from_rendered;
#[cfg(test)]
pub(crate) use application::calc_effective_damage;
pub(crate) use powers::{
    apply_after_card_powers, apply_stance_effect, apply_vulnerable_through_artifact,
    stance_energy_delta,
};

#[cfg(test)]
pub struct PlayBranch {
    pub target_index: Option<usize>,
    #[allow(dead_code, reason = "populated by generate_plays; consumed by callers")]
    pub resolved: CombatScanContext,
}

#[cfg(test)]
#[allow(
    dead_code,
    reason = "test infrastructure: planned play-branch grouping"
)]
pub struct CardPlay {
    pub card_index: usize,
    pub branches: Vec<PlayBranch>,
}

#[cfg(test)]
pub fn generate_plays(
    card_index: usize,
    effect: &CardEffect,
    ctx: &CombatScanContext,
) -> Vec<PlayBranch> {
    let cost = ctx.cards[card_index].cost as i16;
    let required_energy = if effect.x_cost {
        if cost >= 0 { cost } else { 0 }
    } else {
        cost
    };

    if required_energy > ctx.energy {
        return vec![];
    }

    let is_targeted = ctx.cards[card_index].has_target
        || effect
            .damage
            .as_ref()
            .map(|d| d.target_type == TargetType::Targeted)
            .unwrap_or(false)
        || effect.vulnerable.is_some()
        || effect.execute.is_some();

    let living_monsters: Vec<usize> = ctx
        .monsters
        .iter()
        .enumerate()
        .filter(|(_, m)| m.hp > 0)
        .map(|(i, _)| i)
        .collect();

    if is_targeted {
        living_monsters
            .into_iter()
            .map(|mi| PlayBranch {
                target_index: Some(ctx.monsters[mi].command_index),
                resolved: resolve_play(card_index, Some(mi), effect, ctx),
            })
            .collect()
    } else {
        vec![PlayBranch {
            target_index: None,
            resolved: resolve_play(card_index, None, effect, ctx),
        }]
    }
}

pub(crate) fn resolve_play(
    card_index: usize,
    target_monster_idx: Option<usize>,
    effect: &CardEffect,
    ctx: &CombatScanContext,
) -> CombatScanContext {
    let mut monsters = ctx.monsters.clone();
    let cost = ctx.cards[card_index].cost as i16;
    let mut energy = ctx.energy;
    let mut strength_delta = ctx.strength_delta;
    let mut stance = ctx.current_stance;

    let x_value = if effect.x_cost {
        Some(energy + ctx.x_cost_bonus)
    } else {
        None
    };
    energy = if effect.x_cost { 0 } else { energy - cost };

    if let Some(dmg) = &effect.damage {
        apply_damage_from_rendered(
            dmg,
            target_monster_idx,
            x_value,
            &mut monsters,
            ctx.initial_stance,
            stance,
            strength_delta,
        );
    }

    if let Some(vuln_amount) = effect.vulnerable
        && let Some(ti) = target_monster_idx
    {
        apply_vulnerable_through_artifact(&mut monsters[ti], vuln_amount);
    }

    if let Some(exec_threshold) = effect.execute
        && let Some(ti) = target_monster_idx
        && monsters[ti].hp <= exec_threshold
        && monsters[ti].hp > 0
    {
        monsters[ti].hp = 0;
    }

    energy += effect.energy_gain;
    strength_delta += effect.strength_gain;

    let new_stance = apply_stance_effect(&effect.stance, stance);
    energy += stance_energy_delta(stance, new_stance);
    stance = new_stance;

    apply_after_card_powers(&mut monsters);

    let remaining_card_plays = ctx.remaining_card_plays.saturating_sub(1);

    CombatScanContext {
        cards: ctx.cards.clone(),
        energy,
        initial_stance: ctx.initial_stance,
        current_stance: stance,
        strength_delta,
        x_cost_bonus: ctx.x_cost_bonus,
        monsters,
        remaining_card_plays,
    }
}

pub fn combat_ended(monsters: &[MonsterSnapshot]) -> bool {
    if monsters.is_empty() {
        return false;
    }
    monsters.iter().all(|m| m.hp <= 0 || m.is_minion)
}

#[cfg(test)]
#[path = "../tests/damage_tests.rs"]
mod tests;
