use super::super::*;

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
fn parse_hits_twice_zh() {
    let e = parse_description("造成 9 点伤害两次");
    assert_eq!(e.damage, Some(9));
    assert_eq!(e.hits, 2);
}

#[test]
fn parse_hits_twice_en() {
    let e = parse_description("Deal 5 damage twice");
    assert_eq!(e.damage, Some(5));
    assert_eq!(e.hits, 2);
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

#[test]
fn parse_exhaust_count_zh() {
    let e = parse_description("消耗 1 张牌。抽 3 张牌");
    assert_eq!(e.exhaust_count, 1);
    assert_eq!(e.draw, Some(3));
}

#[test]
fn parse_exhaust_count_en() {
    let e = parse_description("Exhaust 2 cards. Draw 3 cards");
    assert_eq!(e.exhaust_count, 2);
    assert_eq!(e.draw, Some(3));
}

#[test]
fn parse_heal_zh() {
    let e = parse_description("回复 5 点生命");
    assert_eq!(e.heal, Some(5));
}

#[test]
fn parse_heal_en() {
    let e = parse_description("Heal 5 HP");
    assert_eq!(e.heal, Some(5));
}

#[test]
fn parse_no_heal_on_damage() {
    let e = parse_description("造成 6 点伤害");
    assert_eq!(e.heal, None);
}

#[test]
fn parse_no_heal_on_self_damage() {
    let e = parse_description("失去 3 点生命");
    assert_eq!(e.heal, None);
}

#[test]
fn parse_draw_zh() {
    let e = parse_description("抽 2 张牌");
    assert_eq!(e.draw, Some(2));
}

#[test]
fn parse_draw_zh_variant() {
    let e = parse_description("抽 3 张牌。造成 6 点伤害");
    assert_eq!(e.draw, Some(3));
    assert_eq!(e.damage, Some(6));
}

#[test]
fn parse_draw_en() {
    let e = parse_description("Draw 2 cards");
    assert_eq!(e.draw, Some(2));
}

#[test]
fn parse_draw_en_singular() {
    let e = parse_description("Draw 1 card");
    assert_eq!(e.draw, Some(1));
}
