use super::*;

#[test]
fn format_card_includes_name_cost_type_and_description() {
    let locale = test_locale();
    let c = CardInfo {
        name: "上勾拳".into(),
        description: "造成 13 点伤害。\n给予 1 层 虚弱 。".into(),
        ..card("Uppercut", "上勾拳", 2, "ATTACK")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("上勾拳"));
    assert!(out.contains("2费/攻击"));
    assert!(out.contains("13 点伤害"));
    assert!(out.contains("1 层 虚弱"));
}

#[test]
fn format_card_matches_exact_locale_template() {
    let locale = test_locale();
    let c = CardInfo {
        name: "上勾拳".into(),
        description: "造成 13 点伤害。\n给予 1 层 虚弱 。".into(),
        ..card("Uppercut", "上勾拳", 2, "ATTACK")
    };

    assert_eq!(
        format_card(&c, &locale),
        "上勾拳(2费/攻击) — 造成 13 点伤害。\n给予 1 层 虚弱。"
    );
}

#[test]
fn format_card_shows_plus_for_upgraded() {
    let locale = test_locale();
    let c = CardInfo {
        name: "防御".into(),
        description: "获得 8 点 格挡 。".into(),
        upgraded: true,
        ..card("Defend_R", "防御", 1, "SKILL")
    };
    let out = format_card(&c, &locale);
    assert!(out.starts_with("+"));
    assert!(out.contains("8 点 格挡"));
}

#[test]
fn format_card_curse_type() {
    let locale = test_locale();
    let c = CardInfo {
        card_type: "CURSE".into(),
        ..card("Shame", "羞耻", -2, "CURSE")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("诅咒"));
}

#[test]
fn format_card_status_type() {
    let locale = test_locale();
    let c = CardInfo {
        card_type: "STATUS".into(),
        ..card("Slimed", "黏液", 1, "STATUS")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("状态"));
}

#[test]
fn format_card_power_type() {
    let locale = test_locale();
    let c = CardInfo {
        card_type: "POWER".into(),
        ..card("Demon Form", "恶魔形态", 3, "POWER")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("能力"));
}

#[test]
fn format_card_unknown_type_fallback() {
    let locale = test_locale();
    let c = CardInfo {
        card_type: "STRANGE".into(),
        ..card("Weird", "奇怪", 0, "STRANGE")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("STRANGE"));
}

#[test]
fn format_card_without_description() {
    let locale = test_locale();
    let c = CardInfo {
        name: "打击".into(),
        description: String::new(),
        ..card("Strike_R", "打击", 1, "ATTACK")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("打击(1费/攻击)"));
    assert!(!out.contains('—'));
}

#[test]
fn format_card_skill_type() {
    let locale = test_locale();
    let c = CardInfo {
        card_type: "SKILL".into(),
        description: "获得 8 点 格挡 。".into(),
        ..card("Defend_R", "防御", 1, "SKILL")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("技能"));
    assert!(out.contains("防御"));
}
