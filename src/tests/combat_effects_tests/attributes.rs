use super::*;

#[test]
fn parse_energy_gain_chinese_char() {
    assert_eq!(super::parse_energy_gain("获得 能量 能量 。"), 2);
}

#[test]
fn parse_energy_gain_zero() {
    assert_eq!(super::parse_energy_gain("no energy here"), 0);
}

#[test]
fn parse_strength_gain_en() {
    let c = skill("Gain 3 Strength.");
    let e = parse_card_effect(&c, &en_locale()).unwrap();
    assert_eq!(e.strength_gain, 3);
}

#[test]
fn parse_strength_gain_no_match() {
    assert_eq!(
        super::parse_strength_gain("no strength here.", &en_locale()),
        0
    );
}

#[test]
fn parse_strength_gain_zh_loss_skipped() {
    let c = skill("失去 3 点 力量 。");
    let e = parse_card_effect(&c, &zh_locale());
    assert!(e.is_none() || e.unwrap().strength_gain == 0);
}

#[test]
fn parse_strength_gain_en_loss_skipped() {
    let c = skill("lose 3 Strength.");
    let e = parse_card_effect(&c, &en_locale());
    assert!(e.is_none() || e.unwrap().strength_gain == 0);
}

#[test]
fn parse_vulnerable_en_with_amount() {
    let c = skill("Apply 2 Vulnerable.");
    let e = parse_card_effect(&c, &en_locale()).unwrap();
    assert_eq!(e.vulnerable, Some(2));
}

#[test]
fn parse_vulnerable_en_default_one() {
    let c = skill("Apply Vulnerable.");
    let e = parse_card_effect(&c, &en_locale()).unwrap();
    assert_eq!(e.vulnerable, Some(1));
}

#[test]
fn parse_vulnerable_no_match() {
    assert_eq!(super::parse_vulnerable("just a skill", &en_locale()), None);
}

#[test]
fn parse_stance_en_wrath() {
    assert_eq!(
        parse_stance("Enter Wrath.", &en_locale()),
        StanceEffect::EnterWrath
    );
}

#[test]
fn parse_stance_en_calm() {
    assert_eq!(
        parse_stance("Enter Calm.", &en_locale()),
        StanceEffect::EnterCalm
    );
}

#[test]
fn parse_stance_en_exit() {
    assert_eq!(
        parse_stance("Exit your stance.", &en_locale()),
        StanceEffect::ExitStance
    );
}

#[test]
fn parse_stance_en_divinity() {
    assert_eq!(
        parse_stance("Enter Divinity.", &en_locale()),
        StanceEffect::EnterDivinity
    );
}

#[test]
fn parse_stance_ja_wrath() {
    assert_eq!(
        parse_stance("憤怒に 入る 。", &ja_locale()),
        StanceEffect::EnterWrath
    );
}

#[test]
fn parse_stance_ja_calm() {
    assert_eq!(
        parse_stance("平静に 入る 。", &ja_locale()),
        StanceEffect::EnterCalm
    );
}

#[test]
fn parse_stance_ja_exit() {
    assert_eq!(
        parse_stance("構え を 解除 する。", &ja_locale()),
        StanceEffect::ExitStance
    );
}

#[test]
fn parse_stance_ja_divinity() {
    assert_eq!(
        parse_stance("神格に 入る 。", &ja_locale()),
        StanceEffect::EnterDivinity
    );
}

#[test]
fn parse_stance_ko_wrath() {
    assert_eq!(
        parse_stance("분노에 들어갑니다 。", &ko_locale()),
        StanceEffect::EnterWrath
    );
}

#[test]
fn parse_stance_ko_calm() {
    assert_eq!(
        parse_stance("평온에 들어갑니다 。", &ko_locale()),
        StanceEffect::EnterCalm
    );
}

#[test]
fn parse_stance_ko_exit() {
    assert_eq!(
        parse_stance("자세를 해제합니다 。", &ko_locale()),
        StanceEffect::ExitStance
    );
}

#[test]
fn parse_stance_ko_divinity() {
    assert_eq!(
        parse_stance("신격에 들어갑니다 。", &ko_locale()),
        StanceEffect::EnterDivinity
    );
}

#[test]
fn parse_stance_none() {
    assert_eq!(
        parse_stance("just a normal card.", &en_locale()),
        StanceEffect::None
    );
}

#[test]
fn parse_mantra_en_with_amount() {
    assert_eq!(super::parse_mantra("Gain 2 Mantra.", &en_locale()), 2);
}

#[test]
fn parse_mantra_en_default_one() {
    assert_eq!(super::parse_mantra("Gain Mantra.", &en_locale()), 1);
}

#[test]
fn parse_mantra_no_match() {
    assert_eq!(super::parse_mantra("just a skill", &en_locale()), 0);
}

#[test]
fn parse_execute_en_fallback() {
    let custom_locale = EffectParserLocale {
        execute_keywords: vec![],
        ..en_locale()
    };
    assert_eq!(
        super::parse_execute("If HP is 15 or less, set its HP to 0.", &custom_locale),
        Some(15)
    );
}

#[test]
fn parse_execute_en_hp_is() {
    let c = skill("If HP is 10, set its HP to 0.");
    let e = parse_card_effect(&c, &en_locale()).unwrap();
    assert_eq!(e.execute, Some(10));
}

#[test]
fn parse_execute_no_match() {
    assert_eq!(super::parse_execute("no execute here.", &en_locale()), None);
}
