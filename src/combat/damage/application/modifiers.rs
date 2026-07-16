use crate::combat::{MonsterSnapshot, Stance};

use super::has_power;

#[cfg(test)]
pub(crate) fn calc_effective_damage(
    base: i16,
    monster: &MonsterSnapshot,
    stance: Stance,
    strength_delta: i16,
) -> i16 {
    calc_effective_damage_from_rendered(base, monster, Stance::Neutral, stance, strength_delta)
}

pub(super) fn calc_effective_damage_from_rendered(
    base: i16,
    monster: &MonsterSnapshot,
    initial_stance: Stance,
    current_stance: Stance,
    strength_delta: i16,
) -> i16 {
    let stance_mult = stance_damage_multiplier(current_stance);
    let initial_mult = stance_damage_multiplier(initial_stance);
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
            .find(|power| power.id == "Slow" || power.id == "缓慢")
            .map(|power| power.amount)
            .unwrap_or(0) as f64;
        damage = (damage * (1.0 + 0.1 * slow_amount)).floor();
    }

    if has_power(monster, "Vulnerable") || monster.powers.iter().any(|power| power.id == "易伤") {
        damage = (damage * 1.5).floor();
    }

    if has_power(monster, "Flight") {
        damage = (damage * 0.5).floor();
    }

    damage as i16
}

fn stance_damage_multiplier(stance: Stance) -> i16 {
    match stance {
        Stance::Neutral | Stance::Calm => 1,
        Stance::Wrath => 2,
        Stance::Divinity => 3,
    }
}
