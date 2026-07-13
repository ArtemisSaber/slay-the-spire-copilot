use super::*;

#[test]
fn parse_exhaust_zh_non_attacks() {
    assert_eq!(
        parse_exhaust("消耗 所有非攻击牌。", &zh_locale()),
        ExhaustKind::NonAttacks
    );
}

#[test]
fn parse_exhaust_zh_attacks() {
    assert_eq!(
        parse_exhaust("消耗 所有攻击牌。", &zh_locale()),
        ExhaustKind::Attacks
    );
}

#[test]
fn parse_exhaust_zh_random() {
    assert_eq!(
        parse_exhaust("消耗 随机 一张牌。", &zh_locale()),
        ExhaustKind::Random
    );
}

#[test]
fn parse_exhaust_zh_chosen_select() {
    assert_eq!(
        parse_exhaust("消耗 选择 的一张牌。", &zh_locale()),
        ExhaustKind::Chosen
    );
}

#[test]
fn parse_exhaust_zh_chosen_one_card() {
    assert_eq!(
        parse_exhaust("消耗 一张牌。", &zh_locale()),
        ExhaustKind::Chosen
    );
}

#[test]
fn parse_exhaust_en_self() {
    assert_eq!(parse_exhaust("Exhaust.", &en_locale()), ExhaustKind::Self_);
}

#[test]
fn parse_exhaust_en_all() {
    assert_eq!(
        parse_exhaust("Exhaust all cards.", &en_locale()),
        ExhaustKind::All
    );
}

#[test]
fn parse_exhaust_en_non_attacks() {
    assert_eq!(
        parse_exhaust("Exhaust all non-attack cards.", &en_locale()),
        ExhaustKind::NonAttacks
    );
}

#[test]
fn parse_exhaust_en_attacks() {
    assert_eq!(
        parse_exhaust("Exhaust all attack cards.", &en_locale()),
        ExhaustKind::Attacks
    );
}

#[test]
fn parse_exhaust_en_random() {
    assert_eq!(
        parse_exhaust("Exhaust a random card.", &en_locale()),
        ExhaustKind::Random
    );
}

#[test]
fn parse_exhaust_en_chosen() {
    assert_eq!(
        parse_exhaust("Choose a card to Exhaust.", &en_locale()),
        ExhaustKind::Chosen
    );
}

#[test]
fn parse_exhaust_no_match() {
    assert_eq!(
        parse_exhaust("just a skill", &en_locale()),
        ExhaustKind::None
    );
}

#[test]
fn parse_x_cost_zh_spend_all() {
    let c = skill("花费所有 [E] 。获得 X 点 力量 。");
    let e = parse_card_effect(&c, &zh_locale()).unwrap();
    assert!(e.x_cost);
}

#[test]
fn parse_x_cost_en_spend_all() {
    let c = skill("Spend all [E]. Gain X Strength.");
    let e = parse_card_effect(&c, &en_locale()).unwrap();
    assert!(e.x_cost);
}
