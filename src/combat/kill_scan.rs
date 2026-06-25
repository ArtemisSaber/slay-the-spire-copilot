use std::collections::HashMap;

use crate::combat::context::build_context;
use crate::combat::damage::{combat_ended, resolve_play};
use crate::combat::effects::parse_card_effect;
use crate::combat::{
    CombatScanContext, KillPlay, KillScanOptions, KillSequence, MonsterSnapshot, Stance,
};
use crate::state::{CardInfo, NormalizedState};

pub(crate) fn find_kill_sequence_inner(
    state: &NormalizedState,
    options: &KillScanOptions,
) -> Option<KillSequence> {
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
        return None;
    }

    let initial_mask: u16 = relevant.iter().fold(0, |m, (i, _, _)| m | (1u16 << i));
    let max_depth = relevant.len().min(ctx.remaining_card_plays);

    let mut dfs_ctx = DfsContext {
        effects: &effects,
        cards: &ctx.cards,
        max_depth,
        max_states: options.max_expanded_states,
        memo: HashMap::new(),
        expanded: 0,
    };

    let result = dfs(
        initial_mask,
        ctx.energy,
        0,
        ctx.current_stance,
        ctx.strength_delta,
        &ctx.monsters,
        &mut dfs_ctx,
    );

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
        Some(seq)
    } else {
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
    remaining_mask: u16,
    energy: i16,
    stance: u8,
    strength_delta: i16,
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
}

fn dfs(
    remaining_mask: u16,
    energy: i16,
    depth: usize,
    stance: Stance,
    strength_delta: i16,
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

    let key = MemoKey {
        remaining_mask,
        energy,
        stance: stance_to_u8(stance),
        strength_delta,
        monsters_hash: monsters_hash(monsters),
    };
    if let Some(result) = ctx.memo.get(&key) {
        return result.clone();
    }
    ctx.memo.insert(key.clone(), None);

    ctx.expanded += 1;

    let mut mask = remaining_mask;
    let mut card_idx = 0;

    while mask > 0 {
        if mask & 1 == 0 {
            card_idx += 1;
            mask >>= 1;
            continue;
        }

        let effect = match ctx.effects.get(card_idx).and_then(|e| e.as_ref()) {
            Some(e) => e,
            None => {
                card_idx += 1;
                mask >>= 1;
                continue;
            }
        };

        let cost = if effect.x_cost {
            energy
        } else {
            ctx.cards[card_idx].cost as i16
        };

        if !effect.x_cost && cost > energy {
            card_idx += 1;
            mask >>= 1;
            continue;
        }

        let play_ctx = CombatScanContext {
            cards: ctx.cards.to_vec(),
            energy,
            initial_stance: stance,
            current_stance: stance,
            strength_delta,
            x_cost_bonus: 0,
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

        if is_random
            && let Some(dmg) = &effect.damage
            && !random_target_guaranteed(dmg.amount, dmg.hits, monsters)
        {
            card_idx += 1;
            mask >>= 1;
            continue;
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

            let new_energy = if effect.x_cost {
                resolved.energy
            } else {
                energy - cost + effect.energy_gain
            };

            let new_mask = remaining_mask & !(1u16 << card_idx);

            if combat_ended(&resolved.monsters) {
                let cmd_target = target_idx.map(|ti| resolved.monsters[ti].command_index);
                let seq = vec![PlayStep {
                    card_index: card_idx,
                    target: cmd_target,
                }];
                ctx.memo.insert(key, Some(seq.clone()));
                return Some(seq);
            }

            if let Some(mut suffix) = dfs(
                new_mask,
                new_energy,
                depth + 1,
                resolved.current_stance,
                resolved.strength_delta,
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
                ctx.memo.insert(key, Some(suffix.clone()));
                return Some(suffix);
            }
        }

        card_idx += 1;
        mask >>= 1;
    }

    None
}

fn random_target_guaranteed(damage_per_hit: i16, hits: i16, monsters: &[MonsterSnapshot]) -> bool {
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
mod tests {
    use crate::combat::MonsterSnapshot;
    use crate::combat::damage::combat_ended;

    use super::random_target_guaranteed;

    #[test]
    fn combat_ended_all_dead() {
        let monsters = vec![MonsterSnapshot {
            command_index: 0,
            hp: 0,
            block: 0,
            powers: vec![],
            is_minion: false,
        }];
        assert!(combat_ended(&monsters));
    }

    #[test]
    fn combat_ended_all_minions() {
        let monsters = vec![MonsterSnapshot {
            command_index: 0,
            hp: 10,
            block: 0,
            powers: vec![],
            is_minion: true,
        }];
        assert!(combat_ended(&monsters));
    }

    #[test]
    fn combat_ended_one_alive_non_minion() {
        let monsters = vec![
            MonsterSnapshot {
                command_index: 0,
                hp: 0,
                block: 0,
                powers: vec![],
                is_minion: false,
            },
            MonsterSnapshot {
                command_index: 1,
                hp: 10,
                block: 0,
                powers: vec![],
                is_minion: false,
            },
        ];
        assert!(!combat_ended(&monsters));
    }

    #[test]
    fn combat_ended_dead_and_minion() {
        let monsters = vec![
            MonsterSnapshot {
                command_index: 0,
                hp: 0,
                block: 0,
                powers: vec![],
                is_minion: false,
            },
            MonsterSnapshot {
                command_index: 1,
                hp: 10,
                block: 0,
                powers: vec![],
                is_minion: true,
            },
        ];
        assert!(combat_ended(&monsters));
    }

    fn dummy_monster(hp: i16, block: i16) -> MonsterSnapshot {
        MonsterSnapshot {
            command_index: 0,
            hp,
            block,
            powers: vec![],
            is_minion: false,
        }
    }

    #[test]
    fn combat_ended_empty_monsters() {
        assert!(!combat_ended(&[]));
    }

    #[test]
    fn random_target_4x3_vs_two_4hp() {
        let monsters = vec![dummy_monster(4, 0), dummy_monster(4, 0)];
        assert!(random_target_guaranteed(3, 4, &monsters));
    }

    #[test]
    fn random_target_4x3_vs_two_5hp_rejected() {
        let monsters = vec![dummy_monster(6, 0), dummy_monster(5, 0)];
        assert!(!random_target_guaranteed(3, 4, &monsters));
    }

    #[test]
    fn random_target_3x5_vs_three_3hp() {
        let monsters = vec![
            dummy_monster(3, 0),
            dummy_monster(3, 0),
            dummy_monster(3, 0),
        ];
        assert!(random_target_guaranteed(3, 5, &monsters));
    }

    #[test]
    fn random_target_single_monster_guaranteed() {
        let monsters = vec![dummy_monster(3, 0)];
        assert!(random_target_guaranteed(3, 4, &monsters));
    }

    #[test]
    fn random_target_single_monster_rejected() {
        let monsters = vec![dummy_monster(100, 0)];
        assert!(!random_target_guaranteed(3, 4, &monsters));
    }

    #[test]
    fn random_target_empty_monsters() {
        let monsters = vec![dummy_monster(0, 0)];
        assert!(random_target_guaranteed(3, 4, &monsters));
    }
}
