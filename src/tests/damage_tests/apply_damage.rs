use super::*;

#[test]
fn apply_damage_fixed_hit_with_x_value() {
    let mut monsters = vec![ms(20, 0, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), Some(3), &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 5);
}

#[test]
fn apply_damage_x_times_with_outstanding_x_value() {
    let mut monsters = vec![ms(60, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::XTimes,
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), Some(3), &mut monsters, Stance::Neutral, 0);
    assert_eq!(
        monsters[0].hp, 42,
        "6 damage X times with X=3 must deal 18, not 54"
    );
}

#[test]
fn apply_damage_x_plus_with_offset() {
    let mut monsters = vec![ms(30, 0, vec![])];
    let dmg = DamageEffect {
        amount: 7,
        hits: HitCount::XPlus(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), Some(2), &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 9);
}

#[test]
fn apply_damage_x_times_default_zero_when_no_x_value() {
    let mut monsters = vec![ms(20, 0, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::XTimes,
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 20);
}

#[test]
fn apply_damage_x_plus_default_zero_when_no_x_value() {
    let mut monsters = vec![ms(20, 0, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::XPlus(2),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 10);
}

#[test]
fn apply_damage_aoe_skips_dead_monsters() {
    let mut monsters = vec![ms(0, 0, vec![]), ms(10, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(1),
        target_type: TargetType::AoE,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
    assert_eq!(monsters[1].hp, 4);
}

#[test]
fn apply_damage_targeted_stops_when_target_dies_mid_hit() {
    let mut monsters = vec![ms(5, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(3),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
}

#[test]
fn apply_damage_targeted_without_target_does_not_panic() {
    let mut monsters = vec![ms(5, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(3),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(
        monsters[0].hp, 5,
        "monster HP should be unchanged when no target is provided"
    );
}

#[test]
fn apply_damage_random_target_stops_when_all_dead() {
    let mut monsters = vec![ms(6, 0, vec![])];
    let dmg = DamageEffect {
        amount: 7,
        hits: HitCount::Fixed(1),
        target_type: TargetType::RandomTarget,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
}

#[test]
fn apply_damage_random_target_no_living_breaks() {
    let mut monsters = vec![ms(0, 0, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(5),
        target_type: TargetType::RandomTarget,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
}
