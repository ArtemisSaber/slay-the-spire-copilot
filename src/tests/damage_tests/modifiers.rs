use super::*;

#[test]
fn intangible_caps_damage_at_1_even_with_wrath_and_strength() {
    let monster = ms(100, 0, vec![("Intangible", 1)]);
    let effective = super::super::calc_effective_damage(10, &monster, Stance::Wrath, 5);
    assert_eq!(effective, 1);
}

#[test]
fn intangible_caps_at_1_even_with_vulnerable() {
    let monster = ms(100, 0, vec![("Intangible", 1), ("Vulnerable", 1)]);
    let effective = super::super::calc_effective_damage(6, &monster, Stance::Neutral, 0);
    assert_eq!(effective, 1);
}

#[test]
fn slow_increases_damage() {
    let monster = ms(20, 0, vec![("Slow", 5)]);
    let effective = super::super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert!(effective > 10);
}

#[test]
fn slow_chinese_id_applies() {
    let monster = ms(20, 0, vec![("缓慢", 5), ("Slow", 1)]);
    let effective = super::super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert!(effective > 10);
}

#[test]
fn vulnerable_multiplies_damage() {
    let monster = ms(20, 0, vec![("Vulnerable", 1)]);
    let effective = super::super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert_eq!(effective, 15);
}

#[test]
fn vulnerable_chinese_id_multiplies_damage() {
    let monster = ms(20, 0, vec![("易伤", 1)]);
    let effective = super::super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert_eq!(effective, 15);
}

#[test]
fn flight_halves_damage() {
    let monster = ms(20, 0, vec![("Flight", 3)]);
    let effective = super::super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert_eq!(effective, 5);
}

#[test]
fn strength_adds_before_multipliers() {
    let monster = ms(20, 0, vec![("Vulnerable", 1)]);
    let effective = super::super::calc_effective_damage(10, &monster, Stance::Neutral, 4);
    assert_eq!(effective, 21);
}

#[test]
fn wrath_doubles_base_damage() {
    let monster = ms(20, 0, vec![]);
    let effective = super::super::calc_effective_damage(6, &monster, Stance::Wrath, 0);
    assert_eq!(effective, 12);
}

#[test]
fn divinity_triples_base_damage() {
    let monster = ms(20, 0, vec![]);
    let effective = super::super::calc_effective_damage(6, &monster, Stance::Divinity, 0);
    assert_eq!(effective, 18);
}

#[test]
fn damage_floor_at_zero() {
    let monster = ms(20, 0, vec![]);
    let effective = super::super::calc_effective_damage(1, &monster, Stance::Neutral, -10);
    assert_eq!(effective, 0);
}
