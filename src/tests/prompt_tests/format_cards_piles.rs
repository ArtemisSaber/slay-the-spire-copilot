use super::*;

#[test]
fn compact_pile_aggregates_duplicates() {
    let locale = test_locale();
    let cards = vec![card("Strike_R", "打击", 1, "ATTACK"); 3];
    let output = compact_pile("=== 抽牌堆", &cards, &locale);
    assert!(output.contains("抽牌堆（3张）"));
    assert!(output.contains("打击×3"));
}

#[test]
fn compact_pile_empty() {
    let locale = test_locale();
    let cards: Vec<CardInfo> = vec![];
    let output = compact_pile("=== 抽牌堆", &cards, &locale);
    assert_eq!(output, "=== 抽牌堆（0张）\n");
}

#[test]
fn build_hand_section_when_empty() {
    let locale = test_locale();
    let state = NormalizedState {
        hand_cards: vec![],
        ..test_state()
    };
    let output = build_hand_section(&state, &locale);
    assert!(output.is_empty());
}

#[test]
fn build_hand_section_with_cards() {
    let locale = test_locale();
    let state = NormalizedState {
        hand_cards: vec![
            card("Strike_R", "打击", 1, "ATTACK"),
            card("Defend_R", "防御", 1, "SKILL"),
        ],
        ..test_state()
    };
    let output = build_hand_section(&state, &locale);
    assert!(output.contains("手牌（2张"));
    assert!(output.contains("打击"));
    assert!(output.contains("防御"));
}

#[test]
fn compact_pile_single_card() {
    let locale = test_locale();
    let cards = vec![card("Bash", "痛击", 2, "ATTACK")];
    let output = compact_pile("=== 弃牌堆", &cards, &locale);
    assert!(output.contains("弃牌堆（1张）"));
    assert!(output.contains("痛击"));
    assert!(!output.contains("×"));
}

#[test]
fn compact_pile_exhaust() {
    let locale = test_locale();
    let cards = vec![
        card("Slimed", "黏液", 1, "STATUS"),
        card("Slimed", "黏液", 1, "STATUS"),
        card("Dazed", "晕眩", 1, "STATUS"),
    ];
    let output = compact_pile("=== 已消耗", &cards, &locale);
    assert!(output.contains("已消耗（3张）"));
    assert!(output.contains("黏液×2"));
    assert!(output.contains("晕眩"));
}

#[test]
fn compact_pile_mixed_duplicates_sorted() {
    let locale = test_locale();
    let cards = vec![
        card("Defend_R", "防御", 1, "SKILL"),
        card("Strike_R", "打击", 1, "ATTACK"),
        card("Defend_R", "防御", 1, "SKILL"),
    ];
    let output = compact_pile("=== 抽牌堆", &cards, &locale);
    assert!(output.contains("防御×2"));
    assert!(output.contains("打击"));
}
