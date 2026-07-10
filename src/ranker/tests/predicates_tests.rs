use super::*;
use std::collections::HashMap;

fn vars(hp: f64, self_damage: f64) -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("current_hp".to_string(), hp);
    m.insert("self_damage".to_string(), self_damage);
    m
}

fn weak_vars(weak_amount: f64, target_damage: f64, target_hits: f64) -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("weak".to_string(), weak_amount);
    m.insert("target_damage".to_string(), target_damage);
    m.insert("target_hits".to_string(), target_hits);
    m
}

#[test]
fn hp_cost_at_high_hp_is_negligible() {
    let v = vars(75.0, 6.0);
    let score = hp_cost_penalty(&v);
    assert!(score < 0);
    assert!(score > -200);
}

#[test]
fn hp_cost_at_45_hp_offering_crossover() {
    let v = vars(45.0, 6.0);
    let score = hp_cost_penalty(&v);
    assert!(score <= -69);
    assert!(score >= -71);
}

#[test]
fn hp_cost_at_low_hp_is_devastating() {
    let v = vars(2.0, 6.0);
    let score = hp_cost_penalty(&v);
    assert!(score < -500_000);
}

#[test]
fn hp_cost_zero_damage_returns_zero() {
    let v = vars(10.0, 0.0);
    let score = hp_cost_penalty(&v);
    assert_eq!(score, 0);
}

#[test]
fn dispatch_known_fn() {
    let v = vars(75.0, 6.0);
    let score = dispatch("hp_cost_penalty", &v).unwrap();
    assert!(score < 0);
}

#[test]
fn retaliation_damage_is_absorbed_by_existing_and_card_block() {
    let mut v = HashMap::new();
    v.insert("current_hp".to_string(), 3.0);
    v.insert("current_block".to_string(), 2.0);
    v.insert("block".to_string(), 1.0);
    v.insert("retaliatory_damage".to_string(), 3.0);

    assert_eq!(retaliation_damage_penalty(&v), 0);
}

#[test]
fn lethal_retaliation_is_scored_as_a_severe_danger() {
    let mut v = HashMap::new();
    v.insert("current_hp".to_string(), 3.0);
    v.insert("current_block".to_string(), 0.0);
    v.insert("retaliatory_damage".to_string(), 3.0);

    assert!(retaliation_damage_penalty(&v) < -100_000);
}

#[test]
fn dispatch_unknown_fn_returns_none() {
    let v = HashMap::new();
    assert_eq!(dispatch("nonexistent", &v), None);
}

// --- weak_new_formula ---

#[test]
fn weak_new_attacking_target() {
    let v = weak_vars(2.0, 12.0, 1.0);
    let score = weak_new_formula(&v);
    // (5 + 12*1*2.5) * 2 = (5 + 30) * 2 = 70
    assert_eq!(score, 70);
}

#[test]
fn weak_new_multihit_target() {
    let v = weak_vars(1.0, 7.0, 3.0);
    let score = weak_new_formula(&v);
    // (5 + 7*3*2.5) * 1 = (5 + 52.5) * 1 = 57
    assert_eq!(score, 57);
}

#[test]
fn weak_new_zero_damage_target() {
    let v = weak_vars(3.0, 0.0, 0.0);
    let score = weak_new_formula(&v);
    // (5 + 0*0*2.5) * 3 = 5 * 3 = 15
    assert_eq!(score, 15);
}

#[test]
fn weak_new_zero_weak_returns_zero() {
    let v = weak_vars(0.0, 12.0, 1.0);
    let score = weak_new_formula(&v);
    assert_eq!(score, 0);
}

// --- weak_refresh_formula ---

#[test]
fn weak_refresh_with_existing() {
    let v = weak_vars(2.0, 12.0, 1.0);
    let score = weak_refresh_formula(&v);
    // 5 * 2 = 10
    assert_eq!(score, 10);
}

#[test]
fn weak_refresh_zero_weak_returns_zero() {
    let v = weak_vars(0.0, 12.0, 1.0);
    let score = weak_refresh_formula(&v);
    assert_eq!(score, 0);
}

// --- dispatch for new functions ---

#[test]
fn dispatch_weak_new() {
    let v = weak_vars(2.0, 12.0, 1.0);
    let score = dispatch("weak_new_formula", &v).unwrap();
    assert_eq!(score, 70);
}

#[test]
fn dispatch_weak_refresh() {
    let v = weak_vars(2.0, 12.0, 1.0);
    let score = dispatch("weak_refresh_formula", &v).unwrap();
    assert_eq!(score, 10);
}
