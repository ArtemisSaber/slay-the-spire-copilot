use std::collections::HashMap;
use std::time::Instant;

use crate::combat::context::build_context;
use crate::combat::effects::parse_card_effect;
use crate::combat::{KillScanOptions, KillSequence, MonsterSnapshot};
use crate::state::{CardInfo, NormalizedState};

mod play;
mod search;

use search::{DfsContext, dfs};

pub(crate) fn find_kill_sequence_inner(
    state: &NormalizedState,
    options: &KillScanOptions,
) -> Option<KillSequence> {
    let started = Instant::now();
    let ctx = build_context(state)?;
    let locale = crate::locales::Locale::load("zh");

    let effects: Vec<_> = ctx
        .cards
        .iter()
        .map(|c| parse_card_effect(c, &locale.effect_parser))
        .collect();

    let relevant: Vec<(usize, &CardInfo, &crate::combat::effects::CardEffect)> = effects
        .iter()
        .enumerate()
        .filter_map(|(i, e)| e.as_ref().map(|eff| (i, &ctx.cards[i], eff)))
        .collect();

    if relevant.is_empty() {
        tracing::debug!("kill_scan: no parseable cards in hand");
        return None;
    }

    let initial_mask: u32 = relevant.iter().fold(0, |m, (i, _, _)| m | (1u32 << i));
    let max_depth = relevant.len().min(ctx.remaining_card_plays);

    tracing::info!(
        "kill_scan searching hand={} energy={} mons={} max_depth={} deadline={:?}",
        ctx.cards.len(),
        ctx.energy,
        ctx.monsters.len(),
        max_depth,
        options.deadline,
    );

    let mut dfs_ctx = DfsContext {
        effects: &effects,
        cards: &ctx.cards,
        max_depth,
        max_states: options.max_expanded_states,
        memo: HashMap::new(),
        expanded: 0,
        deadline: Instant::now() + options.deadline,
        x_cost_bonus: ctx.x_cost_bonus,
        initial_stance: ctx.initial_stance,
    };

    let result = dfs(
        initial_mask,
        ctx.energy,
        0,
        ctx.current_stance,
        ctx.strength_delta,
        0, // initial mantra
        &ctx.monsters,
        &mut dfs_ctx,
    );

    let duration_ms = started.elapsed().as_millis();
    if let Some(steps) = result {
        let seq: KillSequence = steps
            .into_iter()
            .map(|s| crate::combat::KillPlay {
                card: ctx.cards[s.card_index]
                    .uuid
                    .clone()
                    .unwrap_or_else(|| format!("card_{}", s.card_index)),
                target: s.target,
            })
            .collect();
        tracing::info!(
            "kill_scan found lethal {} steps expanded={} memo={} duration_ms={}",
            seq.len(),
            dfs_ctx.expanded,
            dfs_ctx.memo.len(),
            duration_ms,
        );
        Some(seq)
    } else {
        tracing::info!(
            "kill_scan no lethal expanded={} memo={} duration_ms={}",
            dfs_ctx.expanded,
            dfs_ctx.memo.len(),
            duration_ms,
        );
        None
    }
}

pub(crate) fn random_target_guaranteed(
    damage_per_hit: i16,
    hits: i16,
    monsters: &[MonsterSnapshot],
) -> bool {
    for m in monsters {
        if m.hp <= 0 {
            continue;
        }
        for p in &m.powers {
            match p.id.as_str() {
                "Curl Up" | "Flight" | "Malleable" | "Intangible" | "Invincible"
                    if p.amount > 0 =>
                {
                    return false;
                }
                _ => {}
            }
        }
    }

    let total_damage = damage_per_hit as i64 * hits as i64;
    let total_durability: i64 = monsters
        .iter()
        .filter(|m| m.hp > 0)
        .map(|m| m.hp as i64 + m.block as i64)
        .sum();
    let alive_count = monsters.iter().filter(|m| m.hp > 0).count() as i64;
    if alive_count == 0 {
        return true;
    }
    total_damage >= total_durability + (damage_per_hit as i64 - 1) * (alive_count - 1)
}

#[cfg(test)]
#[path = "../tests/kill_scan_test_support.rs"]
mod test_support;
#[cfg(test)]
pub(crate) use test_support::{TestCard, test_scan};

#[cfg(test)]
#[path = "../tests/kill_scan_spec_tests.rs"]
mod tests;
