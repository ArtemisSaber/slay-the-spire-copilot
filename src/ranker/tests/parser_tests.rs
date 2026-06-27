use super::*;

// --- damage (existing) ---

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

// --- hits (existing + improvements) ---

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

// --- block (existing) ---

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

// --- self_damage (existing) ---

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

// --- exhaust (existing) ---

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

// --- heal ---

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

// --- draw ---

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

// --- str_gain (self) ---

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

// --- dex_gain ---

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

// --- poison ---

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

// --- vulnerable ---

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

// --- weak ---

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

// --- energy_gain ([E] count) ---

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

// --- str_loss (enemy) ---

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

// --- mantra ---

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

// --- focus_gain ---

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

// --- channel_orb ---

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

// --- evoke_orb ---

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

// --- orb_slot_expand ---

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

// --- stance exit ---

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

// --- enter wrath ---

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

// --- enter calm ---

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

// --- ethereal ---

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

// --- empty ---

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
