use crate::combat::effects::{DamageEffect, HitCount, TargetType};
use crate::combat::{MonsterSnapshot, Stance};

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
            let Some(ti) = target_idx else {
                tracing::warn!("targeted damage called without target_idx, skipping");
                return;
            };
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

pub(crate) fn calc_effective_damage(
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
