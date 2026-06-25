use crate::combat::effects::{CardEffect, DamageEffect, HitCount, StanceEffect, TargetType};
use crate::combat::{CombatScanContext, MonsterSnapshot, PowerState, Stance};

pub struct PlayBranch {
    pub target_index: Option<usize>,
    pub resolved: CombatScanContext,
}

pub struct CardPlay {
    pub card_index: usize,
    pub branches: Vec<PlayBranch>,
}

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

    let is_targeted = effect
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
        apply_damage(
            dmg,
            target_monster_idx,
            x_value,
            &mut monsters,
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

pub(crate) fn apply_damage(
    dmg: &DamageEffect,
    target_idx: Option<usize>,
    x_value: Option<i16>,
    monsters: &mut [MonsterSnapshot],
    stance: Stance,
    strength_delta: i16,
) {
    let (base_per_hit, hits) = match &dmg.hits {
        HitCount::Fixed(n) => {
            let per_hit = if let Some(xv) = x_value {
                dmg.amount * xv
            } else {
                dmg.amount
            };
            (per_hit, *n)
        }
        HitCount::XTimes => {
            let xv = x_value.unwrap_or(0);
            (dmg.amount * xv, xv)
        }
        HitCount::XPlus(offset) => {
            let xv = x_value.unwrap_or(0);
            let total_hits = xv + *offset;
            (dmg.amount, total_hits)
        }
    };

    match dmg.target_type {
        TargetType::AoE => {
            for _ in 0..hits {
                for mi in 0..monsters.len() {
                    if monsters[mi].hp <= 0 {
                        continue;
                    }
                    pre_hit_curl_up(&mut monsters[mi]);
                    let effective =
                        calc_effective_damage(base_per_hit, &monsters[mi], stance, strength_delta);
                    let unblocked = apply_block(&mut monsters[mi], effective);
                    apply_hp_loss_with_invincible(monsters, mi, unblocked);
                    post_hit_effects(&mut monsters[mi], unblocked > 0);
                }
            }
            apply_pending_curl_up_blocks(monsters);
        }
        TargetType::Targeted => {
            let ti = target_idx.expect("targeted damage requires target");
            for _ in 0..hits {
                if monsters[ti].hp <= 0 {
                    break;
                }
                pre_hit_curl_up(&mut monsters[ti]);
                let effective =
                    calc_effective_damage(base_per_hit, &monsters[ti], stance, strength_delta);
                let unblocked = apply_block(&mut monsters[ti], effective);
                apply_hp_loss_with_invincible(monsters, ti, unblocked);
                post_hit_effects(&mut monsters[ti], unblocked > 0);
            }
            apply_pending_curl_up_blocks(monsters);
        }
        TargetType::RandomTarget => {
            for _ in 0..hits {
                let living: Vec<usize> = (0..monsters.len())
                    .filter(|&i| monsters[i].hp > 0)
                    .collect();
                if living.is_empty() {
                    break;
                }
                let ti = living[0];
                pre_hit_curl_up(&mut monsters[ti]);
                let effective =
                    calc_effective_damage(base_per_hit, &monsters[ti], stance, strength_delta);
                let unblocked = apply_block(&mut monsters[ti], effective);
                apply_hp_loss_with_invincible(monsters, ti, unblocked);
                post_hit_effects(&mut monsters[ti], unblocked > 0);
            }
            apply_pending_curl_up_blocks(monsters);
        }
    }
}

fn calc_effective_damage(
    base: i16,
    monster: &MonsterSnapshot,
    stance: Stance,
    strength_delta: i16,
) -> i16 {
    let stance_mult = stance_damage_multiplier(stance);
    let initial_mult = stance_damage_multiplier(Stance::Neutral);
    let damage = if initial_mult > 0 {
        (base as i32 * stance_mult as i32 / initial_mult as i32) as i16
    } else {
        base
    };
    let damage = damage + strength_delta * stance_mult;
    let damage = damage.max(0);

    if has_power(monster, "Intangible") && damage > 0 {
        return 1;
    }

    let mut damage = damage as f64;

    if has_power(monster, "Slow") {
        let slow_amount = monster
            .powers
            .iter()
            .find(|p| p.id == "Slow" || p.id == "缓慢")
            .map(|p| p.amount)
            .unwrap_or(0) as f64;
        damage = (damage * (1.0 + 0.1 * slow_amount)).floor();
    }

    if has_power(monster, "Vulnerable") || monster.powers.iter().any(|p| p.id == "易伤") {
        damage = (damage * 1.5).floor();
    }

    if has_power(monster, "Flight") {
        damage = (damage * 0.5).floor();
    }

    damage as i16
}

fn stance_damage_multiplier(stance: Stance) -> i16 {
    match stance {
        Stance::Neutral => 1,
        Stance::Calm => 1,
        Stance::Wrath => 2,
        Stance::Divinity => 3,
    }
}

fn has_power(monster: &MonsterSnapshot, id: &str) -> bool {
    monster.powers.iter().any(|p| p.id == id && p.amount > 0)
}

fn apply_block(monster: &mut MonsterSnapshot, damage: i16) -> i16 {
    if monster.block >= damage {
        monster.block -= damage;
        0
    } else {
        let overflow = damage - monster.block;
        monster.block = 0;
        overflow
    }
}

fn apply_hp_loss_with_invincible(monsters: &mut [MonsterSnapshot], idx: usize, unblocked: i16) {
    if unblocked <= 0 {
        return;
    }

    let cap = monsters[idx]
        .powers
        .iter()
        .find(|p| p.id == "Invincible")
        .map(|p| p.amount)
        .unwrap_or(i16::MAX);

    let actual_loss = unblocked.min(cap);
    let new_hp = (monsters[idx].hp as i32 - actual_loss as i32).max(0) as i16;
    monsters[idx].hp = new_hp;

    if let Some(p) = monsters[idx]
        .powers
        .iter_mut()
        .find(|p| p.id == "Invincible")
    {
        p.amount = (p.amount as i32 - actual_loss as i32).max(0) as i16;
    }
}

fn pre_hit_curl_up(monster: &mut MonsterSnapshot) {
    if let Some(p) = monster
        .powers
        .iter_mut()
        .find(|p| (p.id == "Curl Up") && !p.triggered && p.amount > 0)
    {
        p.triggered = true;
    }
}

fn apply_pending_curl_up_blocks(monsters: &mut [MonsterSnapshot]) {
    for m in monsters.iter_mut() {
        if let Some(p) = m.powers.iter().find(|p| p.id == "Curl Up" && p.triggered) {
            let amount = p.amount;
            m.block += amount;
        }
        if let Some(p) = m.powers.iter_mut().find(|p| p.id == "Curl Up") {
            p.amount = 0;
        }
    }
}

fn post_hit_effects(monster: &mut MonsterSnapshot, did_unblocked_damage: bool) {
    if has_power(monster, "Flight")
        && let Some(p) = monster
            .powers
            .iter_mut()
            .find(|p| p.id == "Flight" && p.amount > 0)
    {
        p.amount -= 1;
        if p.amount < 0 {
            p.amount = 0;
        }
    }

    if did_unblocked_damage
        && let Some(p) = monster
            .powers
            .iter_mut()
            .find(|p| p.id == "Malleable" && p.amount > 0)
    {
        monster.block += p.amount;
        p.amount += 1;
    }
}

pub(crate) fn apply_vulnerable_through_artifact(monster: &mut MonsterSnapshot, amount: i16) {
    let artifact_idx = monster
        .powers
        .iter()
        .position(|p| p.id == "Artifact" && p.amount > 0);
    if let Some(ai) = artifact_idx {
        monster.powers[ai].amount -= 1;
        if monster.powers[ai].amount < 0 {
            monster.powers[ai].amount = 0;
        }
        return;
    }

    if let Some(p) = monster
        .powers
        .iter_mut()
        .find(|p| p.id == "Vulnerable" || p.id == "易伤")
    {
        p.amount += amount;
    } else {
        monster.powers.push(PowerState {
            id: "Vulnerable".into(),
            amount,
            triggered: false,
        });
    }
}

pub(crate) fn apply_after_card_powers(monsters: &mut [MonsterSnapshot]) {
    for m in monsters.iter_mut() {
        if let Some(p) = m
            .powers
            .iter_mut()
            .find(|p| (p.id == "Slow" || p.id == "缓慢") && p.amount > 0)
        {
            p.amount += 1;
        }
    }
}

fn apply_stance_effect(effect: &StanceEffect, current: Stance) -> Stance {
    match effect {
        StanceEffect::None => current,
        StanceEffect::EnterWrath => Stance::Wrath,
        StanceEffect::EnterCalm => Stance::Calm,
        StanceEffect::ExitStance => Stance::Neutral,
        StanceEffect::EnterDivinity => Stance::Divinity,
    }
}

fn stance_energy_delta(from: Stance, to: Stance) -> i16 {
    let mut delta = 0i16;

    if from == Stance::Calm && to != Stance::Calm {
        delta += 2;
    }

    if to == Stance::Divinity {
        delta += 3;
    }

    delta
}

pub fn combat_ended(monsters: &[MonsterSnapshot]) -> bool {
    if monsters.is_empty() {
        return false;
    }
    monsters.iter().all(|m| m.hp <= 0 || m.is_minion)
}
