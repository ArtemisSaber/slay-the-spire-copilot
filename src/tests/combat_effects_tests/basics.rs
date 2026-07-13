use super::*;

#[test]
fn parse_strike_damage() {
    let c = card("造成 6 点伤害。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(
        e.damage,
        Some(DamageEffect {
            amount: 6,
            hits: HitCount::Fixed(1),
            target_type: TargetType::Targeted,
        })
    );
}

#[test]
fn parse_multi_hit_damage() {
    let c = card("造成 2 点伤害 5 次。 消耗 。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(
        e.damage,
        Some(DamageEffect {
            amount: 2,
            hits: HitCount::Fixed(5),
            target_type: TargetType::Targeted,
        })
    );
}

#[test]
fn parse_two_hit_damage() {
    let c = card("造成 7 点伤害两次。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.damage.unwrap().hits, HitCount::Fixed(2));
}

#[test]
fn parse_aoe_damage() {
    let c = card("对所有敌人造成 4 点伤害。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.damage.unwrap().target_type, TargetType::AoE);
}

#[test]
fn parse_random_target() {
    let c = card("随机对敌人造成 3 点伤害 4 次。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.damage.unwrap().target_type, TargetType::RandomTarget);
}

#[test]
fn parse_x_cost_damage() {
    let c = card("对所有敌人造成 8 点伤害X次。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert!(e.x_cost);
    assert_eq!(e.damage.unwrap().target_type, TargetType::AoE);
}

#[test]
fn parse_vulnerable() {
    let c = card("造成 10 点伤害。 给予 3 层 易伤 。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.vulnerable, Some(3));
}

#[test]
fn parse_energy_gain() {
    let c = skill("获得 [E] [E] [E] 。 失去 3 点生命。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.energy_gain, 3);
}

#[test]
fn parse_strength_gain() {
    let c = skill("获得 4 点 力量 。 你的回合结束时，失去 4 点 力量 。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.strength_gain, 4);
}

#[test]
fn parse_exhaust_self() {
    let c = skill("获得 [E] [E] 。 消耗 。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.exhaust, ExhaustKind::Self_);
}

#[test]
fn parse_exhaust_all() {
    let c = skill("消耗 所有手牌。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.exhaust, ExhaustKind::All);
}

#[test]
fn parse_execute() {
    let c = skill("如果目标生命值小于等于 30 ，将其生命值变为0。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.execute, Some(30));
}

#[test]
fn parse_stance_enter_wrath() {
    let c = skill("进入 愤怒 。 造成 6 点伤害。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.stance, StanceEffect::EnterWrath);
}

#[test]
fn parse_stance_enter_calm() {
    let c = skill("进入 宁静 。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.stance, StanceEffect::EnterCalm);
}

#[test]
fn parse_stance_exit() {
    let c = skill("退出 姿态 。 获得 4 点 力量 。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.stance, StanceEffect::ExitStance);
}

#[test]
fn parse_stance_enter_divinity() {
    let c = skill("进入 神格 。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.stance, StanceEffect::EnterDivinity);
}

#[test]
fn parse_mantra() {
    let c = skill("获得 3 层 真言 。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert_eq!(e.mantra_gain, 3);
}

#[test]
fn pure_block_card_rejected() {
    let c = skill("获得 5 点 格挡 。");
    assert!(parse_card_effect(&c, &zh_locale()).is_none());
}

#[test]
fn pure_dex_card_rejected() {
    let c = skill("获得 2 点 敏捷 。");
    assert!(parse_card_effect(&c, &zh_locale()).is_none());
}

#[test]
fn strength_double_not_counted_as_strength_gain() {
    let c = skill("将你的 力量 翻倍。");
    let e = parse_card_effect(&c, &zh_locale());
    assert!(e.is_none() || e.unwrap().strength_gain == 0);
}

#[test]
fn parse_x_plus_one() {
    let c = card("造成 7 点伤害 X+1 次。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert!(e.x_cost);
    assert_eq!(
        e.damage,
        Some(DamageEffect {
            amount: 7,
            hits: HitCount::XPlus(1),
            target_type: TargetType::Targeted,
        })
    );
}

#[test]
fn parse_x_cost_hits() {
    let c = card("造成 8 点伤害 X 次。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert!(e.x_cost);
    assert_eq!(e.damage.unwrap().hits, HitCount::XTimes);
}
