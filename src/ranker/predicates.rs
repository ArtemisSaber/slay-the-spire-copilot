use std::collections::HashMap;

pub fn hp_cost_penalty(vars: &HashMap<String, f64>) -> i64 {
    let self_damage = vars.get("self_damage").copied().unwrap_or(0.0);
    if self_damage <= 0.0 {
        return 0;
    }
    let cur_hp = vars.get("current_hp").copied().unwrap_or(1.0).max(1.0);
    let penalty = -1_050_000.0 * self_damage / (cur_hp * cur_hp * cur_hp);
    penalty as i64
}

pub fn dispatch(name: &str, vars: &HashMap<String, f64>) -> Option<i64> {
    match name {
        "hp_cost_penalty" => Some(hp_cost_penalty(vars)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/predicates_tests.rs"]
mod tests;
