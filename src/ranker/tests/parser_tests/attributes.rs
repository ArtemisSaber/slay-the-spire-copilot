use super::super::*;

#[test]
fn parse_str_gain_zh() {
    let e = parse_description("获得 3 点 力量");
    assert_eq!(e.str_gain, Some(3));
}

#[test]
fn parse_str_gain_en() {
    let e = parse_description("Gain 3 Strength");
    assert_eq!(e.str_gain, Some(3));
}

#[test]
fn parse_no_str_gain_on_enemy_loss() {
    let e = parse_description("敌人失去 4 点力量");
    assert_eq!(e.str_gain, None);
    assert_eq!(e.str_loss, Some(4));
}

#[test]
fn parse_dex_gain_zh() {
    let e = parse_description("获得 2 点 敏捷");
    assert_eq!(e.dex_gain, Some(2));
}

#[test]
fn parse_dex_gain_en() {
    let e = parse_description("Gain 2 Dexterity");
    assert_eq!(e.dex_gain, Some(2));
}

#[test]
fn parse_poison_zh() {
    let e = parse_description("给予 5 层 中毒");
    assert_eq!(e.poison, Some(5));
}

#[test]
fn parse_poison_en() {
    let e = parse_description("Apply 5 Poison");
    assert_eq!(e.poison, Some(5));
}

#[test]
fn parse_vulnerable_zh() {
    let e = parse_description("给予 2 层 易伤");
    assert_eq!(e.vulnerable, Some(2));
}

#[test]
fn parse_vulnerable_en() {
    let e = parse_description("Apply 2 Vulnerable");
    assert_eq!(e.vulnerable, Some(2));
}

#[test]
fn parse_weak_zh() {
    let e = parse_description("给予 3 层 虚弱");
    assert_eq!(e.weak, Some(3));
}

#[test]
fn parse_weak_en() {
    let e = parse_description("Apply 3 Weak");
    assert_eq!(e.weak, Some(3));
}

#[test]
fn parse_energy_two() {
    let e = parse_description("[E] [E] 造成 8 点伤害");
    assert_eq!(e.energy_gain, Some(2));
}

#[test]
fn parse_energy_none() {
    let e = parse_description("造成 6 点伤害");
    assert_eq!(e.energy_gain, None);
}

#[test]
fn parse_energy_one() {
    let e = parse_description("获得 [E] 。造成 6 点伤害");
    assert_eq!(e.energy_gain, Some(1));
}

#[test]
fn parse_str_loss_zh() {
    let e = parse_description("敌人失去 4 点力量");
    assert_eq!(e.str_loss, Some(4));
    assert!(!e.str_loss_temp);
}

#[test]
fn parse_str_loss_en() {
    let e = parse_description("Enemy loses 4 Strength");
    assert_eq!(e.str_loss, Some(4));
    assert!(!e.str_loss_temp);
}

#[test]
fn parse_str_loss_temp_zh() {
    let e = parse_description("敌人失去 2 点力量。一回合");
    assert_eq!(e.str_loss, Some(2));
    assert!(e.str_loss_temp);
}

#[test]
fn parse_str_loss_temp_en() {
    let e = parse_description("Enemy loses 2 Strength for 1 turn");
    assert_eq!(e.str_loss, Some(2));
    assert!(e.str_loss_temp);
}

#[test]
fn parse_mantra_zh() {
    let e = parse_description("获得 3 层 真言");
    assert_eq!(e.mantra, Some(3));
}

#[test]
fn parse_mantra_en() {
    let e = parse_description("Gain 3 Mantra");
    assert_eq!(e.mantra, Some(3));
}

#[test]
fn parse_focus_gain_zh() {
    let e = parse_description("获得 2 点 集中");
    assert_eq!(e.focus_gain, Some(2));
}

#[test]
fn parse_focus_gain_en() {
    let e = parse_description("Gain 2 Focus");
    assert_eq!(e.focus_gain, Some(2));
}
