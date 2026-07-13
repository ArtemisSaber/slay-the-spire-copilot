use super::super::*;

#[test]
fn block_card_does_not_parse_as_str_gain() {
    let e = parse_description("获得 5 点 格挡");
    assert_eq!(e.str_gain, None);
}

#[test]
fn block_card_does_not_parse_as_dex_gain() {
    let e = parse_description("获得 5 点 格挡");
    assert_eq!(e.dex_gain, None);
}

#[test]
fn block_card_does_not_parse_as_self_damage() {
    let e = parse_description("获得 5 点 格挡");
    assert_eq!(e.self_damage, None);
}

#[test]
fn block_card_does_not_parse_as_heal() {
    let e = parse_description("获得 5 点 格挡");
    assert_eq!(e.heal, None);
}

#[test]
fn block_card_does_not_parse_as_draw() {
    let e = parse_description("获得 5 点 格挡");
    assert_eq!(e.draw, None);
}

#[test]
fn block_card_correctly_parses_block() {
    let e = parse_description("获得 5 点 格挡");
    assert_eq!(e.block, Some(5));
}

#[test]
fn self_str_gain_does_not_parse_as_block() {
    let e = parse_description("获得 3 点 力量");
    assert_eq!(e.block, None);
    assert_eq!(e.str_gain, Some(3));
}

#[test]
fn self_dex_gain_does_not_parse_as_block() {
    let e = parse_description("获得 2 点 敏捷");
    assert_eq!(e.block, None);
    assert_eq!(e.dex_gain, Some(2));
}

#[test]
fn str_loss_does_not_parse_as_self_damage() {
    let e = parse_description("敌人失去 4 点力量");
    assert_eq!(e.self_damage, None);
}

#[test]
fn self_damage_does_not_parse_as_str_loss() {
    let e = parse_description("失去 6 点生命。获得 3 点力量");
    assert_eq!(e.str_loss, None);
    assert_eq!(e.self_damage, Some(6));
    assert_eq!(e.str_gain, Some(3));
}

#[test]
fn defend_zh_has_no_false_positives() {
    let e = parse_description("获得 5 点 格挡 。");
    assert_eq!(e.block, Some(5));
    assert_eq!(e.damage, None);
    assert_eq!(e.heal, None);
    assert_eq!(e.draw, None);
    assert_eq!(e.self_damage, None);
    assert_eq!(e.str_gain, None);
    assert_eq!(e.dex_gain, None);
    assert_eq!(e.poison, None);
    assert_eq!(e.vulnerable, None);
    assert_eq!(e.weak, None);
    assert_eq!(e.energy_gain, None);
    assert_eq!(e.str_loss, None);
    assert_eq!(e.mantra, None);
    assert_eq!(e.focus_gain, None);
    assert!(e.channel_orb.is_none());
    assert!(e.evoke_orb.is_none());
    assert_eq!(e.orb_slot_expand, 0);
    assert!(!e.exits_stance);
    assert!(!e.ethereal);
}

#[test]
fn defend_en_has_no_false_positives() {
    let e = parse_description("Gain 5 Block.");
    assert_eq!(e.block, Some(5));
    assert_eq!(e.damage, None);
    assert_eq!(e.heal, None);
    assert_eq!(e.draw, None);
    assert_eq!(e.self_damage, None);
    assert_eq!(e.str_gain, None);
    assert_eq!(e.dex_gain, None);
}
