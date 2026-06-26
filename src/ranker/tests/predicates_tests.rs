use super::*;
use std::collections::HashMap;

fn vars(hp: f64, self_damage: f64) -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("current_hp".to_string(), hp);
    m.insert("self_damage".to_string(), self_damage);
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
fn dispatch_unknown_fn_returns_none() {
    let v = HashMap::new();
    assert_eq!(dispatch("nonexistent", &v), None);
}
