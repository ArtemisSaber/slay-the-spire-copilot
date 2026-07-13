use super::*;

#[test]
fn apply_damage_random_target_first_hit_kills_next_hits_next() {
    let mut monsters = vec![
        // Note: random target always picks living[0] (first living)
        ms(10, 0, vec![]),
        ms(10, 0, vec![]),
    ];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(4),
        target_type: TargetType::RandomTarget,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
    assert_eq!(monsters[1].hp, 0);
}

#[test]
fn apply_damage_aoe_multi_hit_hits_all_living() {
    let mut monsters = vec![ms(20, 0, vec![]), ms(20, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(2),
        target_type: TargetType::AoE,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 8);
    assert_eq!(monsters[1].hp, 8);
}

#[test]
fn apply_damage_aoe_mid_hit_death_excludes_from_later_hits() {
    let mut monsters = vec![ms(5, 0, vec![]), ms(20, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(2),
        target_type: TargetType::AoE,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
    assert_eq!(monsters[1].hp, 8);
}

#[test]
fn slow_and_vulnerable_stacking() {
    let monster = ms(20, 0, vec![("Slow", 3), ("Vulnerable", 1)]);
    let effective = super::super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert!(effective > 15);
}

#[test]
fn flight_and_vulnerable_cancel_partially() {
    let monster = ms(20, 0, vec![("Flight", 3), ("Vulnerable", 1)]);
    let effective = super::super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert_eq!(effective, 7);
}
