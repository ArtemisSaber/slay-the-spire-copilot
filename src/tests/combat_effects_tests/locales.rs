use super::*;

#[test]
fn parse_card_effect_en_strike() {
    let c = card("Deal 6 damage.");
    let e = parse_card_effect(&c, &en_locale()).unwrap();
    let dmg = e.damage.as_ref().unwrap();
    assert_eq!(dmg.amount, 6);
    assert_eq!(dmg.target_type, TargetType::Targeted);
}

#[test]
fn parse_card_effect_en_aoe() {
    let c = card("Deal 8 damage to all enemies.");
    let e = parse_card_effect(&c, &en_locale()).unwrap();
    let dmg = e.damage.as_ref().unwrap();
    assert_eq!(dmg.amount, 8);
    assert_eq!(dmg.target_type, TargetType::AoE);
}

#[test]
fn parse_mantra_ja() {
    let c = skill("2 マントラ を得る。");
    let e = parse_card_effect(&c, &ja_locale()).unwrap();
    assert_eq!(e.mantra_gain, 2);
}

#[test]
fn parse_vulnerable_ja() {
    let c = skill("敵に 2 脆弱 を与える。");
    let e = parse_card_effect(&c, &ja_locale()).unwrap();
    assert_eq!(e.vulnerable, Some(2));
}

#[test]
fn parse_strength_gain_ja() {
    let c = skill("3 筋力 を得る。");
    let e = parse_card_effect(&c, &ja_locale()).unwrap();
    assert_eq!(e.strength_gain, 3);
}

#[test]
fn parse_execute_ja() {
    let c = skill("HPが15なら0にする。");
    let e = parse_card_effect(&c, &ja_locale()).unwrap();
    assert_eq!(e.execute, Some(15));
}

#[test]
fn parse_exhaust_ja_self() {
    let c = skill("廃棄 する。");
    let e = parse_card_effect(&c, &ja_locale()).unwrap();
    assert_eq!(e.exhaust, ExhaustKind::Self_);
}

#[test]
fn parse_mantra_ko() {
    let c = skill("만트라 2 를 얻습니다。");
    let e = parse_card_effect(&c, &ko_locale()).unwrap();
    assert_eq!(e.mantra_gain, 2);
}

#[test]
fn parse_vulnerable_ko() {
    let c = skill("적에게 취약 2 를 줍니다。");
    let e = parse_card_effect(&c, &ko_locale()).unwrap();
    assert_eq!(e.vulnerable, Some(2));
}

#[test]
fn parse_strength_gain_ko() {
    let c = skill("힘 3 을 얻습니다。");
    let e = parse_card_effect(&c, &ko_locale()).unwrap();
    assert_eq!(e.strength_gain, 3);
}

#[test]
fn parse_execute_ko() {
    let c = skill("HP가 15 이하라면 체력을 0으로 만듭니다。");
    let e = parse_card_effect(&c, &ko_locale()).unwrap();
    assert_eq!(e.execute, Some(15));
}

#[test]
fn parse_exhaust_ko_self() {
    let c = skill("소멸 됩니다。");
    let e = parse_card_effect(&c, &ko_locale()).unwrap();
    assert_eq!(e.exhaust, ExhaustKind::Self_);
}

#[test]
fn parse_x_cost_zh_x_in_desc() {
    let c = skill("造成 X 点伤害。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert!(e.x_cost);
}

#[test]
fn parse_card_effect_power_rejected() {
    let c = power("At the start of your turn, gain 2 Strength.");
    let e = parse_card_effect(&c, &en_locale()).unwrap();
    assert_eq!(e.strength_gain, 2);
}
