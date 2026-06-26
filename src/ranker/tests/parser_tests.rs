use super::*;

#[test]
fn parse_damage_zh() {
    let desc = "造成 6 点伤害";
    let e = parse_description(desc);
    assert_eq!(e.damage, Some(6));
    assert_eq!(e.hits, 1);
}

#[test]
fn parse_damage_en() {
    let desc = "Deal 6 damage";
    let e = parse_description(desc);
    assert_eq!(e.damage, Some(6));
    assert_eq!(e.hits, 1);
}

#[test]
fn parse_multihit_zh() {
    let desc = "造成 6 点伤害 2 次";
    let e = parse_description(desc);
    assert_eq!(e.damage, Some(6));
    assert_eq!(e.hits, 2);
}

#[test]
fn parse_multihit_en() {
    let desc = "Deal 8 damage 3 times";
    let e = parse_description(desc);
    assert_eq!(e.damage, Some(8));
    assert_eq!(e.hits, 3);
}

#[test]
fn parse_block_zh() {
    let desc = "获得 5 点 格挡";
    let e = parse_description(desc);
    assert_eq!(e.block, Some(5));
}

#[test]
fn parse_block_en() {
    let desc = "Gain 5 Block";
    let e = parse_description(desc);
    assert_eq!(e.block, Some(5));
}

#[test]
fn parse_self_damage_zh() {
    let desc = "失去 3 点生命";
    let e = parse_description(desc);
    assert_eq!(e.self_damage, Some(3));
}

#[test]
fn parse_self_damage_en() {
    let desc = "Lose 3 HP";
    let e = parse_description(desc);
    assert_eq!(e.self_damage, Some(3));
}

#[test]
fn parse_empty_description() {
    let e = parse_description("");
    assert_eq!(e.damage, None);
    assert_eq!(e.hits, 1);
    assert_eq!(e.block, None);
    assert_eq!(e.self_damage, None);
}

#[test]
fn parse_exhaust_en() {
    let desc = "Exhaust a card. Deal 9 damage";
    let e = parse_description(desc);
    assert_eq!(e.damage, Some(9));
    assert_eq!(e.exhaust_count, 1);
}

#[test]
fn parse_exhaust_all_zh() {
    let desc = "消耗 所有 手牌。造成 10 点伤害";
    let e = parse_description(desc);
    assert_eq!(e.damage, Some(10));
    assert_eq!(e.exhaust_count, 1);
}
