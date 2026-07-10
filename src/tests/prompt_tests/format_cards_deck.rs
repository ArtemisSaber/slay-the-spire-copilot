use super::*;

#[test]
fn deck_section_groups_single_type() {
    let locale = test_locale();
    let cards = vec![card("Strike_R", "打击", 1, "ATTACK"); 5];
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("攻击（5张）："));
    assert!(output.contains("打击(1费)（共5张）"));
    assert!(!output.contains("技能"));
    assert!(!output.contains("能力"));
}

#[test]
fn deck_section_shows_cards_by_type_with_counts() {
    let locale = test_locale();
    let mut cards = vec![card("Strike_R", "打击", 1, "ATTACK"); 4];
    cards.extend(vec![card("Defend_R", "防御", 1, "SKILL"); 2]);
    let output = format_deck_section(&cards, &locale);

    assert!(output.contains("=== 卡组 ==="));
    assert!(output.contains("攻击（4张）："));
    assert!(output.contains("技能（2张）："));
    assert!(output.contains("打击(1费)（共4张）"));
    assert!(output.contains("防御(1费)（共2张）"));
}

#[test]
fn deck_section_empty() {
    let locale = test_locale();
    let cards: Vec<CardInfo> = vec![];
    let output = format_deck_section(&cards, &locale);
    assert_eq!(output, "");
}

#[test]
fn deck_section_single_cards() {
    let locale = test_locale();
    let cards = vec![card("Strike_R", "打击", 1, "ATTACK")];
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("打击(1费)"));
    assert!(!output.contains("（共"));
}

#[test]
fn deck_section_includes_curse_and_status() {
    let locale = test_locale();
    let mut cards = vec![card("Strike_R", "打击", 1, "ATTACK")];
    cards.push(CardInfo {
        card_type: "CURSE".into(),
        ..card("Shame", "羞耻", -2, "CURSE")
    });
    cards.push(CardInfo {
        card_type: "STATUS".into(),
        ..card("Slimed", "黏液", 1, "STATUS")
    });
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("攻击（1张）"));
    assert!(output.contains("诅咒（1张）"));
    assert!(output.contains("状态（1张）"));
    assert!(output.contains("羞耻"));
    assert!(output.contains("黏液"));
}

#[test]
fn deck_section_upgraded_with_count() {
    let locale = test_locale();
    let cards = vec![
        CardInfo {
            upgraded: true,
            name: "打击+".into(),
            ..card("Strike_R", "打击+", 1, "ATTACK")
        },
        CardInfo {
            upgraded: true,
            name: "打击+".into(),
            ..card("Strike_R", "打击+", 1, "ATTACK")
        },
    ];
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("+打击+(1费)（共2张）"));
}

#[test]
fn deck_section_card_with_description() {
    let locale = test_locale();
    let cards = vec![CardInfo {
        name: "痛击".into(),
        description: "造成 8 点伤害。\n给予 2 层 易伤 。".into(),
        ..card("Bash", "痛击", 2, "ATTACK")
    }];
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("痛击(2费)"));
    assert!(output.contains("8 点伤害"));
    assert!(output.contains("2 层 易伤"));
}

#[test]
fn deck_section_power_with_count() {
    let locale = test_locale();
    let cards = vec![
        CardInfo {
            card_type: "POWER".into(),
            ..card("Demon Form", "恶魔形态", 3, "POWER")
        },
        CardInfo {
            card_type: "POWER".into(),
            ..card("Demon Form", "恶魔形态", 3, "POWER")
        },
    ];
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("能力（2张）"));
    assert!(output.contains("恶魔形态(3费)（共2张）"));
}
