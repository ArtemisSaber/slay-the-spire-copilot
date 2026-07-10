use super::*;

#[test]
fn danger_detects_low_hp() {
    let d = compute_danger(10, 80, 0, 0, false, &[]);
    assert!(d.hp_critical);
    assert!(!d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn danger_detects_incoming_lethal() {
    let d = compute_danger(20, 80, 5, 30, false, &[]);
    assert!(d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn danger_no_block_against_hit_is_caution() {
    let d = compute_danger(60, 80, 0, 10, false, &["ATTACK"]);
    assert!(d.no_block_against_hit);
    assert!(!d.hp_critical);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_monster_attacking_is_caution() {
    let d = compute_danger(60, 80, 10, 0, false, &["ATTACK"]);
    assert!(d.any_monster_attacking);
    assert!(!d.hp_critical);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_low_hp_is_caution() {
    let d = compute_danger(35, 80, 10, 0, false, &[]);
    assert!(!d.hp_critical);
    assert!(!d.any_monster_attacking);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_wrath_stance_detected() {
    let d = compute_danger(60, 80, 10, 0, true, &["ATTACK"]);
    assert!(d.wrath_stance);
    assert!(d.any_monster_attacking);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_safe_when_healthy_and_not_threatened() {
    let d = compute_danger(70, 80, 20, 0, false, &[]);
    assert!(!d.hp_critical);
    assert!(!d.incoming_lethal);
    assert!(!d.no_block_against_hit);
    assert!(!d.any_monster_attacking);
    assert_eq!(d.level, DangerLevel::Safe);
}

#[test]
fn danger_none_hp_uses_fallback() {
    let d = DangerFlags::compute(None, None, None, 0, &[], &[]);
    assert!(d.hp_critical);
    assert!(!d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn danger_max_hp_zero_fallback() {
    let d = DangerFlags::compute(Some(50), Some(0), Some(0), 0, &[], &[]);
    assert!(!d.hp_critical);
    assert!(!d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Safe);
}

#[test]
fn danger_current_hp_zero_with_max_hp() {
    let d = DangerFlags::compute(Some(0), Some(80), Some(0), 0, &[], &[]);
    assert!(d.hp_critical);
    assert!(!d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn danger_block_absorbs_incoming_lethal() {
    let d = compute_danger(20, 80, 15, 30, false, &["ATTACK"]);
    assert!(!d.incoming_lethal);
    assert!(d.any_monster_attacking);
}

#[test]
fn danger_incoming_lethal_when_damage_exceeds_hp_plus_block() {
    let d = compute_danger(20, 80, 5, 30, false, &[]);
    assert!(d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn danger_wrath_stance_increases_risk() {
    let d = compute_danger(70, 80, 20, 0, true, &[]);
    assert!(d.wrath_stance);
}

#[test]
fn danger_no_wrath_without_stance_power() {
    let d = compute_danger(70, 80, 20, 0, false, &["ATTACK"]);
    assert!(!d.wrath_stance);
    assert_eq!(d.level, DangerLevel::Caution);
}
