use std::collections::HashMap;
use std::time::Instant;

use crate::combat::damage::combat_ended;
use crate::combat::{MonsterSnapshot, Stance};
use crate::state::CardInfo;

/// Check the DFS deadline every this many expanded states (amortizes Instant::now() cost).
const DEADLINE_CHECK_INTERVAL: usize = 1000;

#[derive(Debug, Clone)]
pub(crate) struct PlayStep {
    pub(crate) card_index: usize,
    pub(crate) target: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MemoKey {
    remaining_mask: u32,
    energy: i16,
    stance: u8,
    strength_delta: i16,
    mantra: i16,
    monsters_hash: u64,
}

fn stance_to_u8(s: Stance) -> u8 {
    match s {
        Stance::Neutral => 0,
        Stance::Calm => 1,
        Stance::Wrath => 2,
        Stance::Divinity => 3,
    }
}

fn monsters_hash(monsters: &[MonsterSnapshot]) -> u64 {
    let mut h = 0u64;
    for m in monsters {
        h = h.wrapping_mul(31).wrapping_add(m.hp as u64);
        h = h.wrapping_mul(31).wrapping_add(m.block as u64);
        for p in &m.powers {
            h = h.wrapping_mul(17).wrapping_add(p.amount as u64);
            h = h.wrapping_add(if p.triggered { 1 } else { 0 });
        }
    }
    h
}

pub(crate) struct DfsContext<'a> {
    pub(crate) effects: &'a [Option<crate::combat::effects::CardEffect>],
    pub(crate) cards: &'a [CardInfo],
    pub(crate) max_depth: usize,
    pub(crate) max_states: usize,
    pub(crate) memo: HashMap<MemoKey, Option<Vec<PlayStep>>>,
    pub(crate) expanded: usize,
    pub(crate) deadline: Instant,
    pub(crate) x_cost_bonus: i16,
    pub(crate) initial_stance: Stance,
}

#[allow(
    clippy::too_many_arguments,
    reason = "DFS state params form a natural group"
)]
pub(crate) fn dfs(
    remaining_mask: u32,
    energy: i16,
    depth: usize,
    stance: Stance,
    strength_delta: i16,
    mantra: i16,
    monsters: &[MonsterSnapshot],
    ctx: &mut DfsContext<'_>,
) -> Option<Vec<PlayStep>> {
    if combat_ended(monsters) {
        return Some(vec![]);
    }
    if depth >= ctx.max_depth {
        return None;
    }
    if ctx.expanded >= ctx.max_states {
        return None;
    }

    if ctx.expanded.is_multiple_of(DEADLINE_CHECK_INTERVAL) && Instant::now() > ctx.deadline {
        tracing::debug!(
            "kill_scan DFS deadline expired at {} expanded states",
            ctx.expanded
        );
        return None;
    }

    let key = MemoKey {
        remaining_mask,
        energy,
        stance: stance_to_u8(stance),
        strength_delta,
        mantra,
        monsters_hash: monsters_hash(monsters),
    };
    if let Some(cached) = check_memo(ctx, &key) {
        return cached;
    }
    ctx.expanded += 1;

    let mut mask = remaining_mask;
    let mut card_idx = 0;
    while mask > 0 {
        if mask & 1 != 0
            && let Some(seq) = super::play::try_play_card(
                card_idx,
                remaining_mask,
                energy,
                depth,
                stance,
                strength_delta,
                mantra,
                monsters,
                ctx,
            )
        {
            ctx.memo.insert(key, Some(seq.clone()));
            return Some(seq);
        }
        card_idx += 1;
        mask >>= 1;
    }

    None
}

fn check_memo(ctx: &mut DfsContext<'_>, key: &MemoKey) -> Option<Option<Vec<PlayStep>>> {
    if let Some(result) = ctx.memo.get(key) {
        return Some(result.clone());
    }
    ctx.memo.insert(key.clone(), None);
    None
}
