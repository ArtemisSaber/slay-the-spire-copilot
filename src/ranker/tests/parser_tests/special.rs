use super::super::*;

#[test]
fn parse_channel_lightning_zh() {
    let e = parse_description("充能 1 个 闪电 球");
    assert_eq!(e.channel_orb.as_deref(), Some("Lightning"));
}

#[test]
fn parse_channel_frost_en() {
    let e = parse_description("Channel 1 Frost");
    assert_eq!(e.channel_orb.as_deref(), Some("Frost"));
}

#[test]
fn parse_channel_dark_zh() {
    let e = parse_description("充能 一个 黑暗 球");
    assert_eq!(e.channel_orb.as_deref(), Some("Dark"));
}

#[test]
fn parse_channel_plasma_en() {
    let e = parse_description("Channel 1 Plasma");
    assert_eq!(e.channel_orb.as_deref(), Some("Plasma"));
}

#[test]
fn parse_no_channel_on_damage() {
    let e = parse_description("造成 6 点伤害");
    assert_eq!(e.channel_orb, None);
}

#[test]
fn parse_evoke_zh() {
    let e = parse_description("激发 你的 所有 闪电 球");
    assert_eq!(e.evoke_orb.as_deref(), Some("Lightning"));
}

#[test]
fn parse_evoke_en() {
    let e = parse_description("Evoke your next Frost Orb");
    assert_eq!(e.evoke_orb.as_deref(), Some("Frost"));
}

#[test]
fn parse_orb_slot_expand_zh() {
    let e = parse_description("获得 1 个 充能球栏位");
    assert_eq!(e.orb_slot_expand, 1);
}

#[test]
fn parse_orb_slot_expand_en() {
    let e = parse_description("Gain 2 Orb slots");
    assert_eq!(e.orb_slot_expand, 2);
}

#[test]
fn parse_no_orb_slot_expand() {
    let e = parse_description("造成 6 点伤害");
    assert_eq!(e.orb_slot_expand, 0);
}

#[test]
fn parse_exit_stance_zh() {
    let e = parse_description("退出 当前 姿态");
    assert!(e.exits_stance);
}

#[test]
fn parse_exit_stance_zh_end() {
    let e = parse_description("结束 你的 姿态");
    assert!(e.exits_stance);
}

#[test]
fn parse_exit_stance_en() {
    let e = parse_description("Exit your Stance");
    assert!(e.exits_stance);
}

#[test]
fn parse_not_exit_stance() {
    let e = parse_description("造成 6 点伤害");
    assert!(!e.exits_stance);
}

#[test]
fn parse_enter_wrath_zh() {
    let e = parse_description("进入 愤怒 姿态");
    assert!(e.enters_wrath);
    assert!(!e.enters_calm);
}

#[test]
fn parse_enter_wrath_en() {
    let e = parse_description("Enter Wrath");
    assert!(e.enters_wrath);
}

#[test]
fn parse_enter_calm_zh() {
    let e = parse_description("进入 宁静");
    assert!(e.enters_calm);
    assert!(!e.enters_wrath);
}

#[test]
fn parse_enter_calm_en() {
    let e = parse_description("Enter Calm");
    assert!(e.enters_calm);
}

#[test]
fn parse_ethereal_zh() {
    let e = parse_description("造成 6 点伤害。 虚无");
    assert!(e.ethereal);
}

#[test]
fn parse_ethereal_zh_inline() {
    let e = parse_description("虚无 。获得 5 点 格挡");
    assert!(e.ethereal);
    assert_eq!(e.block, Some(5));
}

#[test]
fn parse_ethereal_en() {
    let e = parse_description("Ethereal. Deal 6 damage");
    assert!(e.ethereal);
}

#[test]
fn parse_not_ethereal() {
    let e = parse_description("造成 6 点伤害");
    assert!(!e.ethereal);
}

#[test]
fn parse_empty_description() {
    let e = parse_description("");
    assert_eq!(e.damage, None);
    assert_eq!(e.hits, 1);
    assert_eq!(e.block, None);
    assert_eq!(e.self_damage, None);
    assert_eq!(e.heal, None);
    assert_eq!(e.draw, None);
    assert_eq!(e.str_gain, None);
    assert_eq!(e.dex_gain, None);
    assert_eq!(e.poison, None);
    assert_eq!(e.vulnerable, None);
    assert_eq!(e.weak, None);
    assert_eq!(e.energy_gain, None);
    assert_eq!(e.str_loss, None);
    assert!(!e.str_loss_temp);
    assert_eq!(e.mantra, None);
    assert_eq!(e.focus_gain, None);
    assert_eq!(e.channel_orb, None);
    assert_eq!(e.evoke_orb, None);
    assert_eq!(e.orb_slot_expand, 0);
    assert!(!e.exits_stance);
    assert!(!e.enters_wrath);
    assert!(!e.enters_calm);
    assert!(!e.ethereal);
}
