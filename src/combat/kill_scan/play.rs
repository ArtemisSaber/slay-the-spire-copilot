use crate::combat::damage::{combat_ended, resolve_play};
use crate::combat::effects::ExhaustKind;
use crate::combat::{CombatScanContext, MonsterSnapshot, Stance};

use super::search::{DfsContext, PlayStep, dfs};

#[allow(
    clippy::too_many_arguments,
    reason = "card exploration needs the full DFS state to recurse"
)]
pub(crate) fn try_play_card(
    card_idx: usize,
    remaining_mask: u32,
    energy: i16,
    depth: usize,
    stance: Stance,
    strength_delta: i16,
    mantra: i16,
    monsters: &[MonsterSnapshot],
    ctx: &mut DfsContext<'_>,
) -> Option<Vec<PlayStep>> {
    let mut effect = ctx.effects.get(card_idx).and_then(|e| e.as_ref())?.clone();
    if effect.exhaust == ExhaustKind::ExhaustAllDamagePerCard
        && let Some(damage) = effect.damage.as_mut()
    {
        let exhausted_cards = remaining_mask.count_ones().saturating_sub(1) as i16;
        damage.hits = crate::combat::effects::HitCount::Fixed(exhausted_cards);
    }

    let cost = if effect.x_cost {
        energy
    } else {
        ctx.cards[card_idx].cost as i16
    };

    if !effect.x_cost && cost > energy {
        return None;
    }

    let play_ctx = CombatScanContext {
        cards: ctx.cards.to_vec(),
        energy,
        initial_stance: ctx.initial_stance,
        current_stance: stance,
        strength_delta,
        x_cost_bonus: ctx.x_cost_bonus,
        monsters: monsters.to_vec(),
        remaining_card_plays: 1,
    };

    let is_targeted = ctx.cards[card_idx].has_target
        || effect
            .damage
            .as_ref()
            .map(|d| matches!(d.target_type, crate::combat::effects::TargetType::Targeted))
            .unwrap_or(false)
        || effect.vulnerable.is_some()
        || effect.execute.is_some();

    let is_random = effect
        .damage
        .as_ref()
        .map(|d| {
            matches!(
                d.target_type,
                crate::combat::effects::TargetType::RandomTarget
            )
        })
        .unwrap_or(false);

    if is_random && let Some(dmg) = &effect.damage {
        let hits = match &dmg.hits {
            crate::combat::effects::HitCount::Fixed(n) => *n,
            _ => return None,
        };
        if !super::random_target_guaranteed(dmg.amount, hits, monsters) {
            return None;
        }
    }

    let living: Vec<usize> = monsters
        .iter()
        .enumerate()
        .filter(|(_, m)| m.hp > 0)
        .map(|(i, _)| i)
        .collect();

    let targets: Vec<Option<usize>> = if is_targeted {
        living.iter().map(|&mi| Some(mi)).collect()
    } else {
        vec![None]
    };

    for target_idx in &targets {
        let resolved = resolve_play(card_idx, *target_idx, &effect, &play_ctx);

        let mut new_energy = resolved.energy;
        let mut new_stance = resolved.current_stance;
        let mut new_mantra = mantra + effect.mantra_gain;

        if new_mantra >= 10 {
            new_mantra = 0;
            if new_stance == Stance::Calm {
                new_energy += 2;
            }
            new_stance = Stance::Divinity;
            new_energy += 3;
        }

        if combat_ended(&resolved.monsters) {
            let cmd_target = target_idx.map(|ti| resolved.monsters[ti].command_index);
            return Some(vec![PlayStep {
                card_index: card_idx,
                target: cmd_target,
            }]);
        }

        let Some(new_mask) =
            remaining_mask_after_exhaust(remaining_mask, card_idx, &effect.exhaust, ctx.cards)
        else {
            continue;
        };

        if let Some(mut suffix) = dfs(
            new_mask,
            new_energy,
            depth + 1,
            new_stance,
            resolved.strength_delta,
            new_mantra,
            &resolved.monsters,
            ctx,
        ) {
            let cmd_target = target_idx.map(|ti| resolved.monsters[ti].command_index);
            suffix.insert(
                0,
                PlayStep {
                    card_index: card_idx,
                    target: cmd_target,
                },
            );
            return Some(suffix);
        }
    }

    None
}

fn remaining_mask_after_exhaust(
    remaining_mask: u32,
    card_idx: usize,
    exhaust: &ExhaustKind,
    cards: &[crate::state::CardInfo],
) -> Option<u32> {
    let played_removed = remaining_mask & !(1u32 << card_idx);
    match exhaust {
        ExhaustKind::None | ExhaustKind::Self_ => Some(played_removed),
        ExhaustKind::All | ExhaustKind::ExhaustAllDamagePerCard => Some(0),
        ExhaustKind::NonAttacks => Some(filter_remaining_cards(played_removed, cards, |card| {
            card.card_type == "ATTACK"
        })),
        ExhaustKind::Attacks => Some(filter_remaining_cards(played_removed, cards, |card| {
            card.card_type != "ATTACK"
        })),
        ExhaustKind::Random | ExhaustKind::Chosen => None,
    }
}

fn filter_remaining_cards(
    remaining_mask: u32,
    cards: &[crate::state::CardInfo],
    keep: impl Fn(&crate::state::CardInfo) -> bool,
) -> u32 {
    cards
        .iter()
        .enumerate()
        .filter(|(index, card)| remaining_mask & (1u32 << index) != 0 && keep(card))
        .fold(0, |mask, (index, _)| mask | (1u32 << index))
}
