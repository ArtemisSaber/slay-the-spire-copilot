use std::collections::HashMap;
use std::time::Instant;

use crate::combat::context::build_context;
use crate::combat::damage::{combat_ended, resolve_play};
use crate::combat::effects::parse_card_effect;
use crate::combat::{
    CombatScanContext, KillPlay, KillScanOptions, KillSequence, MonsterSnapshot, Stance,
};
use crate::state::{CardInfo, NormalizedState};

/// Check the DFS deadline every this many expanded states (amortizes Instant::now() cost).
const DEADLINE_CHECK_INTERVAL: usize = 1000;

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
            .map(|s| KillPlay {
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

#[derive(Debug, Clone)]
struct PlayStep {
    card_index: usize,
    target: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MemoKey {
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

struct DfsContext<'a> {
    effects: &'a [Option<crate::combat::effects::CardEffect>],
    cards: &'a [CardInfo],
    max_depth: usize,
    max_states: usize,
    memo: HashMap<MemoKey, Option<Vec<PlayStep>>>,
    expanded: usize,
    deadline: Instant,
    x_cost_bonus: i16,
}

#[allow(
    clippy::too_many_arguments,
    reason = "DFS state params form a natural group"
)]
fn dfs(
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
            && let Some(seq) = try_play_card(
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

#[allow(
    clippy::too_many_arguments,
    reason = "card exploration needs the full DFS state to recurse"
)]
fn try_play_card(
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
    let effect = ctx.effects.get(card_idx).and_then(|e| e.as_ref())?;

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
        initial_stance: stance,
        current_stance: stance,
        strength_delta,
        x_cost_bonus: ctx.x_cost_bonus,
        monsters: monsters.to_vec(),
        remaining_card_plays: 1,
    };

    let is_targeted = effect
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
        if !random_target_guaranteed(dmg.amount, hits, monsters) {
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
        let resolved = resolve_play(card_idx, *target_idx, effect, &play_ctx);

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

        let new_mask = remaining_mask & !(1u32 << card_idx);

        if combat_ended(&resolved.monsters) {
            let cmd_target = target_idx.map(|ti| resolved.monsters[ti].command_index);
            return Some(vec![PlayStep {
                card_index: card_idx,
                target: cmd_target,
            }]);
        }

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

const KNOWN_MONSTER_POWERS: &[&str] = &[
    "Artifact",
    "人工制品",
    "Curl Up",
    "Flight",
    "Intangible",
    "Invincible",
    "Malleable",
    "Minion",
    "爪牙",
    "Slow",
    "缓慢",
    "Vulnerable",
    "易伤",
    "Time Warp",
    "Strength",
    "力量",
    "Fading",
    "消逝",
    "Life Link",
    "生命链接",
    "Shackled",
    "镣铐",
    "Weakened",
    "虚弱",
    "Generic Strength Up Power",
    "强化",
    "Shifting",
    "变化",
    "Plated Armor",
    "多层护甲",
    "Barricade",
    "壁垒",
    "Anger",
    "Ritual",
    "Regeneration",
    "Metallicize",
];

pub(crate) fn has_dangerous_unknown_powers(monsters: &[MonsterSnapshot]) -> bool {
    for m in monsters {
        for p in &m.powers {
            if !KNOWN_MONSTER_POWERS.contains(&p.id.as_str()) {
                return true;
            }
        }
    }
    false
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

pub(crate) struct TestCard {
    pub uuid: &'static str,
    pub name: &'static str,
    pub cost: i16,
    pub card_type: &'static str,
    pub damage: Option<(
        i16,
        crate::combat::effects::HitCount,
        crate::combat::effects::TargetType,
    )>,
    pub vulnerable: Option<i16>,
    pub strength_gain: i16,
    pub energy_gain: i16,
    pub stance: crate::combat::effects::StanceEffect,
    pub mantra_gain: i16,
    pub execute_threshold: Option<i16>,
    pub x_cost: bool,
}

impl TestCard {
    fn to_effect(&self) -> crate::combat::effects::CardEffect {
        use crate::combat::effects::{CardEffect, DamageEffect, ExhaustKind};
        CardEffect {
            damage: self.damage.as_ref().map(|(amount, hits, tt)| DamageEffect {
                amount: *amount,
                hits: hits.clone(),
                target_type: tt.clone(),
            }),
            energy_gain: self.energy_gain,
            strength_gain: self.strength_gain,
            vulnerable: self.vulnerable,
            stance: self.stance.clone(),
            mantra_gain: self.mantra_gain,
            execute: self.execute_threshold,
            exhaust: ExhaustKind::None,
            x_cost: self.x_cost,
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "test_scan takes all scan parameters explicitly"
)]
pub(crate) fn test_scan(
    hand_cards: &[TestCard],
    energy: i16,
    monsters: &[MonsterSnapshot],
    stance: Stance,
    strength_delta: i16,
    x_cost_bonus: i16,
    remaining_plays: usize,
    max_states: usize,
) -> Option<Vec<KillPlay>> {
    if has_dangerous_unknown_powers(monsters) {
        return None;
    }

    let effects: Vec<Option<crate::combat::effects::CardEffect>> =
        hand_cards.iter().map(|tc| Some(tc.to_effect())).collect();

    let cards: Vec<CardInfo> = hand_cards
        .iter()
        .map(|tc| CardInfo {
            id: String::new(),
            name: tc.name.into(),
            cost: tc.cost as i64,
            card_type: tc.card_type.into(),
            upgraded: false,
            uuid: Some(tc.uuid.into()),
            description: String::new(),
            price: None,
            playable: true,
            has_target: tc
                .damage
                .as_ref()
                .map(|(_, _, tt)| matches!(tt, crate::combat::effects::TargetType::Targeted))
                .unwrap_or(false)
                || tc.vulnerable.is_some()
                || tc.execute_threshold.is_some(),
        })
        .collect();

    let initial_mask: u32 = (0..hand_cards.len()).fold(0, |m, i| m | (1u32 << i));
    let max_depth = hand_cards.len().min(remaining_plays);

    let mut dfs_ctx = DfsContext {
        effects: &effects,
        cards: &cards,
        max_depth,
        max_states,
        memo: HashMap::new(),
        expanded: 0,
        deadline: Instant::now() + std::time::Duration::from_secs(30),
        x_cost_bonus,
    };

    let result = dfs(
        initial_mask,
        energy,
        0,
        stance,
        strength_delta,
        0, // initial mantra
        monsters,
        &mut dfs_ctx,
    );

    result.map(|steps| {
        steps
            .into_iter()
            .map(|s| KillPlay {
                card: cards[s.card_index].uuid.clone().unwrap_or_default(),
                target: s.target,
            })
            .collect()
    })
}

#[cfg(test)]
#[path = "../tests/kill_scan_spec_tests.rs"]
mod tests;
