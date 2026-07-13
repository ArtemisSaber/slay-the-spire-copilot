use super::*;

#[test]
fn artifact_blocks_vulnerable_and_decrements() {
    let mut monster = ms(20, 0, vec![("Artifact", 2)]);
    apply_vulnerable_through_artifact(&mut monster, 3);
    let a = monster.powers.iter().find(|p| p.id == "Artifact").unwrap();
    assert_eq!(a.amount, 1);
    let has_vuln = monster.powers.iter().any(|p| p.id == "Vulnerable");
    assert!(!has_vuln);
}

#[test]
fn artifact_floor_at_zero_when_amount_would_go_negative() {
    let mut monster = ms(20, 0, vec![("Artifact", 0)]);
    apply_vulnerable_through_artifact(&mut monster, 3);
    let a = monster.powers.iter().find(|p| p.id == "Artifact").unwrap();
    assert_eq!(a.amount, 0);
}

#[test]
fn vulnerable_stacking_on_existing() {
    let mut monster = ms(20, 0, vec![("Vulnerable", 2)]);
    apply_vulnerable_through_artifact(&mut monster, 3);
    let v = monster
        .powers
        .iter()
        .find(|p| p.id == "Vulnerable")
        .unwrap();
    assert_eq!(v.amount, 5);
}

#[test]
fn vulnerable_creates_new_stack_when_not_present() {
    let mut monster = ms(20, 0, vec![]);
    apply_vulnerable_through_artifact(&mut monster, 3);
    let v = monster
        .powers
        .iter()
        .find(|p| p.id == "Vulnerable")
        .unwrap();
    assert_eq!(v.amount, 3);
}

#[test]
fn vulnerable_chinese_id_stacking() {
    let mut monster = ms(20, 0, vec![("易伤", 2)]);
    apply_vulnerable_through_artifact(&mut monster, 1);
    let v = monster.powers.iter().find(|p| p.id == "易伤").unwrap();
    assert_eq!(v.amount, 3);
}

#[test]
fn slow_scales_up_after_card() {
    let mut monsters = vec![ms(20, 0, vec![("Slow", 3)])];
    apply_after_card_powers(&mut monsters);
    let s = monsters[0].powers.iter().find(|p| p.id == "Slow").unwrap();
    assert_eq!(s.amount, 4);
}

#[test]
fn slow_chinese_id_scales_up() {
    let mut monsters = vec![ms(20, 0, vec![("缓慢", 1)])];
    apply_after_card_powers(&mut monsters);
    let s = monsters[0].powers.iter().find(|p| p.id == "缓慢").unwrap();
    assert_eq!(s.amount, 2);
}

#[test]
fn slow_zero_amount_not_incremented() {
    let mut monsters = vec![ms(20, 0, vec![("Slow", 0)])];
    apply_after_card_powers(&mut monsters);
    let s = monsters[0].powers.iter().find(|p| p.id == "Slow").unwrap();
    assert_eq!(s.amount, 0);
}
