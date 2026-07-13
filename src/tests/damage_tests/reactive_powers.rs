use super::*;

#[test]
fn curl_up_triggers_and_adds_block_after_hits() {
    let mut monsters = vec![ms_triggered(20, 0, vec![("Curl Up", 10, false)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(3),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 5);
    assert_eq!(monsters[0].block, 10);
}

#[test]
fn curl_up_applies_block_after_hit_then_zeroes() {
    let mut monsters = vec![ms_triggered(20, 0, vec![("Curl Up", 5, false)])];
    let dmg = DamageEffect {
        amount: 8,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 12);
    assert_eq!(monsters[0].block, 5);
}

#[test]
fn curl_up_already_triggered_still_adds_block_once() {
    let mut monsters = vec![ms_triggered(20, 0, vec![("Curl Up", 10, true)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(3),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 5);
    assert_eq!(monsters[0].block, 10);
}

#[test]
fn curl_up_with_zero_amount_does_not_trigger() {
    let mut monsters = vec![ms_triggered(20, 0, vec![("Curl Up", 0, false)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 15);
    assert_eq!(monsters[0].block, 0);
}

#[test]
fn malleable_adds_block_on_unblocked_damage() {
    let mut monsters = vec![ms(30, 0, vec![("Malleable", 3)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].block, 3);
    let m = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Malleable")
        .unwrap();
    assert_eq!(m.amount, 4);
}

#[test]
fn malleable_does_not_trigger_when_all_blocked() {
    let mut monsters = vec![ms(30, 10, vec![("Malleable", 3)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].block, 5);
    let m = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Malleable")
        .unwrap();
    assert_eq!(m.amount, 3);
}

#[test]
fn malleable_with_zero_amount_not_found() {
    let mut monsters = vec![ms(30, 0, vec![("Malleable", 0)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].block, 0);
}

#[test]
fn flight_decrements_on_hit_whether_unblocked_or_not() {
    let mut monsters = vec![ms(30, 10, vec![("Flight", 4)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    let f = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Flight")
        .unwrap();
    assert_eq!(f.amount, 3);
}

#[test]
fn flight_floor_at_zero_does_not_go_negative() {
    let mut monsters = vec![ms(30, 0, vec![("Flight", 0)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    let f = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Flight")
        .unwrap();
    assert_eq!(f.amount, 0);
}
