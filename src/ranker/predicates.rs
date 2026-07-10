use std::collections::HashMap;

pub fn hp_cost_penalty(vars: &HashMap<String, f64>) -> i64 {
    let self_damage = vars.get("self_damage").copied().unwrap_or(0.0);
    health_scaled_damage_penalty(self_damage, vars.get("current_hp").copied().unwrap_or(1.0))
}

/// Penalizes damage from enemy retaliation after the player's existing and
/// card-provided Block absorb it. Unlike an HP cost, retaliation is damage and
/// therefore must be reduced by Block before it is scored.
pub fn retaliation_damage_penalty(vars: &HashMap<String, f64>) -> i64 {
    let retaliation = vars.get("retaliatory_damage").copied().unwrap_or(0.0);
    let block_before_attack = vars.get("current_block").copied().unwrap_or(0.0);
    let card_block = vars.get("block").copied().unwrap_or(0.0);
    let unblocked_damage = (retaliation - block_before_attack - card_block).max(0.0);
    health_scaled_damage_penalty(
        unblocked_damage,
        vars.get("current_hp").copied().unwrap_or(1.0),
    )
}

fn health_scaled_damage_penalty(damage: f64, current_hp: f64) -> i64 {
    if damage <= 0.0 {
        return 0;
    }
    let hp = current_hp.max(1.0);
    let penalty = -1_050_000.0 * damage / (hp * hp * hp);
    penalty as i64
}

pub fn weak_new_formula(vars: &HashMap<String, f64>) -> i64 {
    let weak = vars.get("weak").copied().unwrap_or(0.0);
    if weak <= 0.0 {
        return 0;
    }
    let target_damage = vars.get("target_damage").copied().unwrap_or(0.0);
    let target_hits = vars.get("target_hits").copied().unwrap_or(0.0);
    let target_total_damage = target_damage * target_hits;
    let score = (5.0 + target_total_damage * 2.5) * weak;
    score as i64
}

pub fn weak_refresh_formula(vars: &HashMap<String, f64>) -> i64 {
    let weak = vars.get("weak").copied().unwrap_or(0.0);
    if weak <= 0.0 {
        return 0;
    }
    let score = 5.0 * weak;
    score as i64
}

pub fn dispatch(name: &str, vars: &HashMap<String, f64>) -> Option<i64> {
    match name {
        "hp_cost_penalty" => Some(hp_cost_penalty(vars)),
        "retaliation_damage_penalty" => Some(retaliation_damage_penalty(vars)),
        "weak_new_formula" => Some(weak_new_formula(vars)),
        "weak_refresh_formula" => Some(weak_refresh_formula(vars)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/predicates_tests.rs"]
mod tests;
