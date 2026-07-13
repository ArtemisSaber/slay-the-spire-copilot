use super::*;

#[test]
fn block_exact_match_zero_overflow() {
    // Test via a targeted hit where block == damage
    let mut monsters = vec![ms(20, 5, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 20);
    assert_eq!(monsters[0].block, 0);
}

#[test]
fn block_overflow_hp_reduction() {
    let mut monsters = vec![ms(20, 3, vec![])];
    let dmg = DamageEffect {
        amount: 10,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 13);
    assert_eq!(monsters[0].block, 0);
}

#[test]
fn block_completely_absorbs_damage() {
    let mut monsters = vec![ms(20, 20, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 20);
    assert_eq!(monsters[0].block, 15);
}

#[test]
fn invincible_caps_damage_per_hit() {
    let mut monsters = vec![ms(50, 0, vec![("Invincible", 5)])];
    let dmg = DamageEffect {
        amount: 20,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 45);
}

#[test]
fn invincible_amount_decreases_after_capping() {
    let mut monsters = vec![ms(50, 0, vec![("Invincible", 8)])];
    let dmg = DamageEffect {
        amount: 15,
        hits: HitCount::Fixed(2),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    let inv = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Invincible")
        .unwrap();
    assert!(inv.amount < 8);
}

#[test]
fn invincible_does_not_affect_zero_damage_hits() {
    let mut monsters = vec![ms(50, 0, vec![("Invincible", 5)])];
    let dmg = DamageEffect {
        amount: 1,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    monsters[0].block = 5;
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    let inv = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Invincible")
        .unwrap();
    assert_eq!(inv.amount, 5);
}
